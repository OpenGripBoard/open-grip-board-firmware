use std::{
    sync::mpsc::{self, Sender},
    time::{Duration, Instant},
};

use anyhow::Result;

use esp_idf_hal::{
    delay::Ets,
    gpio::{Input, Output, PinDriver},
    i2c::I2cDriver,
    peripherals::Peripherals,
};

use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection};
use hx711::Hx711;
use open_grip_board_firmware::{app_errors::AppError, view_model::AppViewModel};
use open_grip_board_firmware::{
    init::{init_display, init_i2c, init_load_cell, init_mqtt, init_wifi},
    views::AppDisplay,
};

fn main() -> Result<()> {
    // Required by ESP-IDF
    esp_idf_svc::sys::link_patches();

    // set log level
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let Peripherals {
        modem,
        i2c0,
        pins,
        spi2,
        ..
    } = peripherals;

    let mut mipidsi_buffer = [0u8; 512];
    let mut framebuffer = vec![Rgb565::BLACK; 320 * 170];

    let (mut backlight, mut display) = init_display(
        pins.gpio4,
        pins.gpio5,
        pins.gpio6,
        pins.gpio7,
        pins.gpio14,
        pins.gpio15,
        spi2,
        &mut mipidsi_buffer,
    )?;

    // Touchscreen
    let i2c = init_i2c(
        pins.gpio18,
        pins.gpio8,
        i2c0,
    )?;

    // Load cell
    let mut hx711 = init_load_cell(
        pins.gpio21,
        pins.gpio22,
    )?;

    // Backlight ON (active low)
    backlight.set_low()?;
    let mut backlight_is_on: bool = true;

    // Touchscreen over i2c
    let (tx_touch, rx_touch) = mpsc::channel::<Option<(u16, u16)>>();

    // poll touchscreen in background thread
    std::thread::spawn(move || touch_polling_loop(i2c, tx_touch));

    let mut model = AppViewModel::new();
    let mut app_display = AppDisplay::new(&mut framebuffer);
    model.draw(&mut app_display)?;
    display
        .set_pixels(0, 0, 319, 169, framebuffer.iter().copied())
        .map_err(|e| anyhow::anyhow!("Display update failed: {:?}", e))?;

    let mut last_touch: Instant = Instant::now();

    // WiFi
    let mut wifi = init_wifi(modem)?;

    log::info!("HX711 initialized");
    // give the hx711 some time before first read
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let tare_offset = tare_load_cell(&mut hx711, 20)?;
    let (tx_load, rx_load) = mpsc::channel::<f32>();
    // poll load cell in background thread
    std::thread::spawn(move || load_cell_polling_loop(&mut hx711, tx_load, tare_offset, 0.001));

    let mut last_wifi_retry = Instant::now();
    let mut should_redraw = false;
    let mut mqtt_client: Option<EspMqttClient> = None;
    let mut mqtt_connection: Option<EspMqttConnection> = None;
    const BOARD_NAME: &str = env!("BOARD_NAME");
    loop {
        if model.is_recording {
            let mut latest = None;
            while let Ok(reading) = rx_load.try_recv() {
                latest = Some(reading);
            }
            if let Some(reading) = latest {
                log::info!("reading is: {}", reading);
                model.current_reading = reading;

                model.past_readings.pop_front();
                model.past_readings.push_back(reading);
                should_redraw = true;
            }
        }

        if wifi.is_connected()? {
            let ip_info = wifi.sta_netif().get_ip_info()?;
            if ip_info.ip != std::net::Ipv4Addr::UNSPECIFIED {
                if !model.wifi_is_connected {
                    log::info!("Wi-Fi connected with IP: {:?}", ip_info.ip);
                    should_redraw = true;
                }
                model.wifi_is_connected = true;
            } else {
                if model.wifi_is_connected {
                    should_redraw = true
                }
                model.wifi_is_connected = false;
            }
        } else {
            if model.wifi_is_connected {
                should_redraw = true
            }
            model.wifi_is_connected = false;
            mqtt_client = None;
            mqtt_connection = None;
            if last_wifi_retry.elapsed() >= Duration::from_secs(5) {
                log::info!("Wi-Fi disconnected, attempting reconnect...");
                wifi.connect()?;
                last_wifi_retry = Instant::now();
            }
        }

        if mqtt_client.is_none() && model.wifi_is_connected {
            let (client, connection) = init_mqtt()?;

            std::thread::spawn(move || {
                let mut connection = connection;

                while let Ok(event) = connection.next() {
                    log::debug!("MQTT event: {:?}", event.payload());
                }

                log::warn!("MQTT connection thread ended");
            });

            mqtt_client = Some(client);

            log::info!("MQTT client initialized");
        }

        while let Ok(event) = rx_touch.try_recv() {
            last_touch = Instant::now();
            if backlight_is_on && model.on_touch(event)? {
                should_redraw = true;
            }
        }
        if should_redraw {
            let mut app_display = AppDisplay::new(&mut framebuffer);
            model.draw(&mut app_display)?;
            display
                .set_pixels(0, 0, 319, 169, framebuffer.iter().copied())
                .map_err(|e| anyhow::anyhow!("Display update failed: {:?}", e))?;
            should_redraw = false;
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
        let should_be_on = last_touch.elapsed() <= Duration::from_secs(60);
        if should_be_on != backlight_is_on {
            if should_be_on {
                backlight.set_low()?;
            } else {
                backlight.set_high()?;
            }
            backlight_is_on = should_be_on;
        }
    }
}

fn touch_polling_loop(mut i2c: I2cDriver<'static>, tx: Sender<Option<(u16, u16)>>) {
    const TOUCH_ADDR: u8 = 0x15;
    let mut buf = [0u8; 7];
    let mut was_touched = false;

    loop {
        if i2c.write_read(TOUCH_ADDR, &[0x00], &mut buf, 100).is_ok() {
            let touches = buf[2];
            if touches > 0 {
                let y = 170 - ((((buf[3] & 0x0f) as u16) << 8) | buf[4] as u16);
                let x = (((buf[5] & 0x0f) as u16) << 8) | buf[6] as u16;
                if !was_touched {
                    tx.send(Some((x, y))).ok();
                    was_touched = true;
                }
            } else if was_touched {
                tx.send(None).ok();
                was_touched = false;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn tare_load_cell(
    hx711: &mut Hx711<Ets, PinDriver<'_, Input>, PinDriver<'_, Output>>,
    samples: u32,
) -> Result<i32, AppError> {
    if samples == 0 {
        return Err(AppError::App("samples must be > 0".to_string()));
    }
    let mut total: i64 = 0;
    for _ in 0..samples {
        let raw = hx711
            .retrieve()
            .map_err(|_| AppError::App(format!("load cell read failed")))?;
        total += raw as i64;

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok((total / samples as i64) as i32)
}

fn load_cell_polling_loop(
    hx711: &mut Hx711<Ets, PinDriver<'_, Input>, PinDriver<'_, Output>>,
    tx: Sender<f32>,
    tare_offset: i32,
    calibration_multiplier: f32,
) {
    let n = 3;
    loop {
        let mut value: f32 = 0.0;
        let mut actual = 0;
        while actual < n {
            match hx711.retrieve() {
                Ok(raw) => {
                    log::debug!("HX711 raw: {}", raw as f32);
                    value += raw as f32
                }
                Err(err) => {
                    log::error!("HX711 error: {:?}", err);
                }
            }
            // Don't hammer the HX711.
            // The HX711 normally operates at 10 SPS or 80 SPS
            // depending on the RATE configuration.
            std::thread::sleep(std::time::Duration::from_millis(100));
            actual += 1;
        }
        value /= n as f32;
        value -= tare_offset as f32;
        tx.send(value * calibration_multiplier).ok();
    }
}

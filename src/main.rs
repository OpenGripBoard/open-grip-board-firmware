use std::sync::mpsc::{self, Sender};

use anyhow::Result;

use embassy_time::{Duration, Instant};
use embedded_hal::spi::MODE_0;

use esp_idf_hal::{
    delay::Ets,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{config, SpiDeviceDriver, SpiDriverConfig},
    units::FromValueType,
};

use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, ColorOrder, Orientation, Rotation},
    Builder,
};
use open_grip_board_firmware::view_model::AppViewModel;
use open_grip_board_firmware::views::AppDisplay;

fn main() -> Result<()> {
    // Required by ESP-IDF
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;

    // Waveshare ESP32-C6-LCD-1.9
    //
    // LCD:
    // MOSI = GPIO4
    // SCLK = GPIO5
    // DC   = GPIO6
    // CS   = GPIO7
    // RST  = GPIO14
    // BL   = GPIO15 (LOW = ON)

    let sclk = peripherals.pins.gpio5;
    let mosi = peripherals.pins.gpio4;
    let dc = PinDriver::output(peripherals.pins.gpio6)?;
    let cs = peripherals.pins.gpio7;
    let rst = PinDriver::output(peripherals.pins.gpio14)?;
    let mut backlight = PinDriver::output(peripherals.pins.gpio15)?;

    // Backlight OFF (active low)
    backlight.set_high()?;
    let mut delay = Ets;

    // SPI
    let spi_config = config::Config::new()
        .baudrate(40.MHz().into())
        .data_mode(MODE_0);

    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        sclk,
        mosi,
        None::<esp_idf_hal::gpio::AnyIOPin<'_>>,
        Some(cs),
        &SpiDriverConfig::new(),
        &spi_config,
    )?;

    // framebuffer to build the screen plus second buffer for the mipidsi SPI interface.
    let mut framebuffer = vec![Rgb565::BLACK; 320 * 170];
    let mut buffer = [0u8; 512];
    let display_interface = SpiInterface::new(spi, dc, &mut buffer);

    // ST7789V2
    let mut display = Builder::new(ST7789, display_interface)
        .display_size(170, 320)
        .display_offset(35, 0)
        .color_order(ColorOrder::Rgb)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .reset_pin(rst)
        .init(&mut delay)
        .map_err(|e| anyhow::anyhow!("Display initialization failed: {:?}", e))?;

    // Backlight ON (active low)
    backlight.set_low()?;
    let mut backlight_is_on: bool = true;

    // Touchscreen over i2c
    let i2c_config = I2cConfig::new()
        .baudrate(300.kHz().into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio18, // SDA
        peripherals.pins.gpio8,  // SCL
        &i2c_config,
    )
    .map_err(|e| anyhow::anyhow!("I2C init failed: {:?}", e))?;
    let (tx, rx) = mpsc::channel::<Option<(u16, u16)>>();

    // poll touchscreen in background thread
    std::thread::spawn(move || touch_polling_loop(i2c, tx));

    let mut model = AppViewModel::new();
    let mut app_display = AppDisplay::new(&mut framebuffer);
    model.draw(&mut app_display)?;
    display
        .set_pixels(0, 0, 319, 169, framebuffer.iter().copied())
        .map_err(|e| anyhow::anyhow!("Display update failed: {:?}", e))?;

    let mut last_touch: Instant = Instant::now();

    loop {
        while let Ok(event) = rx.try_recv() {
            last_touch = Instant::now();
            if backlight_is_on && model.on_touch(event)? {
                let mut app_display = AppDisplay::new(&mut framebuffer);
                model.draw(&mut app_display)?;
                display
                    .set_pixels(0, 0, 319, 169, framebuffer.iter().copied())
                    .map_err(|e| anyhow::anyhow!("Display update failed: {:?}", e))?;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
        let should_be_on = last_touch.elapsed() <= Duration::from_secs(10);
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

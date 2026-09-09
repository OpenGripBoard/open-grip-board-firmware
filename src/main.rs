use std::sync::mpsc::{self, Sender};

use anyhow::Result;

use embedded_hal::spi::MODE_0;

use esp_idf_hal::{
    delay::Ets,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{config, SpiDeviceDriver, SpiDriverConfig},
    units::FromValueType,
};

use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, ColorOrder, Orientation, Rotation},
    Builder,
};
use open_grip_board_firmware::{
    view_model::{AppViewModel, Language},
    views::{draw_screen, View},
};

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

    // mipidsi SPI interface needs a buffer.
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

    let model = AppViewModel {
        view: View::Boot,
        wifi_is_connected: false,
        is_recording: false,
        max_weight: 0,
        max_weight_avg: 0,
        language: Language::De,
    };

    draw_screen(&mut display, &model)?;

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

    let (tx, rx) = mpsc::channel::<(u16, u16)>();

    std::thread::spawn(move || touch_polling_loop(i2c, tx));

    loop {
        while let Ok((x, y)) = rx.try_recv() {
            println!("touch: x={} y={}", x, y);
            // app_state.handle_touch(x, y);
            if (0..320).contains(&x) && (0..170).contains(&y) {
                draw_screen(&mut display, &model)?;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

fn touch_polling_loop(mut i2c: I2cDriver<'static>, tx: Sender<(u16, u16)>) {
    const TOUCH_ADDR: u8 = 0x15;
    let mut buf = [0u8; 7];

    loop {
        if i2c.write_read(TOUCH_ADDR, &[0x00], &mut buf, 100).is_ok() {
            let touches = buf[2];
            if touches > 0 {
                let y = 170 - ((((buf[3] & 0x0f) as u16) << 8) | buf[4] as u16);
                let x = (((buf[5] & 0x0f) as u16) << 8) | buf[6] as u16;
                tx.send((x, y)).ok();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

use anyhow::Result;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

use embedded_hal::spi::MODE_0;

use esp_idf_hal::{
    delay::Ets,
    gpio::PinDriver,
    peripherals::Peripherals,
    spi::{config, SpiDeviceDriver, SpiDriverConfig},
    units::FromValueType,
};

use mipidsi::{
    Builder, interface::SpiInterface, models::ST7789, options::{ColorInversion, ColorOrder, Orientation, Rotation},
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

    // Clear display
    display
        .clear(Rgb565::BLACK)
        .map_err(|e| anyhow::anyhow!("Display initialization failed: {:?}", e))?;

    // Draw red rectangle
    Rectangle::new(Point::new(10, 10), Size::new(150, 100))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut display)
        .unwrap();

    // Text
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    Text::new("Hello ESP32-C6!", Point::new(20, 150), text_style)
        .draw(&mut display)
        .unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

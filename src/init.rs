use anyhow::Result;

use embedded_hal::spi::MODE_0;

use esp_idf_hal::{
    delay::Ets,
    gpio::*,
    i2c::{I2cConfig, I2cDriver, I2C0},
    spi::{config, SpiDeviceDriver, SpiDriverConfig},
    units::FromValueType,
};

use esp_idf_hal::{
    gpio::Pull,
    modem::Modem,
    spi::SPI2,
};
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mqtt::client::MqttClientConfiguration,
    wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use hx711::Hx711;

use mipidsi;
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorInversion, ColorOrder, Orientation, Rotation},
    Builder, Display,
};

use crate::app_errors::AppError;

pub fn init_wifi<'a>(modem: Modem<'a>) -> Result<EspWifi<'a>, AppError> {
    // init wifi
    const WIFI_SSID: &str = env!("WIFI_SSID");
    const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
    log::info!("initialize Wifi ...");
    let sysloop = EspSystemEventLoop::take()?;
    let mut wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::try_from(WIFI_SSID)?,
        password: heapless::String::try_from(WIFI_PASSWORD)?,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;
    wifi.start()?;
    wifi.connect()?;
    log::info!("Connecting to Wi-Fi (SSID: {})...", WIFI_SSID);
    return Ok(wifi);
}

pub fn init_mqtt<'a>() -> Result<(EspMqttClient<'a>, EspMqttConnection), AppError> {
    const MQTT_URL: &str = env!("MQTT_URL");
    const MQTT_USER: &str = env!("MQTT_USER");
    const MQTT_PASSWORD: &str = env!("MQTT_PASSWORD");
    const BOARD_NAME: &str = env!("BOARD_NAME");

    let (mqtt_client, mqtt_connection) = EspMqttClient::new(
        MQTT_URL,
        &MqttClientConfiguration {
            client_id: Some(&format!("esp32c6-{}", BOARD_NAME)),
            username: Some(MQTT_USER),
            password: Some(MQTT_PASSWORD),
            ..Default::default()
        },
    )?;
    return Ok((mqtt_client, mqtt_connection));
}

pub fn init_load_cell(
    gpio21: Gpio21<'static>,
    gpio22: Gpio22<'static>,
) -> Result<
    Hx711<
        Ets,
        PinDriver<'static, esp_idf_hal::gpio::Input>,
        PinDriver<'static, esp_idf_hal::gpio::Output>,
    >,
    AppError,
> {
    // The pins 3,21,22,23 should be save, 12&13 are SD card
    let dout = PinDriver::input(gpio21, Pull::Floating)?;
    let pd_sck = PinDriver::output(gpio22)?;
    let delay = Ets;
    return Hx711::new(delay, dout, pd_sck)
        .map_err(|e| AppError::App(format!("hx711 init failed {:?}", e)));
}

pub fn init_i2c<'a>(
    gpio18: Gpio18<'static>,
    gpio8: Gpio8<'static>,
    i2c0: I2C0<'static>,
) -> Result<I2cDriver<'a>, AppError> {
    let i2c_config = I2cConfig::new()
        .baudrate(300.kHz().into())
        .sda_enable_pullup(true)
        .scl_enable_pullup(true);

    let i2c = I2cDriver::new(
        i2c0,
        gpio18, // SDA
        gpio8,  // SCL
        &i2c_config,
    )
    .map_err(|e| AppError::App(format!("I2C init failed: {:?}", e)))?;
    return Ok(i2c);
}

pub fn init_display<'a>(
    gpio4: Gpio4<'static>,
    gpio5: Gpio5<'static>,
    gpio6: Gpio6<'static>,
    gpio7: Gpio7<'static>,
    gpio14: Gpio14<'static>,
    gpio15: Gpio15<'static>,
    spi2: SPI2<'static>,
    buffer: &'a mut [u8; 512],
) -> Result<
    (
        PinDriver<'static, Output>,
        Display<
            mipidsi::interface::SpiInterface<
                'a,
                esp_idf_hal::spi::SpiDeviceDriver<'a, esp_idf_hal::spi::SpiDriver<'a>>,
                esp_idf_hal::gpio::PinDriver<'a, esp_idf_hal::gpio::Output>,
            >,
            mipidsi::models::ST7789,
            esp_idf_hal::gpio::PinDriver<'a, esp_idf_hal::gpio::Output>,
        >,
    ),
    AppError,
> {
    // Waveshare ESP32-C6-LCD-1.9
    //
    // LCD:
    // MOSI = GPIO4
    // SCLK = GPIO5
    // DC   = GPIO6
    // CS   = GPIO7
    // RST  = GPIO14
    // BL   = GPIO15 (LOW = ON)

    let sclk = gpio5;
    let mosi = gpio4;
    let dc = PinDriver::output(gpio6)?;
    let cs = gpio7;
    let rst = PinDriver::output(gpio14)?;
    let mut backlight = PinDriver::output(gpio15)?;

    // Backlight OFF (active low)
    backlight.set_high()?;

    // SPI
    let spi_config = config::Config::new()
        .baudrate(40.MHz().into())
        .data_mode(MODE_0);

    let spi = SpiDeviceDriver::new_single(
        spi2,
        sclk,
        mosi,
        None::<esp_idf_hal::gpio::AnyIOPin<'_>>,
        Some(cs),
        &SpiDriverConfig::new(),
        &spi_config,
    )?;

    let display_interface = SpiInterface::new(spi, dc, buffer);
    let mut delay = Ets;
    // ST7789V2
    let display = Builder::new(ST7789, display_interface)
        .display_size(170, 320)
        .display_offset(35, 0)
        .color_order(ColorOrder::Rgb)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .reset_pin(rst)
        .init(&mut delay)
        .map_err(|e| anyhow::anyhow!("Display initialization failed: {:?}", e))?;
    return Ok((backlight, display));
}

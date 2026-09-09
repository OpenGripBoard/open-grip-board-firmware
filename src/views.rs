use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle},
    text::Text,
};

use embedded_vintage_fonts::FONT_12X16;
use esp_idf_hal::{
    gpio::{Output, PinDriver},
    spi::{SpiDeviceDriver, SpiDriver},
};

use mipidsi::{interface::SpiInterface, models::ST7789, Display};

use tinybmp::Bmp;

use crate::{
    app_errors::{AppError, AppResult},
    view_model::AppViewModel,
};

pub enum View {
    Boot,
    HomeScreen,
    Recording,
    LanguageSelection,
    ConnectApp,
}

struct AppColor;

impl AppColor {
    const PRIMARY: Rgb565 = Rgb565::new(5, 18, 20);
    const SECONDARY: Rgb565 = Rgb565::new(17, 40, 28);
    const LIGHT: Rgb565 = Rgb565::new(25, 53, 30);
    const DARK: Rgb565 = Rgb565::new(4, 8, 5);
    const ERROR: Rgb565 = Rgb565::new(25, 13, 3);
    const GOOD: Rgb565 = Rgb565::new(2, 51, 8);
}

struct AppSpacing;

impl AppSpacing {
    const MEDIUM: i32 = 8;
}

struct AppIcon;

impl AppIcon {
    const PLAY: &[u8] = include_bytes!("../icons/play.bmp");
    const GLOBE: &[u8] = include_bytes!("../icons/globe.bmp");
}

type AppDisplay<'a> = Display<
    SpiInterface<'a, SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Output>>,
    ST7789,
    PinDriver<'a, Output>,
>;

pub fn draw_screen<'a>(display: &mut AppDisplay<'_>, model: &AppViewModel) -> AppResult<()> {
    match model.view {
        View::Boot => boot_screen(display),
        View::HomeScreen => home_screen(display, model.wifi_is_connected),
        View::Recording => recording_screen(display),
        View::LanguageSelection => language_selection_screen(display),
        View::ConnectApp => connect_app_screen(display),
    }
}

fn boot_screen<'a>(display: &mut AppDisplay<'_>) -> Result<(), AppError> {
    display.clear(AppColor::DARK)?;
    let text_style = MonoTextStyle::new(&FONT_12X16, AppColor::PRIMARY);
    let screen_height = 170;
    let text_margin: i32 = (screen_height - 16) / 2;
    let center_offset: i32 = 6;
    Text::new(
        "OpenGripBoard",
        Point::new(text_margin, screen_height / 2 + center_offset),
        text_style,
    )
    .draw(display)?;
    Ok(())
}

fn home_screen<'a>(display: &mut AppDisplay<'_>, wifi_is_connected: bool) -> Result<(), AppError> {
    display.clear(AppColor::DARK)?;
    button(
        display,
        AppSpacing::MEDIUM,
        AppSpacing::MEDIUM,
        240,
        48,
        "Training starten",
        AppColor::PRIMARY,
        AppColor::LIGHT,
    )?;
    button(
        display,
        AppSpacing::MEDIUM,
        2 * AppSpacing::MEDIUM + 48,
        240,
        48,
        "App verbinden",
        AppColor::LIGHT,
        AppColor::DARK,
    )?;
    button(
        display,
        2 * AppSpacing::MEDIUM + 240,
        AppSpacing::MEDIUM,
        56,
        28,
        "WiFi",
        if wifi_is_connected {
            AppColor::GOOD
        } else {
            AppColor::ERROR
        },
        AppColor::DARK,
    )?;
    icon_button(
        display,
        2 * AppSpacing::MEDIUM + 240,
        2 * AppSpacing::MEDIUM + 28,
        56,
        56,
        AppIcon::GLOBE,
        AppColor::LIGHT,
    )?;
    Ok(())
}

fn recording_screen<'a>(display: &mut AppDisplay<'_>) -> Result<(), AppError> {
    button(
        display,
        320 - 96 - AppSpacing::MEDIUM,
        AppSpacing::MEDIUM,
        96,
        48,
        "000.0 kg",
        AppColor::LIGHT,
        AppColor::DARK,
    )?;
    icon_button(
        display,
        320 - 96 - AppSpacing::MEDIUM,
        2 * AppSpacing::MEDIUM + 48,
        96,
        98,
        AppIcon::PLAY,
        AppColor::PRIMARY,
    )?;
    Ok(())
}
fn language_selection_screen<'a>(display: &mut AppDisplay<'_>) -> Result<(), AppError> {
    Ok(())
}
fn connect_app_screen<'a>(display: &mut AppDisplay<'_>) -> Result<(), AppError> {
    Ok(())
}

fn button<'a>(
    display: &mut AppDisplay<'_>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    button_text: &str,
    background_color: Rgb565,
    text_color: Rgb565,
) -> AppResult<()> {
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(width, height)),
        Size::new(8, 8),
    )
    .into_styled(PrimitiveStyle::with_fill(background_color))
    .draw(display)?;
    let text_style = MonoTextStyle::new(&FONT_12X16, text_color);
    let button_height: i32 = height.try_into().unwrap();
    let text_margin: i32 = (button_height - 16) / 2;
    let center_offset: i32 = 6;
    Text::new(
        button_text,
        Point::new(x + text_margin, y + button_height / 2 + center_offset),
        text_style,
    )
    .draw(display)?;
    Ok(())
}

fn icon_button(
    display: &mut AppDisplay<'_>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    bmp_data: &[u8],
    background_color: Rgb565,
) -> AppResult<()> {
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(width, height)),
        Size::new(8, 8),
    )
    .into_styled(PrimitiveStyle::with_fill(background_color))
    .draw(display)?;
    let icon_size = 40;
    let button_height: i32 = height.try_into().unwrap();
    let button_width: i32 = width.try_into().unwrap();
    let x_margin: i32 = (button_width - icon_size) / 2;
    let y_margin: i32 = (button_height - icon_size) / 2;
    let bmp = Bmp::from_slice(bmp_data).unwrap();
    Image::new(&bmp, Point::new(x + x_margin, y + y_margin)).draw(display)?;
    Ok(())
}

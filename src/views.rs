use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use esp_idf_hal::{
    gpio::{Output, PinDriver},
    spi::{SpiDeviceDriver, SpiDriver},
};

use mipidsi::{interface::SpiInterface, models::ST7789, Display};

use strum::IntoEnumIterator;

use crate::{
    app_errors::{AppError, AppResult},
    button::Button,
    icon_button::IconButton,
    view_model::{ActionId, AppDrawable, AppViewModel, Language},
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
    const LIGHT: Rgb565 = Rgb565::new(25, 53, 30);
    const DARK: Rgb565 = Rgb565::new(4, 8, 5);
    const ERROR: Rgb565 = Rgb565::new(25, 13, 3);
    const GOOD: Rgb565 = Rgb565::new(2, 51, 8);
}

struct AppSpacing;

impl AppSpacing {
    const MEDIUM: u32 = 8;
}

struct AppIcon;

impl AppIcon {
    const PLAY: &[u8] = include_bytes!("../icons/play.bmp");
    const GLOBE: &[u8] = include_bytes!("../icons/globe.bmp");
}

pub type AppDisplay<'a> = Display<
    SpiInterface<'a, SpiDeviceDriver<'a, SpiDriver<'a>>, PinDriver<'a, Output>>,
    ST7789,
    PinDriver<'a, Output>,
>;

pub fn get_view_elements(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    match model.view {
        View::Boot => boot_screen(),
        View::HomeScreen => home_screen(model),
        View::ConnectApp => connect_app_screen(),
        View::LanguageSelection => language_selection_screen(model),
        View::Recording => recording_screen(model),
    }
}

fn boot_screen() -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    let elements: Vec<Box<dyn AppDrawable>> = vec![
        Box::new(Background::new(AppColor::DARK)),
        Box::new(Button::new(
            ActionId::Default,
            0,
            0,
            320,
            170,
            "OpenGripBoard".to_string(),
            AppColor::PRIMARY,
            AppColor::DARK,
            None,
        )),
    ];
    Ok(elements)
}

fn home_screen(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    let elements: Vec<Box<dyn AppDrawable>> = vec![
        Box::new(Background::new(AppColor::DARK)),
        Box::new(Button::new(
            ActionId::StartTraining,
            AppSpacing::MEDIUM,
            AppSpacing::MEDIUM,
            240,
            48,
            model.language.get_str("start_training").to_string(),
            AppColor::PRIMARY,
            AppColor::LIGHT,
            None,
        )),
        Box::new(Button::new(
            ActionId::ConnectApp,
            AppSpacing::MEDIUM,
            2 * AppSpacing::MEDIUM + 48,
            240,
            48,
            model.language.get_str("connect_app").to_string(),
            AppColor::LIGHT,
            AppColor::DARK,
            None,
        )),
        Box::new(Button::new(
            ActionId::Wifi,
            2 * AppSpacing::MEDIUM + 240,
            AppSpacing::MEDIUM,
            56,
            28,
            "WiFi".to_string(),
            if model.wifi_is_connected {
                AppColor::GOOD
            } else {
                AppColor::ERROR
            },
            AppColor::DARK,
            None,
        )),
        Box::new(IconButton::new(
            ActionId::Globe,
            2 * AppSpacing::MEDIUM + 240,
            2 * AppSpacing::MEDIUM + 28,
            56,
            56,
            AppColor::LIGHT,
            AppIcon::GLOBE.try_into().unwrap(),
        )),
    ];

    Ok(elements)
}

struct Background {
    background_color: Rgb565,
}

impl Background {
    pub fn new(background_color: Rgb565) -> Self {
        Self { background_color }
    }
}

impl AppDrawable for Background {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        display.clear(self.background_color)?;
        Ok(())
    }
    fn eval_touch(&self, _x: &u32, _y: &u32) -> bool {
        false
    }
    fn get_id(&self) -> (&ActionId, &Option<Language>) {
        (&ActionId::Default, &None)
    }
}

fn recording_screen(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    let elements: Vec<Box<dyn AppDrawable>> = vec![
        Box::new(Background::new(AppColor::DARK)),
        Box::new(Button::new(
            ActionId::StartRecording,
            320 - 96 - AppSpacing::MEDIUM,
            AppSpacing::MEDIUM,
            96,
            48,
            format!(
                "{} {}",
                model.current_reading,
                model.language.get_str("kg").to_string()
            ),
            AppColor::LIGHT,
            AppColor::DARK,
            None,
        )),
        Box::new(IconButton::new(
            ActionId::Start,
            320 - 96 - AppSpacing::MEDIUM,
            2 * AppSpacing::MEDIUM + 48,
            96,
            98,
            AppColor::PRIMARY,
            AppIcon::PLAY.try_into().unwrap(),
        )),
    ];
    return Ok(elements);
}

fn language_selection_screen(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    let mut y_pos = AppSpacing::MEDIUM;
    let mut elements: Vec<Box<dyn AppDrawable>> = vec![Box::new(Background::new(AppColor::DARK))];
    for lang in Language::iter() {
        elements.push(Box::new(Button::new(
            ActionId::LanguageSelection,
            AppSpacing::MEDIUM,
            y_pos,
            240,
            48,
            lang.get_str("lang_name").to_string(),
            if model.language == lang {
                AppColor::PRIMARY
            } else {
                AppColor::LIGHT
            },
            if model.language == lang {
                AppColor::LIGHT
            } else {
                AppColor::DARK
            },
            Some(lang),
        )));
        y_pos += AppSpacing::MEDIUM + 48;
    }
    Ok(elements)
}

fn connect_app_screen<'a>() -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    let elements: Vec<Box<dyn AppDrawable>> = vec![];
    Ok(elements)
}

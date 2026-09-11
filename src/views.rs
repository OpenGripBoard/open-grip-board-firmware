use crate::{
    app_errors::{AppError, AppResult},
    button::Button,
    clickable_image::ClickableImage,
    icon_button::IconButton,
    view_model::{ActionId, AppDrawable, AppViewModel, Language},
};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use strum::IntoEnumIterator;
pub enum View {
    Boot,
    HomeScreen,
    Recording,
    LanguageSelection,
    ConnectApp,
    Statistics,
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
    const STOP: &[u8] = include_bytes!("../icons/stop.bmp");
    const EXIT: &[u8] = include_bytes!("../icons/exit.bmp");
}

pub const DISPLAY_WIDTH: usize = 320;
pub const DISPLAY_HEIGHT: usize = 170;
pub const FRAMEBUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
pub struct AppDisplay<'a> {
    framebuffer: &'a mut [Rgb565],
}

impl<'a> AppDisplay<'a> {
    pub fn new(framebuffer: &'a mut [Rgb565]) -> Self {
        assert_eq!(framebuffer.len(), FRAMEBUFFER_SIZE);
        Self { framebuffer }
    }
    pub fn framebuffer(&self) -> &[Rgb565] {
        self.framebuffer
    }
}

impl OriginDimensions for AppDisplay<'_> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

impl DrawTarget for AppDisplay<'_> {
    type Color = Rgb565;
    type Error = AppError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x < 0
                || point.y < 0
                || point.x >= DISPLAY_WIDTH as i32
                || point.y >= DISPLAY_HEIGHT as i32
            {
                continue;
            }
            let index = point.y as usize * DISPLAY_WIDTH + point.x as usize;
            self.framebuffer[index] = color;
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.framebuffer.fill(color);
        Ok(())
    }
}

struct Background {
    background_color: Rgb565,
}

impl Background {
    pub fn new(background_color: Rgb565) -> Self {
        Self { background_color }
    }
}

pub fn get_view_elements(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    match model.view {
        View::Boot => boot_screen(),
        View::HomeScreen => home_screen(model),
        View::ConnectApp => connect_app_screen(model),
        View::LanguageSelection => language_selection_screen(model),
        View::Recording => recording_screen(model),
        View::Statistics => statistics_screen(model),
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
            73,
            model.language.get_str("start_training").to_string(),
            AppColor::PRIMARY,
            AppColor::LIGHT,
            None,
        )),
        Box::new(Button::new(
            ActionId::ConnectApp,
            AppSpacing::MEDIUM,
            2 * AppSpacing::MEDIUM + 73,
            240,
            73,
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
            118,
            AppColor::LIGHT,
            AppIcon::GLOBE.try_into().unwrap(),
        )),
    ];

    Ok(elements)
}

impl AppDrawable for Background {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        display.clear(self.background_color).unwrap();
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
    let mut elements: Vec<Box<dyn AppDrawable>> = vec![
        Box::new(Background::new(AppColor::DARK)),
        Box::new(Button::new(
            ActionId::None,
            320 - 96 - AppSpacing::MEDIUM,
            AppSpacing::MEDIUM,
            96,
            48,
            format!("{:.1}", model.current_reading),
            AppColor::LIGHT,
            AppColor::DARK,
            None,
        )),
    ];

    if model.is_recording {
        elements.push(Box::new(IconButton::new(
            ActionId::Stop,
            320 - 96 - AppSpacing::MEDIUM,
            2 * AppSpacing::MEDIUM + 48,
            96,
            98,
            AppColor::PRIMARY,
            AppIcon::STOP.try_into().unwrap(),
        )));
        let mut i = 0;
        let max = 100;
        model.past_readings.clone().into_iter().for_each(|reading| {
            let max_height = 170 - 2 * AppSpacing::MEDIUM - 2;
            let height =
                ((reading / max as f32) * max_height as f32).clamp(0.0, max_height as f32) as u32;
            elements.push(Box::new(Button::new(
                ActionId::None,
                AppSpacing::MEDIUM + i * 10,
                170 - (AppSpacing::MEDIUM + height - 2),
                AppSpacing::MEDIUM,
                height + 2,
                "".to_string(),
                AppColor::LIGHT,
                AppColor::DARK,
                None,
            )));
            i += 1;
        })
    } else {
        elements.push(Box::new(IconButton::new(
            ActionId::Start,
            320 - 96 - AppSpacing::MEDIUM,
            2 * AppSpacing::MEDIUM + 48,
            96,
            98,
            AppColor::PRIMARY,
            AppIcon::PLAY.try_into().unwrap(),
        )));
    }
    return Ok(elements);
}

fn statistics_screen(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    let elements: Vec<Box<dyn AppDrawable>> = vec![
        Box::new(Background::new(AppColor::DARK)),
        Box::new(Button::new(
            ActionId::None,
            320 - 96 - AppSpacing::MEDIUM,
            AppSpacing::MEDIUM,
            96,
            48,
            format!("0.0 {}", model.language.get_str("kg").to_string()),
            AppColor::LIGHT,
            AppColor::DARK,
            None,
        )),
        Box::new(IconButton::new(
            ActionId::Exit,
            320 - 96 - AppSpacing::MEDIUM,
            2 * AppSpacing::MEDIUM + 48,
            96,
            98,
            AppColor::PRIMARY,
            AppIcon::EXIT.try_into().unwrap(),
        )),
        Box::new(Button::new(
            ActionId::None,
            AppSpacing::MEDIUM,
            0,
            320 - 3 * AppSpacing::MEDIUM - 96,
            73,
            format!(
                "{}\n{}",
                model.language.get_str("max_weight").to_string(),
                format!("{:.1} kg", model.max_weight)
            ),
            AppColor::DARK,
            AppColor::LIGHT,
            None,
        )),
        Box::new(Button::new(
            ActionId::None,
            AppSpacing::MEDIUM,
            AppSpacing::MEDIUM + 73,
            320 - 3 * AppSpacing::MEDIUM - 96,
            73,
            format!(
                "{}\n{}",
                model.language.get_str("max_weight_avg").to_string(),
                format!("{:.1} kg", model.max_weight_avg)
            ),
            AppColor::DARK,
            AppColor::LIGHT,
            None,
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
            304,
            73,
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
        y_pos += AppSpacing::MEDIUM + 73;
    }
    Ok(elements)
}

fn connect_app_screen<'a>(model: &AppViewModel) -> Result<Vec<Box<dyn AppDrawable>>, AppError> {
    const QR_BMP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/qr.bmp"));
    let elements: Vec<Box<dyn AppDrawable>> = vec![
        Box::new(Background::new(AppColor::DARK)),
        Box::new(Button::new(
            ActionId::Exit,
            170 + 2 * AppSpacing::MEDIUM,
            48,
            320 - 3 * AppSpacing::MEDIUM - 170,
            74,
            model.language.get_str("back").to_string(),
            AppColor::PRIMARY,
            AppColor::LIGHT,
            None,
        )),
        Box::new(ClickableImage::new(ActionId::None, 0, 0, 170, 170, QR_BMP)),
    ];
    Ok(elements)
}

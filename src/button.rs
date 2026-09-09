use std::i128::MIN;

use embedded_graphics::{
    geometry::{Point, Size},
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    primitives::{Primitive, PrimitiveStyle, Rectangle, RoundedRectangle},
    text::Text,
    Drawable,
};
use embedded_vintage_fonts::FONT_12X16;

use crate::{
    app_errors::AppResult,
    view_model::{ActionId, AppDrawable, Language},
    views::AppDisplay,
};

pub struct Button {
    id: ActionId,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    button_text: String,
    background_color: Rgb565,
    text_color: Rgb565,
    language: Option<Language>,
}

impl Button {
    pub fn new(
        id: ActionId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        button_text: String,
        background_color: Rgb565,
        text_color: Rgb565,
        language: Option<Language>,
    ) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            button_text,
            background_color,
            text_color,
            language,
        }
    }
}

impl AppDrawable for Button {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        let x = self.x.try_into().unwrap();
        let y = self.y.try_into().unwrap();
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(x, y), Size::new(self.width, self.height)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(self.background_color))
        .draw(display)?;
        let text_style = MonoTextStyle::new(&FONT_12X16, self.text_color);
        let button_height: i32 = self.height.try_into().unwrap();
        let text_margin: i32 = ((button_height - 16) / 2).min(16);
        let center_offset: i32 = 6;
        Text::new(
            &self.button_text,
            Point::new(x + text_margin, y + button_height / 2 + center_offset),
            text_style,
        )
        .draw(display)?;
        Ok(())
    }

    fn eval_touch(&self, x: &u32, y: &u32) -> bool {
        (self.x..(self.x + self.width)).contains(x) && (self.y..(self.y + self.height)).contains(y)
    }

    fn get_id(&self) -> (&ActionId, &Option<Language>) {
        (&self.id, &self.language)
    }
}

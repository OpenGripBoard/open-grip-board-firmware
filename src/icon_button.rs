use embedded_graphics::{
    geometry::{Point, Size},
    image::Image,
    pixelcolor::Rgb565,
    primitives::{Primitive, PrimitiveStyle, Rectangle, RoundedRectangle},
    Drawable,
};
use tinybmp::Bmp;

use crate::{
    app_errors::AppResult,
    view_model::{ActionId, AppDrawable, Language},
    views::AppDisplay,
};

pub struct IconButton {
    id: ActionId,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    background_color: Rgb565,
    bmp_data: [u8; 4854],
}

impl IconButton {
    pub fn new(
        id: ActionId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        background_color: Rgb565,
        bmp_data: [u8; 4854],
    ) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            background_color,
            bmp_data,
        }
    }
}

impl AppDrawable for IconButton {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        let x = self.x.try_into().unwrap();
        let y = self.y.try_into().unwrap();
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(x, y), Size::new(self.width, self.height)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(self.background_color))
        .draw(display)?;
        let icon_size = 40;
        let button_height: i32 = self.height.try_into().unwrap();
        let button_width: i32 = self.width.try_into().unwrap();
        let x_margin: i32 = (button_width - icon_size) / 2;
        let y_margin: i32 = (button_height - icon_size) / 2;
        let bmp = Bmp::from_slice(&self.bmp_data).unwrap();
        Image::new(&bmp, Point::new(x + x_margin, y + y_margin)).draw(display)?;
        Ok(())
    }
    fn eval_touch(&self, x: &u32, y: &u32) -> bool {
        (self.x..(self.x + self.width)).contains(x) && (self.y..(self.y + self.height)).contains(y)
    }
    fn get_id(&self) -> (&ActionId, &Option<Language>) {
        (&self.id, &None)
    }
}

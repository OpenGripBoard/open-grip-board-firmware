use embedded_graphics::{geometry::Point, image::Image, Drawable};
use tinybmp::Bmp;

use crate::{
    app_errors::AppResult,
    view_model::{ActionId, AppDrawable, Language},
    views::AppDisplay,
};

pub struct ClickableImage<'a> {
    id: ActionId,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    bmp_data: &'a [u8],
}

impl<'a> ClickableImage<'a> {
    pub fn new(id: ActionId, x: u32, y: u32, width: u32, height: u32, bmp_data: &'a [u8]) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            bmp_data,
        }
    }
}

impl<'a> AppDrawable for ClickableImage<'a> {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        let (img_width, img_height): (i32, i32) = bmp_dimensions(self.bmp_data).unwrap();
        let x: i32 = self.x.try_into().unwrap();
        let y: i32 = self.y.try_into().unwrap();
        let container_height: i32 = self.height.try_into().unwrap();
        let container_width: i32 = self.width.try_into().unwrap();
        let x_margin: i32 = (container_width - img_width) / 2;
        let y_margin: i32 = (container_height - img_height) / 2;
        let bmp = Bmp::from_slice(self.bmp_data).unwrap();
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

fn bmp_dimensions(data: &[u8]) -> Option<(i32, i32)> {
    if data.len() < 26 || &data[0..2] != b"BM" {
        return None;
    }
    // DIB header:
    // offset 18: width  (i32)
    // offset 22: height (i32)
    let width = i32::from_le_bytes(data[18..22].try_into().ok()?);
    let height = i32::from_le_bytes(data[22..26].try_into().ok()?);
    Some((width, height))
}

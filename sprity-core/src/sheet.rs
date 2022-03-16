use image::RgbaImage;

#[derive(Debug)]
pub struct Sheet {
    pub image: RgbaImage,
    pub cols: u32,
    pub rows: u32,
}

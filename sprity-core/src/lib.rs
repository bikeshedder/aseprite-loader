mod loader;
mod meta;

use image::RgbaImage;
pub use loader::{LoadDirError, Loader};
pub use meta::SpriteMeta;

#[derive(Debug)]
pub struct SpriteSheet {
    pub name: String,
    pub tags: Vec<SpriteSheetTags>,
    pub layers: Vec<SpriteSheetLayer>,
    pub sprites: Vec<RgbaImage>,
}

#[derive(Debug)]
pub struct SpriteSheetLayer {
    pub name: String,
}

#[derive(Debug)]
pub struct SpriteSheetTags {
    pub name: String,
}

pub trait Sprite {
    fn name(&self) -> &str;
}

pub trait Tag {
    fn name(&self) -> &str;
}

pub trait Layer {
    fn name(&self) -> &str;
}

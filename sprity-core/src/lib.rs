mod color;
mod loader;
mod meta;

pub use color::Color;
pub use loader::{
    Frame, ImageLoader, ListDirError, LoadDirError, LoadImageError, LoadSpriteError, Loader,
    SpriteLoader,
};
pub use meta::{DynamicSpriteSheetMeta, SpriteSheetMeta};

pub trait Sprite {
    fn name(&self) -> &str;
}

pub trait SpriteWithMeta: Sprite {
    fn meta() -> SpriteSheetMeta;
}

pub trait Tag {
    fn name(&self) -> &str;
}

pub trait Layer {
    fn name(&self) -> &str;
}

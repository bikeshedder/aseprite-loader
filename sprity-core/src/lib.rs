mod color;
mod loader;
mod meta;
mod palette;
mod sheet;

pub use color::Color;
pub use loader::{
    ImageLoader, ListDirError, LoadDirError, LoadImageError, LoadSpriteError, Loader, SpriteLoader,
};
pub use meta::{
    DynamicSpriteSheetMeta, LayerIterator, SpriteSheetMeta, StaticSpriteSheetMeta, TagIterator,
};
pub use palette::Palette;
pub use sheet::Sheet;

pub trait Sprite {
    fn name(&self) -> &str;
}

pub trait Tag {
    fn name(&self) -> &str;
}

pub trait Layer {
    fn name(&self) -> &str;
}

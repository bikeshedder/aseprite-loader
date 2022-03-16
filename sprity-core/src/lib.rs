mod loader;
mod meta;
mod sheet;

pub use loader::{LoadDirError, Loader};
pub use meta::{
    DynamicSpriteSheetMeta, LayerIterator, SpriteSheetMeta, StaticSpriteSheetMeta, TagIterator,
};
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

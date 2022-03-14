mod loader;
mod meta;

pub use loader::{LoadDirError, Loader};
pub use meta::SpriteMeta;

pub trait Sprite {
    fn name(&self) -> &str;
}

pub trait Tag {
    fn name(&self) -> &str;
}

pub trait Layer {
    fn name(&self) -> &str;
}

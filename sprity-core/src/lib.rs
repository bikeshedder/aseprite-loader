mod loader;
mod meta;

pub use loader::{LoadDirError, Loader};
pub use meta::SpriteMeta;

pub trait Sprite {
    fn name(&self) -> &str;
}

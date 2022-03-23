use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::meta::{DynamicSpriteSheetMeta, SpriteSheetMeta};

pub trait Loader {
    /// Load all sprites from a given directory that this
    /// loader supports and return a vector of sprite sheet
    /// metadata.
    fn load_dir_meta(
        &self,
        dir: &dyn AsRef<Path>,
    ) -> Result<Vec<DynamicSpriteSheetMeta>, LoadDirError>;
    /// List all file names in a given directory that match
    /// the given list of sprite sheet metadata. The list
    /// of filenames is in the same order as the given list
    /// of metadata.
    fn list_dir(
        &self,
        dir: &dyn AsRef<Path>,
        meta: &[&dyn SpriteSheetMeta],
    ) -> Result<Vec<PathBuf>, ListDirError>;
    /// Load a sprite given its data and metadata.
    fn load_sprite<'a>(
        &self,
        data: &'a [u8],
        meta: &dyn SpriteSheetMeta,
    ) -> Result<Box<dyn SpriteLoader + 'a>, LoadSpriteError>;
}

#[derive(Error, Debug)]
pub enum LoadDirError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("parsing of `{filename}` failed: {message}")]
    Parse { filename: String, message: String },
    #[error("directory does not contain any {ext} files: {dir}")]
    EmptyDirectory { ext: &'static str, dir: String },
    #[error("unknown error occured")]
    Other(#[from] Box<dyn std::error::Error>),
}

#[derive(Error, Debug)]
pub enum ListDirError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("missing sprite: {0}")]
    MissingSprite(String),
}

pub trait SpriteLoader {
    /// Get size of the sprite (width, height)
    fn size(&self) -> (u16, u16);
    /// This iterator should be used to load the actual images
    /// needed for this sprite. The order of the images is important
    /// as they are used by the return value of the `frames` method.
    fn images(&self) -> Box<dyn Iterator<Item = Box<dyn ImageLoader + '_>> + '_>;
    /// Get the image indices for a given tag and layer
    fn frames(&self, tag: usize, layer: usize) -> Vec<usize>;
    /// Get frame durations for a given tag
    fn durations(&self, tag: usize) -> Vec<u16>;
}

#[derive(Error, Debug)]
pub enum LoadSpriteError {
    #[error("parsing failed {message}")]
    Parse { message: String },
}

pub trait ImageLoader {
    /// Load the image into the provided target buffer and return the
    /// slice of data that was actually loaded.
    fn load<'a>(&self, target: &'a mut [u8]) -> Result<&'a [u8], LoadImageError>;
    fn size(&self) -> (u16, u16);
    fn origin(&self) -> (i16, i16);
    /// Get the image size in bytes: width * height * 4
    fn bytes(&self) -> usize {
        let size = self.size();
        (size.0 * size.1 * 4).into()
    }
}

#[derive(Error, Debug)]
pub enum LoadImageError {
    #[error("target buffer too small")]
    TargetBufferTooSmall,
    #[error("missing palette")]
    MissingPalette,
    #[error("unsupported color depth")]
    UnsupportedColorDepth,
    #[error("decompression failed")]
    DecompressError,
    #[error("invalid image data")]
    InvalidImageData,
}

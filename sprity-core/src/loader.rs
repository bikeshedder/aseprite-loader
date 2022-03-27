use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

pub trait Loader {
    /// List all file names in a given directory that match
    /// the given list of sprite sheet metadata. The list
    /// of filenames is in the same order as the given list
    /// of metadata.
    fn list_dir(&self, dir: &dyn AsRef<Path>) -> Result<Vec<(String, PathBuf)>, ListDirError>;
    /// Load a sprite given its data and metadata.
    fn load_sprite<'a>(
        &self,
        data: &'a [u8],
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
    #[error("directory listing failed")]
    ListDir(#[from] ListDirError),
    #[error("sprite loading failed")]
    LoadSprite(#[from] LoadSpriteError),
}

#[derive(Error, Debug)]
pub enum ListDirError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("missing sprite: {0}")]
    MissingSprite(String),
}

#[derive(Debug, Copy, Clone)]
pub struct Frame {
    pub duration: u16,
    pub origin: (i16, i16),
    pub image_index: usize,
}

pub trait SpriteLoader {
    /// Get size of the sprite (width, height)
    fn size(&self) -> (u16, u16);
    /// Get tag names
    fn tags(&self) -> &[String];
    /// Get layer names
    fn layers(&self) -> &[String];
    /// Get the image indices for a given tag and layer
    fn frames(&self, tag: usize, layer: usize) -> &[Frame];
    /// Load the image into the provided target buffer and return the
    /// slice of data that was actually loaded.
    fn load_image<'a>(
        &self,
        index: usize,
        target: &'a mut [u8],
    ) -> Result<&'a [u8], LoadImageError>;
}

#[derive(Error, Debug)]
pub enum LoadSpriteError {
    #[error("parsing failed {message}")]
    Parse { message: String },
    #[error("missing tag: {0}")]
    MissingTag(String),
    #[error("missing layer: {0}")]
    MissingLayer(String),
    #[error("frame index out of range: {0}")]
    FrameIndexOutOfRange(usize),
}

pub trait ImageLoader {
    /// Load the image into the provided target buffer and return the
    /// slice of data that was actually loaded.
    fn load<'a>(&self, target: &'a mut [u8]) -> Result<&'a [u8], LoadImageError>;
    fn size(&self) -> (u16, u16);
    /// Get the image size in bytes: width * height * 4
    fn bytes(&self) -> usize {
        let size = self.size();
        (size.0 * size.1 * 4).into()
    }
}

#[derive(Error, Debug)]
pub enum LoadImageError {
    #[error("invalid image index")]
    InvalidImageIndex,
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

/* FIXME this code used to live in sprity-aseprite but is actually a core feature

       // The loaded file must contain all the required tags and layers in
       // the the correct order. Extra layers and tags are ignored.
       let file_tags: Vec<_> = file.tags().collect();
       let file_tags_map: HashMap<_, _> = file
           .tags()
           .enumerate()
           .map(|(index, tag)| (tag.name.to_upper_camel_case(), index))
           .collect();

       let tag_mapping: Vec<_> = (0..meta.tag_count())
           .map(|index| meta.tag(index))
           .map(|name| {
               file_tags_map
                   .get(name)
                   .ok_or_else(|| LoadSpriteError::MissingTag(name.to_owned()))
                   .cloned()
           })
           .try_collect()?;

       let file_layers: HashMap<_, _> = file
           .normal_layers()
           .map(|(index, layer)| (layer.name.to_upper_camel_case(), index))
           .collect();

       let layer_count = meta.layer_count();
       let layer_mapping: HashMap<_, _> = (0..layer_count)
           .map(|meta_index| (meta_index, meta.layer(meta_index)))
           .map(|(meta_index, name)| {
               file_layers
                   .get(name)
                   .map(|file_index| (meta_index, *file_index))
                   .ok_or_else(|| LoadSpriteError::MissingLayer(name.to_owned()))
           })
           .try_collect()?;

       let mut cels: Vec<Option<&CelChunk>> = vec![None; file.frames.len() * layer_count];
       for (frame, cel) in file.cels() {
           if let Some(meta_layer_index) = layer_mapping.get(&cel.layer_index.into()) {
               cels[frame * layer_count + meta_layer_index] = Some(cel);
           }
       }




        meta.iter()
            .map(|&meta| {
                available_sheets
                    .get(meta.name())
                    .cloned()
                    .ok_or_else(|| ListDirError::MissingSprite(meta.name().to_owned()))
            })
            .try_collect()
*/

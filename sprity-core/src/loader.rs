use std::{io, path::Path};

use thiserror::Error;

use crate::{
    meta::{DynamicSpriteSheetMeta, SpriteSheetMeta},
    sheet::Sheet,
};

pub trait Loader {
    fn load_dir_meta(
        &self,
        dir: &dyn AsRef<Path>,
    ) -> Result<Vec<DynamicSpriteSheetMeta>, LoadDirError>;
    fn load_dir(
        &self,
        dir: &dyn AsRef<Path>,
        meta: &[&dyn SpriteSheetMeta],
    ) -> Result<Vec<Sheet>, LoadDirError>;
}

#[derive(Error, Debug)]
pub enum LoadDirError {
    #[error("an IO error occured")]
    Io(#[from] io::Error),
    #[error("parsing of `{filename}` failed: {message}")]
    Parse { filename: String, message: String },
    #[error("directory does not contain any {ext} files: {dir}")]
    EmptyDirectory { ext: &'static str, dir: String },
    #[error("missing sprite: {0}")]
    MissingSprite(String),
    #[error("unknown error occured")]
    Other(#[from] Box<dyn std::error::Error>),
}

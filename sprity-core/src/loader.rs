use std::{io, path::Path};

use thiserror::Error;

use crate::meta::SpriteMeta;

pub trait Loader {
    fn load_dir_meta(&self, dir: &dyn AsRef<Path>) -> Result<Vec<SpriteMeta>, LoadDirError>;
}

#[derive(Error, Debug)]
pub enum LoadDirError {
    #[error("an IO error occured")]
    Io(#[from] io::Error),
    #[error("parsing of `{filename}` failed: {message}")]
    Parse { filename: String, message: String },
    #[error("directory does not contain any {ext} files: {dir}")]
    EmptyDirectory { ext: &'static str, dir: String },
    #[error("unknown error occured")]
    Other(#[from] Box<dyn std::error::Error>),
}

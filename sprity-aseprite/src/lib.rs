// FIXME this should be removed once the implementation is complete
#![allow(unused)]

use std::path::Path;

use sprity_core::{LoadDirError, SpriteMeta};

use crate::{binary::chunk::Chunk, json::Layer};

pub mod binary;
pub mod json;

/*
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct AsepriteFile {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub layers: Vec<String>,
}

impl AsepriteFile {
    pub fn load(name: &str, path: &PathBuf) -> Self {
        let content = std::fs::read(path).unwrap();
        let aseprite = sprity_aseprite::binary::file::parse_file(&content).unwrap();
        // FIXME
        Self {
            name: name.to_upper_camel_case(),
            path: path.to_owned(),
            tags: vec![],
            layers: vec![],
        }
    }
}

*/

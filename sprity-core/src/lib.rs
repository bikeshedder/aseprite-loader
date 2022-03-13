use std::path::PathBuf;

use heck::ToUpperCamelCase;

pub trait Sprite {
    fn name(&self) -> &'static str;
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct AsepriteFile {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub layers: Vec<String>,
}

impl AsepriteFile {
    pub fn load(name: &str, path: &PathBuf) -> Self {
        let aseprite = aseprite_reader::Aseprite::from_path(path).unwrap();
        Self {
            name: name.to_upper_camel_case(),
            path: path.to_owned(),
            tags: vec![],
            layers: vec![],
        }
    }
}

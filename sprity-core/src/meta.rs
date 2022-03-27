use heck::ToUpperCamelCase;

use crate::{LoadSpriteError, SpriteLoader};

pub struct SpriteSheetMeta {
    pub name: &'static str,
    pub tags: &'static [&'static str],
    pub layers: &'static [&'static str],
}

pub struct DynamicSpriteSheetMeta {
    pub name: String,
    pub tags: Vec<String>,
    pub layers: Vec<String>,
}

impl DynamicSpriteSheetMeta {
    pub fn from_loader(name: String, loader: &dyn SpriteLoader) -> Result<Self, LoadSpriteError> {
        Ok(Self {
            name,
            tags: loader
                .tags()
                .iter()
                .map(|s| s.to_upper_camel_case())
                .collect(),
            layers: loader
                .layers()
                .iter()
                .map(|s| s.to_upper_camel_case())
                .collect(),
        })
    }
}

use std::{fs::read_dir, path::Path};

use itertools::Itertools;
use sprity_core::{LoadDirError, SpriteMeta};

use super::{chunk::Chunk, file::parse_file};

static ASEPRITE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

pub struct BinaryLoader {}

impl BinaryLoader {
    fn load_sprite_meta(
        &self,
        name: &str,
        path: impl AsRef<Path>,
    ) -> Result<SpriteMeta, LoadDirError> {
        let content = std::fs::read(path)?;
        let aseprite = parse_file(&content).map_err(|e| LoadDirError::Parse {
            filename: name.to_owned(),
            message: e.to_string(),
        })?;
        let layers = aseprite
            .frames
            .iter()
            .flat_map(|frame| {
                frame.chunks.iter().filter_map(|chunk| match chunk {
                    Chunk::Layer(chunk) => Some(chunk),
                    _ => None,
                })
            })
            .map(|chunk| chunk.name.to_owned())
            .collect::<Vec<_>>();

        let tags = aseprite
            .frames
            .iter()
            .flat_map(|frame| {
                frame.chunks.iter().filter_map(|chunk| match chunk {
                    Chunk::Tags(chunk) => Some(&chunk.tags),
                    _ => None,
                })
            })
            .flat_map(|tags| tags.iter().map(|tag| tag.name.to_owned()))
            .collect::<Vec<_>>();

        Ok(SpriteMeta {
            name: name.to_owned(),
            layers,
            tags,
        })
    }
}

impl sprity_core::Loader for BinaryLoader {
    fn load_dir_meta(&self, dir: &dyn AsRef<Path>) -> Result<Vec<SpriteMeta>, LoadDirError> {
        let entries: Vec<_> = read_dir(dir)?.try_collect()?;
        entries
            .iter()
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_stem()?;
                let ext = path.extension()?;
                if ASEPRITE_EXTENSIONS.contains(&ext.to_str()?.to_lowercase().as_ref()) {
                    Some(self.load_sprite_meta(name.to_str()?, &path))
                } else {
                    None
                }
            })
            .try_collect()
    }
}

use std::{
    collections::HashMap,
    fs::read_dir,
    path::{Path, PathBuf},
};

use flate2::Decompress;
use heck::ToUpperCamelCase;
use itertools::Itertools;
use sprity_core::{
    DynamicSpriteSheetMeta, ImageLoader, ListDirError, LoadDirError, LoadImageError,
    LoadSpriteError, SpriteLoader, SpriteSheetMeta,
};

use super::{
    chunks::cel::ImageCel,
    color_depth::ColorDepth,
    file::{parse_file, File},
    palette::Palette,
};

static ASEPRITE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

pub struct BinaryLoader {}

impl BinaryLoader {}

impl sprity_core::Loader for BinaryLoader {
    fn load_dir_meta(
        &self,
        dir: &dyn AsRef<Path>,
    ) -> Result<Vec<DynamicSpriteSheetMeta>, LoadDirError> {
        let entries: Vec<_> = read_dir(dir)?.try_collect()?;
        entries
            .iter()
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_stem()?.to_str()?.to_upper_camel_case();
                let ext = path.extension()?;
                if ASEPRITE_EXTENSIONS.contains(&ext.to_str()?.to_lowercase().as_ref()) {
                    Some(load_sprite_meta(&name, &path))
                } else {
                    None
                }
            })
            .try_collect()
    }

    fn list_dir(
        &self,
        dir: &dyn AsRef<Path>,
        meta: &[&dyn SpriteSheetMeta],
    ) -> Result<Vec<PathBuf>, sprity_core::ListDirError> {
        let available_sheets: HashMap<_, _> = read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_stem()?.to_str()?.to_upper_camel_case();
                let ext = path.extension()?;
                if ASEPRITE_EXTENSIONS.contains(&ext.to_str()?.to_lowercase().as_ref()) {
                    Some((name, path))
                } else {
                    None
                }
            })
            .collect();
        meta.iter()
            .map(|&meta| {
                available_sheets
                    .get(meta.name())
                    .cloned()
                    .ok_or_else(|| ListDirError::MissingSprite(meta.name().to_owned()))
            })
            .try_collect()
    }

    fn load_sprite<'a>(
        &self,
        data: &'a [u8],
        meta: &dyn SpriteSheetMeta,
    ) -> Result<Box<dyn SpriteLoader + 'a>, LoadSpriteError> {
        Ok(Box::new(AsepriteSpriteLoader::load(data, meta)?))
    }
}

fn load_sprite_meta(
    name: &str,
    path: &dyn AsRef<Path>,
) -> Result<DynamicSpriteSheetMeta, LoadDirError> {
    let content = std::fs::read(path)?;
    let file = parse_file(&content).map_err(|e| LoadDirError::Parse {
        filename: name.to_owned(),
        message: e.to_string(),
    });
    file.map(|file| DynamicSpriteSheetMeta {
        name: name.to_owned(),
        layers: file
            .normal_layers()
            .map(|layer| layer.name.to_owned())
            .collect(),
        tags: file.tags().map(|tag| tag.name.to_owned()).collect(),
    })
}

struct AsepriteSpriteLoader<'a> {
    file: File<'a>,
}

impl AsepriteSpriteLoader<'_> {
    fn load<'a>(
        data: &'a [u8],
        meta: &dyn SpriteSheetMeta,
    ) -> Result<AsepriteSpriteLoader<'a>, LoadSpriteError> {
        let file = parse_file(data).map_err(|e| LoadSpriteError::Parse {
            message: e.to_string(),
        })?;
        Ok(AsepriteSpriteLoader { file })
    }
}

impl<'a> SpriteLoader for AsepriteSpriteLoader<'a> {
    fn size(&self) -> (u16, u16) {
        (self.file.header.width, self.file.header.height)
    }
    fn images(&self) -> Box<dyn Iterator<Item = Box<dyn ImageLoader + '_>> + '_> {
        Box::new(self.file.image_cels().map(|(frame, cel)| {
            Box::new(AsepriteImageLoader {
                file: &self.file,
                frame,
                cel,
            }) as Box<dyn ImageLoader + '_>
        }))
    }
    fn frames(&self, tag: usize, layer: usize) -> Vec<usize> {
        todo!()
    }
    fn durations(&self, tag: usize) -> Vec<u16> {
        todo!()
    }
}

struct AsepriteImageLoader<'a> {
    file: &'a File<'a>,
    frame: usize,
    cel: ImageCel<'a>,
}

impl<'a> ImageLoader for AsepriteImageLoader<'a> {
    fn load<'b>(&self, target: &'b mut [u8]) -> Result<&'b [u8], LoadImageError> {
        let target_size = usize::from(self.cel.width * self.cel.height * 4);
        if target.len() < target_size {
            return Err(LoadImageError::TargetBufferTooSmall);
        }
        let target = &mut target[..target_size];
        match (self.file.header.color_depth, self.cel.compressed) {
            (ColorDepth::Rgba, false) => target.copy_from_slice(self.cel.data),
            (ColorDepth::Rgba, true) => decompress(self.cel.data, target)?,
            (ColorDepth::Grayscale, false) => {
                grayscale_to_rgba(self.cel.data, target)?;
            }
            (ColorDepth::Grayscale, true) => {
                let mut buf = vec![0u8; (self.cel.width * self.cel.height * 2).into()];
                decompress(self.cel.data, &mut buf)?;
                grayscale_to_rgba(&buf, target)?;
            }
            (ColorDepth::Indexed, false) => {
                indexed_to_rgba(
                    self.cel.data,
                    self.file
                        .palette
                        .as_ref()
                        .ok_or(LoadImageError::MissingPalette)?,
                    target,
                )?;
            }
            (ColorDepth::Indexed, true) => {
                let mut buf = vec![0u8; (self.cel.width * self.cel.height).into()];
                decompress(self.cel.data, &mut buf)?;
                indexed_to_rgba(
                    &buf,
                    self.file
                        .palette
                        .as_ref()
                        .ok_or(LoadImageError::MissingPalette)?,
                    target,
                )?;
            }
            (ColorDepth::Unknown(_), _) => return Err(LoadImageError::UnsupportedColorDepth),
        }
        Ok(target)
    }
    fn size(&self) -> (u16, u16) {
        (self.cel.width, self.cel.height)
    }
    fn origin(&self) -> (i16, i16) {
        (self.cel.x, self.cel.y)
    }
}

fn decompress(data: &[u8], target: &mut [u8]) -> Result<(), LoadImageError> {
    let mut decompressor = Decompress::new(true);
    match decompressor.decompress(data, target, flate2::FlushDecompress::Finish) {
        Ok(flate2::Status::Ok | flate2::Status::BufError) => Err(LoadImageError::DecompressError),
        Ok(flate2::Status::StreamEnd) => Ok(()),
        Err(_) => Err(LoadImageError::DecompressError),
    }
}

fn grayscale_to_rgba(source: &[u8], target: &mut [u8]) -> Result<(), LoadImageError> {
    if target.len() != source.len() * 2 {
        return Err(LoadImageError::InvalidImageData);
    }
    for (i, chunk) in source.chunks(2).enumerate() {
        target[i * 4] = chunk[0];
        target[i * 4 + 1] = chunk[0];
        target[i * 4 + 2] = chunk[0];
        target[i * 4 + 3] = chunk[1];
    }
    Ok(())
}

fn indexed_to_rgba(
    source: &[u8],
    palette: &Palette,
    target: &mut [u8],
) -> Result<(), LoadImageError> {
    if target.len() != source.len() * 4 {
        return Err(LoadImageError::InvalidImageData);
    }
    for (i, px) in source.iter().enumerate() {
        let color = palette.colors[*px as usize];
        target[i * 4] = color.red;
        target[i * 4 + 1] = color.green;
        target[i * 4 + 2] = color.blue;
        target[i * 4 + 3] = color.alpha;
    }
    Ok(())
}

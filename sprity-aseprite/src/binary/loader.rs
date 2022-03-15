use std::{borrow::Cow, fs::read_dir, path::Path};

use flate2::Decompress;
use image::{
    buffer::ConvertBuffer, GenericImage, GrayAlphaImage, ImageBuffer, ImageError, LumaA, Rgba,
    RgbaImage,
};
use itertools::Itertools;
use sprity_core::{LoadDirError, SpriteMeta, SpriteSheet, SpriteSheetLayer, SpriteSheetTags};
use thiserror::Error;

use crate::binary::chunks::cel::CelContent;

use super::{chunks::cel::CelChunk, color_depth::ColorDepth, file::parse_file, header::Header};

static ASEPRITE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

pub struct BinaryLoader {}

impl BinaryLoader {
    fn load_sprite_meta(
        &self,
        name: &str,
        path: impl AsRef<Path>,
    ) -> Result<SpriteMeta, LoadDirError> {
        let content = std::fs::read(path)?;
        let file = parse_file(&content).map_err(|e| LoadDirError::Parse {
            filename: name.to_owned(),
            message: e.to_string(),
        });
        file.map(|file| SpriteMeta {
            name: name.to_owned(),
            layers: file.layers().map(|layer| layer.name.to_owned()).collect(),
            tags: file.tags().map(|tag| tag.name.to_owned()).collect(),
        })
    }
    fn load_sprite(&self, name: &str, path: impl AsRef<Path>) -> Result<SpriteSheet, LoadDirError> {
        let content = std::fs::read(path)?;
        let file = parse_file(&content).map_err(|e| LoadDirError::Parse {
            filename: name.to_owned(),
            message: e.to_string(),
        })?;
        let mut sprites = Vec::new();
        for frame in file.frames.iter() {
            for cel in frame.cels() {
                if let Some(image) = read_image(&file.header, cel).unwrap() {
                    sprites.push(image);
                }
            }
        }
        Ok(SpriteSheet {
            name: name.to_owned(),
            tags: file
                .tags()
                .map(|tag| SpriteSheetTags {
                    name: tag.name.to_owned(),
                })
                .collect(),
            layers: file
                .tags()
                .map(|layer| SpriteSheetLayer {
                    name: layer.name.to_owned(),
                })
                .collect(),
            sprites,
        })
    }
}

#[derive(Debug, Error)]
enum DecompressError {
    #[error("decompression failed")]
    FlateStatus(flate2::Status),
    #[error("decompression failed")]
    FlateError(#[from] flate2::DecompressError),
}

fn decompress(width: u16, height: u16, bpp: u16, data: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let width: usize = width.into();
    let height: usize = height.into();
    let bytes_per_pixel: usize = (bpp / 8).into();
    let mut decompressor = Decompress::new(true);
    let mut output = vec![0; width * height * bytes_per_pixel];
    let status = decompressor.decompress(data, &mut output, flate2::FlushDecompress::Finish)?;
    match status {
        flate2::Status::Ok | flate2::Status::BufError => Err(DecompressError::FlateStatus(status)),
        flate2::Status::StreamEnd => Ok(output),
    }
}

#[derive(Error, Debug)]
enum ReadImageError {
    #[error("Decompression failed")]
    Decompress(#[from] DecompressError),
    #[error("Incomplete image")]
    IncompleteImage,
    #[error("Unsupported color depth")]
    UnsupportedColorDepth(ColorDepth),
    #[error("Negative cel X value")]
    NegativeCelX(i16),
    #[error("Negative cel Y value")]
    NegativeCelY(i16),
    #[error("Image error")]
    ImageError(#[from] ImageError),
}

struct ImageData<'a> {
    width: u16,
    height: u16,
    data: Cow<'a, [u8]>,
}

fn read_image(header: &Header, cel: &CelChunk) -> Result<Option<RgbaImage>, ReadImageError> {
    let image_data = match cel.content {
        CelContent::RawImageData {
            width,
            height,
            data,
        } => ImageData {
            width,
            height,
            data: Cow::Borrowed(data),
        },
        CelContent::CompressedImage {
            width,
            height,
            data,
        } => ImageData {
            width,
            height,
            data: Cow::Owned(decompress(width, height, header.color_depth.bpp(), data)?),
        },
        _ => return Ok(None),
    };
    let x: u32 = cel
        .x
        .try_into()
        .map_err(|_| ReadImageError::NegativeCelX(cel.x))?;
    let y: u32 = cel
        .y
        .try_into()
        .map_err(|_| ReadImageError::NegativeCelY(cel.y))?;
    let image: RgbaImage = match header.color_depth {
        ColorDepth::Grayscale => {
            let mut image = GrayAlphaImage::new(header.width.into(), header.height.into());
            image.copy_from(
                &ImageBuffer::<LumaA<u8>, _>::from_raw(
                    image_data.width.into(),
                    image_data.height.into(),
                    image_data.data,
                )
                .ok_or(ReadImageError::IncompleteImage)?,
                x,
                y,
            )?;
            image.convert()
        }
        ColorDepth::Rgba => {
            let mut image = RgbaImage::new(header.width.into(), header.height.into());
            image.copy_from(
                &ImageBuffer::<Rgba<u8>, _>::from_raw(
                    image_data.width.into(),
                    image_data.height.into(),
                    image_data.data,
                )
                .ok_or(ReadImageError::IncompleteImage)?,
                x,
                y,
            )?;
            image
        }
        color_depth => return Err(ReadImageError::UnsupportedColorDepth(color_depth)),
    };
    Ok(Some(image))
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
    fn load_dir(
        &self,
        dir: &dyn AsRef<Path>,
    ) -> Result<Vec<sprity_core::SpriteSheet>, LoadDirError> {
        let entries: Vec<_> = read_dir(dir)?.try_collect()?;
        entries
            .iter()
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_stem()?;
                let ext = path.extension()?;
                if ASEPRITE_EXTENSIONS.contains(&ext.to_str()?.to_lowercase().as_ref()) {
                    Some(self.load_sprite(name.to_str()?, &path))
                } else {
                    None
                }
            })
            .try_collect()
    }
}

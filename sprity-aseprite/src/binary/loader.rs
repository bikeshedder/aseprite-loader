use std::{borrow::Cow, collections::HashMap, fs::read_dir, path::Path};

use flate2::Decompress;
use heck::ToUpperCamelCase;
use image::{
    buffer::ConvertBuffer, GenericImage, GrayAlphaImage, ImageBuffer, ImageError, LumaA, Rgba,
    RgbaImage, SubImage,
};
use itertools::Itertools;
use sprity_core::{DynamicSpriteSheetMeta, LoadDirError, Sheet, SpriteSheetMeta};
use thiserror::Error;

use crate::binary::chunks::cel::CelContent;

use super::{chunks::cel::CelChunk, color_depth::ColorDepth, file::parse_file, header::Header};

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
    fn load_dir(
        &self,
        dir: &dyn AsRef<Path>,
        sheets: &[&dyn SpriteSheetMeta],
    ) -> Result<Vec<Sheet>, LoadDirError> {
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
        let sheets_to_load: Vec<_> = sheets
            .iter()
            .map(|&meta| {
                available_sheets
                    .get(meta.name())
                    .map(|path| (path, meta))
                    .ok_or_else(|| LoadDirError::MissingSprite(meta.name().to_owned()))
            })
            .try_collect()?;
        sheets_to_load
            .into_iter()
            .map(|(path, meta)| load_sheet(path, meta))
            .try_collect()
    }
}

fn load_sprite_meta(
    name: &str,
    path: impl AsRef<Path>,
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

#[derive(Debug, Error)]
enum DecompressError {
    #[error("decompression failed")]
    FlateStatus(flate2::Status),
    #[error("decompression failed")]
    FlateError(#[from] flate2::DecompressError),
}

fn decompress(
    width: u16,
    height: u16,
    pixel_size: usize,
    data: &[u8],
) -> Result<Vec<u8>, DecompressError> {
    let width: usize = width.into();
    let height: usize = height.into();
    let mut decompressor = Decompress::new(true);
    let mut output = vec![0; width * height * pixel_size];
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

fn read_image(
    header: &Header,
    cel: &CelChunk,
    target: &mut SubImage<&mut RgbaImage>,
) -> Result<bool, ReadImageError> {
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
            data: Cow::Owned(decompress(
                width,
                height,
                header
                    .color_depth
                    .pixel_size()
                    .ok_or(ReadImageError::UnsupportedColorDepth(header.color_depth))?,
                data,
            )?),
        },
        _ => return Ok(false),
    };
    let x: u32 = cel
        .x
        .try_into()
        .map_err(|_| ReadImageError::NegativeCelX(cel.x))?;
    let y: u32 = cel
        .y
        .try_into()
        .map_err(|_| ReadImageError::NegativeCelY(cel.y))?;
    match header.color_depth {
        ColorDepth::Grayscale => {
            let mut image = GrayAlphaImage::new(header.width.into(), header.height.into());
            target.copy_from(
                &ImageBuffer::<LumaA<u8>, _>::from_raw(
                    image_data.width.into(),
                    image_data.height.into(),
                    image_data.data,
                )
                .ok_or(ReadImageError::IncompleteImage)?
                .convert(),
                x,
                y,
            )?;
        }
        ColorDepth::Rgba => {
            let mut image = RgbaImage::new(header.width.into(), header.height.into());
            target.copy_from(
                &ImageBuffer::<Rgba<u8>, _>::from_raw(
                    image_data.width.into(),
                    image_data.height.into(),
                    image_data.data,
                )
                .ok_or(ReadImageError::IncompleteImage)?,
                x,
                y,
            )?;
        }
        color_depth => return Err(ReadImageError::UnsupportedColorDepth(color_depth)),
    };
    Ok(true)
}

fn load_sheet(
    path: &dyn AsRef<Path>,
    meta: &dyn sprity_core::SpriteSheetMeta,
) -> Result<Sheet, LoadDirError> {
    let content = std::fs::read(path)?;
    let file = parse_file(&content).map_err(|e| LoadDirError::Parse {
        filename: meta.name().to_owned(),
        message: e.to_string(),
    })?;
    let sprite_width: u32 = file.header.width.into();
    let sprite_height: u32 = file.header.height.into();
    let tags: HashMap<String, &super::chunks::tags::Tag> = file
        .tags()
        .map(|tag| (tag.name.to_upper_camel_case(), tag))
        .collect();
    // FIXME add support for layer blend mode, opacity and index reordering
    let layers: HashMap<&str, (usize, &super::chunks::layer::LayerChunk)> = file
        .normal_layers()
        .enumerate()
        .map(|(index, layer)| (layer.name, (index, layer)))
        .collect();
    let mut indices: Vec<Option<(usize, usize)>> = vec![None; tags.len() * layers.len()];
    let image_count: u32 = file
        .image_cels()
        .count()
        .try_into()
        .expect("More than 2^32 image cels counted");
    // XXX replace this by usize::log2 once it hits stable rust
    let cols = (1..).find(|cols| cols * cols >= image_count).unwrap();
    // XXX replace this by usize::div_ceil once it hits stable rust
    let rows = (image_count + cols - 1) / cols;
    let mut image = RgbaImage::new(sprite_width * cols, sprite_height * rows);
    let mut index = 0u32;
    for (frame_index, frame) in file.frames.iter().enumerate() {
        for cel in frame.cels() {
            let x = index % cols;
            let y = index / cols;
            if read_image(
                &file.header,
                cel,
                &mut image.sub_image(
                    x * sprite_width,
                    y * sprite_height,
                    sprite_width,
                    sprite_height,
                ),
            )
            .map_err(|e| LoadDirError::Other(Box::new(e)))?
            {
                index += 1;
            }
        }
    }
    Ok(Sheet { cols, rows, image })
}

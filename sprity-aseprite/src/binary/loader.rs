use std::{
    collections::HashMap,
    fs::read_dir,
    path::{Path, PathBuf},
};

use flate2::Decompress;
use heck::ToUpperCamelCase;
use sprity_core::{Frame, LoadImageError, LoadSpriteError, SpriteLoader};

use super::{
    chunks::{cel::CelContent, layer::LayerType},
    color_depth::ColorDepth,
    file::{parse_file, File},
    image::Image,
    palette::Palette,
};

static ASEPRITE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

pub struct BinaryLoader {}

impl BinaryLoader {}

impl sprity_core::Loader for BinaryLoader {
    fn list_dir(
        &self,
        dir: &dyn AsRef<Path>,
    ) -> Result<Vec<(String, PathBuf)>, sprity_core::ListDirError> {
        Ok(read_dir(dir)?
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
            .collect())
    }
    fn load_sprite<'a>(
        &self,
        data: &'a [u8],
    ) -> Result<Box<dyn SpriteLoader + 'a>, LoadSpriteError> {
        Ok(Box::new(AsepriteSpriteLoader::load(data)?))
    }
}

struct AsepriteSpriteLoader<'a> {
    file: File<'a>,
    tags: Vec<String>,
    layers: Vec<String>,
    // This vector maps (tag_index * layer_index + layer_index) to a
    // list of durations, origins and image indices.
    frames: Vec<Vec<Frame>>,
    images: Vec<Image<'a>>,
}

impl AsepriteSpriteLoader<'_> {
    fn load(data: &[u8]) -> Result<AsepriteSpriteLoader, LoadSpriteError> {
        let file = parse_file(data).map_err(|e| LoadSpriteError::Parse {
            message: e.to_string(),
        })?;
        let tags: Vec<_> = file
            .tags
            .iter()
            .map(|tag| tag.name.to_upper_camel_case())
            .collect();
        let layers: Vec<_> = file
            .layers
            .iter()
            .filter_map(|layer| {
                if layer.layer_type == LayerType::Normal {
                    Some(layer.name.to_upper_camel_case())
                } else {
                    None
                }
            })
            .collect();
        // Map between (frame_index, layer_index) to an actual image object
        let mut image_vec: Vec<Image> = Vec::new();
        let mut image_map: HashMap<(usize, usize), usize> = HashMap::new();
        for (frame_index, frame) in file.frames.iter().enumerate() {
            for cel in frame.cels.iter().filter_map(|x| x.as_ref()) {
                if let CelContent::Image(image) = &cel.content {
                    let image_index = image_vec.len();
                    image_vec.push(image.clone());
                    image_map.insert((frame_index, cel.layer_index.into()), image_index);
                }
            }
        }
        let mut frames: Vec<Vec<Frame>> = Vec::with_capacity(file.tags.len() * file.layers.len());
        for tag in file.tags.iter() {
            for layer_index in 0..layers.len() {
                let mut image_refs: Vec<Frame> = Vec::new();
                for frame_index in tag.frames.clone() {
                    let frame_index = usize::from(frame_index);
                    // FIXME make sure the frame_index is < frames
                    let frame = file
                        .frames
                        .get(frame_index)
                        .ok_or(LoadSpriteError::FrameIndexOutOfRange(frame_index))?;
                    let cel = frame.cels[layer_index]
                        .as_ref()
                        .ok_or(LoadSpriteError::Parse {
                            message: format!(
                                "Tag {:?} references non existant cel (frame={}, layer={})",
                                tag.name, frame_index, layer_index
                            ),
                        })?;
                    let image_index = match cel.content {
                        CelContent::Image(_) => image_map[&(frame_index, layer_index)],
                        CelContent::LinkedCel { frame_position } => {
                            *image_map.get(&(frame_position.into(), layer_index))
                                .ok_or_else(|| LoadSpriteError::Parse {
                                    message: format!(
                                        "Cel(frame={}, layer={}) references anothes linked cel (frame={}, layer={}) and not an image cel.", 
                                        frame_index, layer_index, frame_position, layer_index
                                    )
                                })?
                        }
                        _ => return Err(LoadSpriteError::Parse { message: format!("Cel(frame={}, layer={}) referenced by tag {:?} is neither a image cel nor a linked cel", frame_index, layer_index, tag.name)})
                    };
                    let image = &image_vec[image_index];
                    image_refs.push(Frame {
                        duration: frame.duration,
                        origin: (cel.x, cel.y),
                        size: (image.width, image.height),
                        image_index,
                    });
                }
                frames.push(image_refs);
            }
        }
        Ok(AsepriteSpriteLoader {
            file,
            tags,
            layers,
            frames,
            images: image_vec,
        })
    }
}

impl<'a> SpriteLoader for AsepriteSpriteLoader<'a> {
    fn size(&self) -> (u16, u16) {
        (self.file.header.width, self.file.header.height)
    }
    fn tags(&self) -> &[String] {
        &self.tags
    }
    fn layers(&self) -> &[String] {
        &self.layers
    }
    fn frames(&self, tag: usize, layer: usize) -> &[Frame] {
        &self.frames[tag * self.layers.len() + layer]
    }
    fn load_image<'b>(
        &self,
        index: usize,
        target: &'b mut [u8],
    ) -> Result<&'b [u8], LoadImageError> {
        let image = self
            .images
            .get(index)
            .ok_or(LoadImageError::InvalidImageIndex)?;
        let target_size = usize::from(image.width * image.height * 4);
        if target.len() < target_size {
            return Err(LoadImageError::TargetBufferTooSmall);
        }
        let target = &mut target[..target_size];
        match (self.file.header.color_depth, image.compressed) {
            (ColorDepth::Rgba, false) => target.copy_from_slice(image.data),
            (ColorDepth::Rgba, true) => decompress(image.data, target)?,
            (ColorDepth::Grayscale, false) => {
                grayscale_to_rgba(image.data, target)?;
            }
            (ColorDepth::Grayscale, true) => {
                let mut buf = vec![0u8; (image.width * image.height * 2).into()];
                decompress(image.data, &mut buf)?;
                grayscale_to_rgba(&buf, target)?;
            }
            (ColorDepth::Indexed, false) => {
                indexed_to_rgba(
                    image.data,
                    self.file
                        .palette
                        .as_ref()
                        .ok_or(LoadImageError::MissingPalette)?,
                    target,
                )?;
            }
            (ColorDepth::Indexed, true) => {
                let mut buf = vec![0u8; (image.width * image.height).into()];
                decompress(image.data, &mut buf)?;
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

//! This module contains the actual loader API. This API is based on the
//! `binary`-module of this crate and does provide a convenience API for
//! accessing layers, frames, tags and fully blended images.

use flate2::Decompress;
use std::{
    collections::hash_map::DefaultHasher,
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::RangeInclusive,
};

mod blend;

use crate::{
    binary::{
        blend_mode::BlendMode,
        chunks::{
            cel::CelContent,
            layer::{LayerFlags, LayerType},
            slice::SliceChunk,
            tags::AnimationDirection,
        },
        color_depth::ColorDepth,
        file::{parse_file, File},
        image::Image,
        palette::Palette,
    },
    loader::blend::{blend_mode_to_blend_fn, Color},
};

/// This can be used to load an Aseprite file.
#[derive(Debug)]
pub struct AsepriteFile<'a> {
    pub file: File<'a>,
    /// All layers in the file in order
    pub layers: Vec<Layer>,
    /// All frames in the file in order
    pub frames: Vec<Frame>,
    /// All tags in the file
    pub tags: Vec<Tag>,
    /// All images in the file
    pub images: Vec<Image<'a>>,
}

/// A cel in a frame
///
/// This is a reference to an image cel
#[derive(Debug, Copy, Clone)]
pub struct FrameCel {
    pub origin: (i16, i16),
    pub size: (u16, u16),
    pub layer_index: usize,
    pub image_index: usize,
}

/// A frame in the file
///
/// This is a collection of cels for each layer
#[derive(Debug, Clone)]
pub struct Frame {
    pub duration: u16,
    pub origin: (i16, i16),
    pub cels: Vec<FrameCel>,
}

/// A tag in the file
///
/// This is a range of frames over the frames in the file, ordered by frame index
#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub range: RangeInclusive<u16>,
    pub direction: AnimationDirection,
    pub repeat: Option<u16>,
}

/// A layer in the file
#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub opacity: u8,
    pub blend_mode: BlendMode,
    pub visible: bool,
}

impl AsepriteFile<'_> {
    /// Load a aseprite file from a byte slice
    pub fn load(data: &[u8]) -> Result<AsepriteFile<'_>, LoadSpriteError> {
        let file = parse_file(data).map_err(|e| LoadSpriteError::Parse {
            message: e.to_string(),
        })?;
        let layers: Vec<_> = file
            .layers
            .iter()
            .filter_map(|layer| {
                if layer.layer_type == LayerType::Normal {
                    Some(Layer {
                        name: layer.name.to_string(),
                        opacity: layer.opacity,
                        blend_mode: layer.blend_mode,
                        visible: layer.flags.contains(LayerFlags::VISIBLE),
                    })
                } else {
                    None
                }
            })
            .collect();

        let mut image_vec: Vec<Image<'_>> = Vec::new();
        let mut image_map: HashMap<(usize, usize), usize> = HashMap::new();

        for (frame_index, frame) in file.frames.iter().enumerate() {
            for cel in frame.cels.iter().filter_map(|x| x.as_ref()) {
                if let CelContent::Image(image) = &cel.content {
                    let image_index = image_vec.len();
                    image_vec.push(image.clone());
                    let _ = image_map.insert((frame_index, cel.layer_index.into()), image_index);
                }
            }
        }

        let mut frames: Vec<Frame> = Vec::new();
        let mut tags: Vec<Tag> = Vec::new();

        for tag in file.tags.iter() {
            tags.push(Tag {
                name: tag.name.to_string(),
                range: tag.frames.clone(),
                direction: tag.animation_direction,
                repeat: if tag.animation_repeat > 0 {
                    Some(tag.animation_repeat)
                } else {
                    None
                },
            });
        }

        for (index, frame) in file.frames.iter().enumerate() {
            let mut cels: Vec<FrameCel> = Vec::new();
            for cel in frame.cels.iter().filter_map(|x| x.as_ref()) {
                let image_index = match cel.content {
                    CelContent::Image(_) => image_map[&(index, cel.layer_index.into())],
                    CelContent::LinkedCel { frame_position } => *image_map
                        .get(&(frame_position.into(), cel.layer_index.into()))
                        .ok_or_else(|| LoadSpriteError::Parse {
                            message: format!(
                                "invalid linked cel at frame {} layer {}",
                                index, cel.layer_index
                            ),
                        })?,
                    _ => {
                        return Err(LoadSpriteError::Parse {
                            message: "invalid cel".to_owned(),
                        })
                    }
                };
                let width = image_vec[image_index].width;
                let height = image_vec[image_index].height;
                cels.push(FrameCel {
                    origin: (cel.x, cel.y),
                    size: (width, height),
                    layer_index: cel.layer_index.into(),
                    image_index,
                });
            }

            frames.push(Frame {
                duration: frame.duration,
                origin: (0, 0),
                cels,
            });
        }

        Ok(AsepriteFile {
            file,
            tags,
            layers,
            frames,
            images: image_vec,
        })
    }
    /// Get size of the sprite (width, height)
    pub fn size(&self) -> (u16, u16) {
        (self.file.header.width, self.file.header.height)
    }
    /// Get tag names
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
    /// Get layer names
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
    /// Get the image indices for a given tag and layer
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
    /// Get image count
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Get image loader for a given frame index
    /// This will combine all layers into a single image
    /// returns a hash describing the image, since cels can be reused in multiple frames
    pub fn combined_frame_image(
        &self,
        frame_index: usize,
        target: &mut [u8],
    ) -> Result<u64, LoadImageError> {
        let mut hasher = DefaultHasher::new();

        let target_size =
            usize::from(self.file.header.width) * usize::from(self.file.header.height) * 4;

        if target.len() < target_size {
            return Err(LoadImageError::TargetBufferTooSmall);
        }

        let frame = &self.frames[frame_index];

        for cel in frame.cels.iter() {
            let layer = &self.layers[cel.layer_index];
            if !layer.visible {
                continue;
            }

            let mut cel_target = vec![0; usize::from(cel.size.0) * usize::from(cel.size.1) * 4];
            self.load_image(cel.image_index, &mut cel_target).unwrap();
            let layer = &self.layers[cel.layer_index];

            (cel.image_index, cel.layer_index, cel.origin, cel.size).hash(&mut hasher);

            let blend_fn = blend_mode_to_blend_fn(layer.blend_mode);

            for y in 0..cel.size.1 {
                for x in 0..cel.size.0 {
                    let Some(target_x) = x.checked_add_signed(cel.origin.0) else {
                        continue;
                    };
                    if target_x >= self.file.header.width {
                        continue;
                    }
                    let Some(target_y) = y.checked_add_signed(cel.origin.1) else {
                        continue;
                    };
                    if target_y >= self.file.header.height {
                        continue;
                    }

                    let target_index = usize::from(target_y) * usize::from(self.file.header.width)
                        + usize::from(target_x);
                    let cel_index = usize::from(y) * usize::from(cel.size.0) + usize::from(x);

                    let cel_pixel: &[u8] = &cel_target[cel_index * 4..cel_index * 4 + 4];
                    let target_pixel: &mut [u8] =
                        &mut target[target_index * 4..target_index * 4 + 4];

                    let back = Color::from(&*target_pixel);
                    let front = Color::from(cel_pixel);
                    let out = blend_fn(back, front, layer.opacity);

                    target_pixel[0] = out.r;
                    target_pixel[1] = out.g;
                    target_pixel[2] = out.b;
                    target_pixel[3] = out.a;
                }
            }
        }

        Ok(hasher.finish())
    }

    /// Get image loader for a given image index
    pub fn load_image(&self, index: usize, target: &mut [u8]) -> Result<(), LoadImageError> {
        let image = &self.images[index];
        let target_size = usize::from(image.width) * usize::from(image.height) * 4;
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
                let mut buf = vec![0u8; usize::from(image.width) * usize::from(image.height) * 2];
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
                let mut buf = vec![0u8; usize::from(image.width) * usize::from(image.height)];
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
        Ok(())
    }
    pub fn slices(&self) -> &[SliceChunk<'_>] {
        &self.file.slices
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadSpriteError {
    #[error("parsing failed {message}")]
    Parse { message: String },
    #[error("missing tag: {0}")]
    MissingTag(String),
    #[error("missing layer: {0}")]
    MissingLayer(String),
    #[error("frame index out of range: {0}")]
    FrameIndexOutOfRange(usize),
}

#[allow(missing_copy_implementations)]
#[derive(Error, Debug)]
pub enum LoadImageError {
    #[error("target buffer too small")]
    TargetBufferTooSmall,
    #[error("missing palette")]
    MissingPalette,
    #[error("unsupported color depth")]
    UnsupportedColorDepth,
    #[error("decompression failed")]
    DecompressError,
    #[error("invalid image data")]
    InvalidImageData,
}

pub fn decompress(data: &[u8], target: &mut [u8]) -> Result<(), LoadImageError> {
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
        let color = palette.colors[usize::from(*px)];
        target[i * 4] = color.red;
        target[i * 4 + 1] = color.green;
        target[i * 4 + 2] = color.blue;
        target[i * 4 + 3] = color.alpha;
    }
    Ok(())
}

#[test]
fn test_cel() {
    use image::RgbaImage;
    use tempfile::TempDir;

    let path = "./tests/combine.aseprite";
    let file = std::fs::read(path).unwrap();
    let file = AsepriteFile::load(&file).unwrap();

    for frame in file.frames().iter() {
        for (i, cel) in frame.cels.iter().enumerate() {
            let (width, height) = cel.size;

            let mut target = vec![0; usize::from(width * height) * 4];
            file.load_image(cel.image_index, &mut target).unwrap();

            let image = RgbaImage::from_raw(u32::from(width), u32::from(height), target).unwrap();

            let tmp = TempDir::with_prefix("aseprite-loader").unwrap();
            let path = tmp.path().join(format!("cel_{}.png", i));
            image.save(path).unwrap();
        }
    }
}

#[test]
fn test_combine() {
    use image::RgbaImage;
    use tempfile::TempDir;

    let path = "./tests/combine.aseprite";
    let file = std::fs::read(path).unwrap();
    let file = AsepriteFile::load(&file).unwrap();

    let (width, height) = file.size();
    for (index, _) in file.frames().iter().enumerate() {
        let mut target = vec![0; usize::from(width * height) * 4];
        let _ = file.combined_frame_image(index, &mut target).unwrap();
        let image = RgbaImage::from_raw(u32::from(width), u32::from(height), target).unwrap();

        let tmp = TempDir::with_prefix("aseprite-loader").unwrap();
        println!("{:?}", tmp);
        let path = tmp.path().join(format!("combined_{}.png", index));
        image.save(path).unwrap();
    }
}

/// https://github.com/bikeshedder/aseprite-loader/issues/4
#[test]
fn test_issue_4_1() {
    let path = "./tests/issue_4_1.aseprite";
    let file = std::fs::read(path).unwrap();
    let file = AsepriteFile::load(&file).unwrap();
    let (width, height) = file.size();
    let mut buf = vec![0; usize::from(width * height) * 4];
    for idx in 0..file.frames().len() {
        let _ = file.combined_frame_image(idx, &mut buf).unwrap();
    }
}

#[test]
fn test_issue_4_2() {
    let path = "./tests/issue_4_2.aseprite";
    let file = std::fs::read(path).unwrap();
    let file = AsepriteFile::load(&file).unwrap();
    let (width, height) = file.size();
    let mut buf = vec![0; usize::from(width * height) * 4];
    for idx in 0..file.frames().len() {
        let _ = file.combined_frame_image(idx, &mut buf).unwrap();
    }
}

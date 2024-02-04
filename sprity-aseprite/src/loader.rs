use std::{collections::HashMap, ops::Range};

use flate2::Decompress;

use crate::binary::{
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
};

#[derive(Debug)]
pub struct AsepriteFile<'a> {
    file: File<'a>,
    /// All layers in the file in order
    layers: Vec<Layer>,
    /// All frames in the file in order
    frames: Vec<Frame>,
    /// All tags in the file
    tags: Vec<Tag>,
    /// All images in the file
    images: Vec<Image<'a>>,
}

/// A cell in a frame
/// This is a reference to an image cell
#[derive(Debug, Copy, Clone)]
pub struct FrameCell {
    pub origin: (i16, i16),
    pub size: (u16, u16),
    pub layer_index: usize,
    pub image_index: usize,
}

/// A frame in the file
/// This is a collection of cells for each layer
#[derive(Debug, Clone)]
pub struct Frame {
    pub duration: u16,
    pub origin: (i16, i16),
    pub cells: Vec<FrameCell>,
}

/// A tag in the file
/// This is a range of frames over the frames in the file, ordered by frame index
#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub range: Range<u16>,
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
            let mut cells: Vec<FrameCell> = Vec::new();
            for cel in frame.cels.iter().filter_map(|x| x.as_ref()) {
                let image_index = match cel.content {
                    CelContent::Image(_) => image_map[&(index, cel.layer_index.into())],
                    CelContent::LinkedCel { frame_position } => *image_map
                        .get(&(frame_position.into(), cel.layer_index.into()))
                        .ok_or_else(|| LoadSpriteError::Parse {
                            message: format!(
                                "invalid linked cell at frame {} layer {}",
                                index, cel.layer_index
                            ),
                        })?,
                    _ => {
                        return Err(LoadSpriteError::Parse {
                            message: "invalid cell".to_owned(),
                        })
                    }
                };
                let width = image_vec[image_index].width;
                let height = image_vec[image_index].height;
                cells.push(FrameCell {
                    origin: (cel.x, cel.y),
                    size: (width, height),
                    layer_index: cel.layer_index.into(),
                    image_index,
                });
            }

            frames.push(Frame {
                duration: frame.duration,
                origin: (0, 0),
                cells,
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
    /// returns a hash describing the image, since cells can be reused in multiple frames
    pub fn combined_frame_image(
        &self,
        frame_index: usize,
        target: &mut [u8],
    ) -> Result<u64, LoadImageError> {
        let mut hash = 0u64;

        let target_size =
            usize::from(self.file.header.width) * usize::from(self.file.header.height) * 4;

        if target.len() < target_size {
            return Err(LoadImageError::TargetBufferTooSmall);
        }

        let frame = &self.frames[frame_index];

        for cell in frame.cells.iter() {
            let layer = &self.layers[cell.layer_index];
            if layer.visible == false {
                continue;
            }

            let mut cell_target = vec![0; usize::from(cell.size.0) * usize::from(cell.size.1) * 4];
            self.load_image(cell.image_index, &mut cell_target).unwrap();
            let layer = &self.layers[cell.layer_index];

            hash += cell.image_index as u64;
            hash += cell.layer_index as u64 * 100;
            hash += cell.origin.0 as u64 * 10000;
            hash += cell.origin.1 as u64 * 1000000;
            hash += cell.size.0 as u64 * 100000000;
            hash += cell.size.1 as u64 * 10000000000;

            for y in 0..cell.size.1 {
                for x in 0..cell.size.0 {
                    let origin_x = usize::from(x + cell.origin.0 as u16);
                    let origin_y = usize::from(y + cell.origin.1 as u16);

                    let target_index =
                        (origin_y * usize::from(self.file.header.width) + origin_x) as usize;
                    let cell_index =
                        (usize::from(y) * usize::from(cell.size.0) + usize::from(x)) as usize;

                    let target_pixel: &mut [u8] =
                        &mut target[target_index * 4..target_index * 4 + 4];

                    let cell_pixel: &[u8] = &cell_target[cell_index * 4..cell_index * 4 + 4];
                    let cell_alpha = cell_target[cell_index * 4 + 3];

                    let total_alpha = ((cell_alpha as u16 * layer.opacity as u16) / 255) as u8;

                    for i in 0..4 {
                        target_pixel[i] = blend_channel(
                            target_pixel[i],
                            cell_pixel[i],
                            total_alpha,
                            layer.blend_mode,
                        );
                    }
                }
            }
        }

        Ok(hash)
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

fn blend_channel(first: u8, second: u8, alpha: u8, blend_mode: BlendMode) -> u8 {
    let alpha = alpha as f32 / 255.0;
    let first = first as f32 / 255.0;
    let second = second as f32 / 255.0;

    let result = match blend_mode {
        BlendMode::Normal => second,
        BlendMode::Multiply => first * second,
        BlendMode::Screen => 1.0 - (1.0 - first) * (1.0 - second),
        BlendMode::Darken => first.min(second),
        BlendMode::Lighten => first.max(second),
        BlendMode::Addition => (first + second).min(1.0),
        BlendMode::Subtract => (first - second).max(0.0),
        BlendMode::Difference => (first - second).abs(),
        BlendMode::Overlay => {
            if first < 0.5 {
                2.0 * first * second
            } else {
                1.0 - 2.0 * (1.0 - first) * (1.0 - second)
            }
        }
        // @todo: missing modes
        _ => first,
    };

    let blended = first * (1.0 - alpha) + result * alpha;
    (blended.min(1.0).max(0.0) * 255.0).round() as u8
}

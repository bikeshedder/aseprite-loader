use std::collections::BTreeMap;

use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};
use thiserror::Error;

use crate::{LoadImageError, SpriteLoader};

const MAX_WIDTH: u16 = 4096;
const MAX_HEIGHT: u16 = 4096;

#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub texture: RgbaImage,
    pub sprites: Vec<Sprite>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Sprite {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Using the loader figure out a possible initial size
/// for the packing algorithm.
fn initial_size(loader: &dyn SpriteLoader) -> (u16, u16) {
    let mult = (loader.images() as f32).sqrt() as u16;
    let size = loader.size();
    (size.0 * mult / 2, size.1 * mult / 2)
}

fn try_sizes(initial_size: (u16, u16)) -> impl Iterator<Item = (u16, u16)> {
    (0..).scan(initial_size, |size, step| {
        if step == 0 {
            Some(*size)
        } else if size.0 >= MAX_WIDTH && size.1 >= MAX_HEIGHT {
            None
        } else {
            // Increment width and height by 1.125 making sure the size
            // increases by at least 8 pixels each time.
            *size = (
                (size.0 * 9 / 8).clamp(size.0 + 8, MAX_WIDTH),
                (size.1 * 9 / 8).clamp(size.1 + 8, MAX_HEIGHT),
            );
            Some(*size)
        }
    })
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("Not enough space")]
    NotEnoughSpace,
    #[error("Image loading failed: {0}")]
    LoadImage(#[from] LoadImageError),
}

pub fn pack(sprite_loader: &dyn SpriteLoader) -> Result<SpriteSheet, PackError> {
    // Prepare rectangles to be placed on the sprite sheet
    // without actually loading the images.
    let mut rects_to_place: GroupedRectsToPlace<usize> = GroupedRectsToPlace::new();
    for image_index in 0..sprite_loader.images() {
        let image = sprite_loader.image_loader(image_index);
        let size = image.size();
        rects_to_place.push_rect(
            image_index,
            None,
            RectToInsert::new(size.0.into(), size.1.into(), 1),
        );
    }

    // Start rectangle_pack algorithm with an increasing
    // size for the target texture.
    let mut target_bins = BTreeMap::new();
    let placements = try_sizes(initial_size(sprite_loader))
        .filter_map(|size| {
            target_bins.insert(0, TargetBin::new(size.0.into(), size.1.into(), 1));
            pack_rects(
                &rects_to_place,
                &mut target_bins,
                &volume_heuristic,
                &contains_smallest_box,
            )
            .ok()
        })
        .next()
        .ok_or(PackError::NotEnoughSpace)?;

    // Find width and height for target texture.
    let (width, height) =
        placements
            .packed_locations()
            .values()
            .fold((0, 0), |(width, height), (_, location)| {
                (
                    width.max(location.x() + location.width()),
                    height.max(location.y() + location.height()),
                )
            });

    // Load sprites and copy them into the target texture
    // to the appropriate positions.
    let mut texture = RgbaImage::new(width, height);
    let sprite_size = sprite_loader.size();
    let mut buf = vec![0u8; (sprite_size.0 * sprite_size.1 * 4).into()];
    let mut sprites = vec![Sprite::default(); sprite_loader.images()];
    for (&image_index, (_, location)) in placements.packed_locations() {
        let image_loader = sprite_loader.image_loader(image_index);
        let image_size = image_loader.size();
        let buf = image_loader.load(&mut buf)?;
        let img =
            ImageBuffer::<Rgba<u8>, _>::from_raw(image_size.0.into(), image_size.1.into(), buf)
                .expect("Image creation failed. This should never happen and means that there is a bug in the packing algorithm.");
        texture.copy_from(&img, location.x(), location.y())
            .expect("Image copying failed. This should never happen and means that there is a bug in the packing algorithm.");
        sprites[image_index] = Sprite {
            x: location.x(),
            y: location.y(),
            width: location.width(),
            height: location.height(),
        };
    }

    Ok(SpriteSheet { texture, sprites })
}

#[test]
fn test_try_sizes() {
    assert_eq!(
        try_sizes((256, 256)).collect::<Vec<_>>(),
        vec![
            (256, 256),
            (384, 384),
            (576, 576),
            (864, 864),
            (1296, 1296),
            (1944, 1944),
            (2916, 2916),
            (4096, 4096)
        ]
    )
}

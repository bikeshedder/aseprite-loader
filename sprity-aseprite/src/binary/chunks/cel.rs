use nom::bytes::complete::take;
use strum_macros::FromRepr;

use crate::binary::{
    errors::ParseResult,
    scalars::{byte, dword, short, word, Byte, Dword, Short, Word},
};

#[derive(Debug)]
pub struct CelChunk<'a> {
    /// Layer index (see NOTE.2)
    pub layer_index: Word,
    /// X position
    pub x: Short,
    /// Y position
    pub y: Short,
    /// Opacity level
    pub opacity: Byte,
    /// Cel Data
    pub content: CelContent<'a>,
}

#[derive(Debug, FromRepr)]

enum CelType {
    /// 0 - Raw Image Data (unused, compressed image is preferred)
    RawImageData,
    /// 1 - Linked Cel
    LinkedCel,
    /// 2 - Compressed Image
    CompressedImage,
    /// 3 - Compressed Tilemap
    CompressedTilemap,
    Unknown(Word),
}

impl From<Word> for CelType {
    fn from(word: Word) -> Self {
        CelType::from_repr(word.into()).unwrap_or(Self::Unknown(word))
    }
}

#[derive(Debug)]
pub enum CelContent<'a> {
    RawImageData {
        /// Width in pixels
        width: Word,
        /// Height in pixels
        height: Word,
        /// Raw pixel data: row by row from top to bottom,
        /// for each scanline read pixels from left to right.
        data: &'a [u8],
    },
    LinkedCel {
        /// Frame position to link with
        frame_position: Word,
    },
    CompressedImage {
        /// Width in pixels
        width: Word,
        /// Height in pixels
        height: Word,
        /// "Raw Cel" data compressed with ZLIB method (see NOTE.3)
        data: &'a [u8],
    },
    CompressedTilemap {
        /// Width in number of tiles
        width: Word,
        /// Height in number of tiles
        height: Word,
        /// Bits per tile (at the moment it's always 32-bit per tile)
        bits_per_tile: Word,
        /// Bitmask for tile ID (e.g. 0x1fffffff for 32-bit tiles)
        bitmask_tile_id: Dword,
        /// Bitmask for X flip
        bitmask_x_flip: Dword,
        /// Bitmask for Y-Flip
        bitmask_y_flip: Dword,
        /// Bitmask for 90CW rotation,
        bitmask_90cw_rotation: Dword,
        /// Row by row, from top to bottom tile by tile
        /// compressed with ZLIB method (see NOTE.3)
        data: &'a [u8],
    },
    Unknown(&'a [u8]),
}

impl<'a> CelChunk<'a> {
    pub fn image(&self) -> Option<ImageCel> {
        match self.content {
            CelContent::RawImageData {
                width,
                height,
                data,
            } => Some(ImageCel {
                x: self.x,
                y: self.y,
                width,
                height,
                data,
                compressed: false,
            }),
            CelContent::CompressedImage {
                width,
                height,
                data,
            } => Some(ImageCel {
                x: self.x,
                y: self.y,
                width,
                height,
                data,
                compressed: true,
            }),
            _ => None,
        }
    }
}

pub fn parse_cel_chunk<'a>(input: &'a [u8]) -> ParseResult<CelChunk<'a>> {
    let (input, layer_index) = word(input)?;
    let (input, x) = short(input)?;
    let (input, y) = short(input)?;
    let (input, opacity) = byte(input)?;
    let (input, cel_type) = word(input)?;
    let cel_type = CelType::from(cel_type);
    let (input, _) = take(7usize)(input)?;
    let content = match cel_type {
        CelType::RawImageData => {
            let (input, width) = word(input)?;
            let (input, height) = word(input)?;
            CelContent::RawImageData {
                width,
                height,
                data: input,
            }
        }
        CelType::LinkedCel => {
            let (_, frame_position) = word(input)?;
            CelContent::LinkedCel { frame_position }
        }
        CelType::CompressedImage => {
            let (input, width) = word(input)?;
            let (input, height) = word(input)?;
            CelContent::CompressedImage {
                width,
                height,
                data: input,
            }
        }
        CelType::CompressedTilemap => {
            let (input, width) = word(input)?;
            let (input, height) = word(input)?;
            let (input, bits_per_tile) = word(input)?;
            let (input, bitmask_tile_id) = dword(input)?;
            let (input, bitmask_y_flip) = dword(input)?;
            let (input, bitmask_x_flip) = dword(input)?;
            let (input, bitmask_90cw_rotation) = dword(input)?;
            let (input, _) = take(10usize)(input)?;
            CelContent::CompressedTilemap {
                width,
                height,
                bits_per_tile,
                bitmask_tile_id,
                bitmask_x_flip,
                bitmask_y_flip,
                bitmask_90cw_rotation,
                data: input,
            }
        }
        CelType::Unknown(_) => CelContent::Unknown(input),
    };
    Ok((
        &input[input.len()..],
        CelChunk {
            layer_index,
            x,
            y,
            opacity,
            content,
        },
    ))
}

pub struct ImageCel<'a> {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub compressed: bool,
    pub data: &'a [u8],
}

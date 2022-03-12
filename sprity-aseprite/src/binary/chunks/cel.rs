use nom::bytes::complete::take;
use strum_macros::FromRepr;

use crate::binary::{
    errors::ParseResult,
    scalars::{byte, dword, short, word, Byte, Dword, Short, Word},
};

#[derive(Debug)]
pub struct CelChunk<'a> {
    /// Layer index (see NOTE.2)
    layer_index: Word,
    /// X position
    x: Short,
    /// Y position
    y: Short,
    /// Opacity level
    opacity: Byte,
    /// Cel Type
    /// 0 - Raw Image Data (unused, compressed image is preferred)
    /// 1 - Linked Cel
    /// 2 - Compressed Image
    /// 3 - Compressed Tilemap
    cel_type: CelType,
    cel_type_data: CelTypeData<'a>,
}

#[derive(Debug, FromRepr)]
pub enum CelType {
    RawImageData,
    LinkedCel,
    CompressedImage,
    CompressedTilemap,
    Unknown(Word),
}

impl From<Word> for CelType {
    fn from(word: Word) -> Self {
        CelType::from_repr(word.into()).unwrap_or(Self::Unknown(word))
    }
}

#[derive(Debug)]
pub enum CelTypeData<'a> {
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

pub fn parse_cel_chunk<'a>(input: &'a [u8]) -> ParseResult<CelChunk<'a>> {
    let (input, layer_index) = word(input)?;
    let (input, x) = short(input)?;
    let (input, y) = short(input)?;
    let (input, opacity) = byte(input)?;
    let (input, cel_type) = word(input)?;
    let cel_type = CelType::from(cel_type);
    let (input, _) = take(7usize)(input)?;
    let cel_type_data = match cel_type {
        CelType::RawImageData => {
            let (input, width) = word(input)?;
            let (input, height) = word(input)?;
            CelTypeData::RawImageData {
                width,
                height,
                data: input,
            }
        }
        CelType::LinkedCel => {
            let (input, frame_position) = word(input)?;
            CelTypeData::LinkedCel { frame_position }
        }
        CelType::CompressedImage => {
            let (input, width) = word(input)?;
            let (input, height) = word(input)?;
            CelTypeData::CompressedImage {
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
            CelTypeData::CompressedTilemap {
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
        CelType::Unknown(_) => CelTypeData::Unknown(input),
    };
    Ok((
        &input[input.len()..],
        CelChunk {
            layer_index,
            x,
            y,
            opacity,
            cel_type,
            cel_type_data,
        },
    ))
}

use bitflags::bitflags;
use nom::bytes::complete::take;

use crate::binary::{
    blend_mode::BlendMode,
    errors::ParseResult,
    scalars::{byte, dword, parse_string, word, Byte, Dword, Word},
};

pub struct LayerChunk {
    pub flags: LayerFlags,
    pub layer_type: LayerType,
    pub child_level: Word,
    pub blend_mode: BlendMode,
    pub opacity: Byte,
    pub name: String,
    pub tileset_index: Option<Dword>,
}

bitflags! {
    pub struct LayerFlags: Word {
        const VISIBLE = 0x1;
        const EDITABLE = 0x2;
        const LOCK_MOVEMENT = 0x4;
        const BACKGROUND = 0x8;
        const PREFER_LINKED_CELS = 0x16;
        const COLLAPSED = 0x32;
        const REFERENCE = 0x64;
    }
}

pub enum LayerType {
    Normal,
    Group,
    Tilemap,
    Unknown(Word),
}

impl From<Word> for LayerType {
    fn from(word: Word) -> Self {
        match word {
            0 => Self::Normal,
            1 => Self::Group,
            2 => Self::Tilemap,
            n => Self::Unknown(n),
        }
    }
}

pub fn parse_layer_chunk(input: &[u8]) -> ParseResult<LayerChunk> {
    let (input, flags) = word(input)?;
    let flags = LayerFlags::from_bits_truncate(flags);
    let (input, layer_type) = word(input)?;
    let layer_type = LayerType::from(layer_type);
    let (input, child_level) = word(input)?;
    let (input, _default_layer_width) = word(input)?;
    let (input, _default_layer_height) = word(input)?;
    let (input, blend_mode) = word(input)?;
    let blend_mode = BlendMode::from(blend_mode);
    let (input, opacity) = byte(input)?;
    let (input, _) = take(3usize)(input)?;
    let (input, name) = parse_string(input)?;
    let (input, tileset_index) = match layer_type {
        LayerType::Tilemap => {
            let (input, tileset_index) = dword(input)?;
            (input, Some(tileset_index))
        }
        _ => (input, None),
    };
    Ok((
        input,
        LayerChunk {
            flags,
            layer_type,
            child_level,
            blend_mode,
            opacity,
            name,
            tileset_index,
        },
    ))
}

use bitflags::bitflags;
use nom::{bytes::complete::take, combinator::cond, Parser};

use crate::binary::{
    blend_mode::BlendMode,
    errors::ParseResult,
    scalars::{byte, dword, parse_string, word, Byte, Dword, Word},
};

#[derive(Debug)]
pub struct LayerChunk<'a> {
    pub flags: LayerFlags,
    pub layer_type: LayerType,
    pub child_level: Word,
    pub blend_mode: BlendMode,
    pub opacity: Byte,
    pub name: &'a str,
    pub tileset_index: Option<Dword>,
}

bitflags! {
    #[derive(Debug)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn parse_layer_chunk(input: &[u8]) -> ParseResult<'_, LayerChunk<'_>> {
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
    let (input, tileset_index) =
        cond(matches!(layer_type, LayerType::Tilemap), dword).parse(input)?;
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

#[test]
fn test_layers() {
    use crate::binary::file::parse_file;
    let input = std::fs::read("./tests/layers.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.frames.len(), 1);
    assert_eq!(file.frames[0].duration, 100);
    assert_eq!(file.layers.len(), 3);
    assert_eq!(file.layers[0].name, "Layer 1");
    assert_eq!(file.layers[1].name, "Layer 2");
    assert_eq!(file.layers[2].name, "Layer 3");
}

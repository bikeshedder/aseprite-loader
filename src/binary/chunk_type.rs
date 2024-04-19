use nom::combinator::map;
use strum_macros::FromRepr;

use super::{errors::ParseResult, scalars::word};

#[derive(FromRepr, Debug, Copy, Clone)]
pub enum ChunkType {
    Palette0004 = 0x0004,
    Palette0011 = 0x0011,
    Layer = 0x2004,
    Cel = 0x2005,
    CelExtra = 0x2006,
    ColorProfile = 0x2007,
    ExternalFile = 0x2008,
    Mask = 0x2016,
    Path = 0x2017,
    Tags = 0x2018,
    Palette = 0x2019,
    UserData = 0x2020,
    Slice = 0x2022,
    Tileset = 0x2023,
}

pub fn parse_chunk_type(input: &[u8]) -> ParseResult<'_, Result<ChunkType, u16>> {
    map(word, |n| ChunkType::from_repr(n.into()).ok_or(n))(input)
}

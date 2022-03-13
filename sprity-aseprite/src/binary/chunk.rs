use nom::{
    bytes::complete::take,
    combinator::{flat_map, map},
    multi::count,
};

use crate::binary::chunk_type::{parse_chunk_type, ChunkType};

use super::{
    chunks::{
        cel::{parse_cel_chunk, CelChunk},
        cel_extra::{parse_cel_extra_chunk, CelExtraChunk},
        layer::{parse_layer_chunk, LayerChunk},
        tags::{parse_tags_chunk, TagsChunk},
        user_data::{parse_user_data_chunk, UserDataChunk},
    },
    errors::{ParseError, ParseResult},
    scalars::{dword, dword_size, word, Dword},
};

#[derive(Debug)]
pub enum Chunk<'a> {
    Palette0004,
    Palette0011,
    Layer(LayerChunk<'a>),
    Cel(CelChunk<'a>),
    CelExtra(CelExtraChunk<'a>),
    Tags(TagsChunk<'a>),
    UserData(UserDataChunk<'a>),
    NotImplemented(ChunkType),
    Unsupported(u16),
}

pub fn parse_chunks(input: &[u8], chunk_count: usize) -> ParseResult<Vec<Chunk>> {
    count(parse_chunk, chunk_count)(input)
}

pub fn parse_chunk<'a>(input: &'a [u8]) -> ParseResult<Chunk<'a>> {
    let (input, size) = dword_size(input, ParseError::InvalidFrameSize)?;
    let (rest, input) = take(size - 4)(input)?;
    let (chunk_data, chunk_type) = parse_chunk_type(input)?;
    let chunk = match chunk_type {
        Ok(ChunkType::Palette0004) => Chunk::Palette0004,
        Ok(ChunkType::Palette0011) => Chunk::Palette0011,
        Ok(ChunkType::Layer) => Chunk::Layer(parse_layer_chunk(chunk_data)?.1),
        Ok(ChunkType::Cel) => Chunk::Cel(parse_cel_chunk(chunk_data)?.1),
        Ok(ChunkType::CelExtra) => Chunk::CelExtra(parse_cel_extra_chunk(chunk_data)?.1),
        // TODO Color Profile Chunk
        // TODO External Files Chunk
        // TODO Mask Chunk
        // TODO Path Chunk
        Ok(ChunkType::Tags) => Chunk::Tags(parse_tags_chunk(chunk_data)?.1),
        // TODO Palette Chunk
        Ok(ChunkType::UserData) => Chunk::UserData(parse_user_data_chunk(chunk_data)?.1),
        // TODO Slice Chunk
        // TODO Tileset Chunk
        Ok(chunk_type) => Chunk::NotImplemented(chunk_type),
        Err(chunk_type) => Chunk::Unsupported(chunk_type),
    };
    Ok((rest, chunk))
}

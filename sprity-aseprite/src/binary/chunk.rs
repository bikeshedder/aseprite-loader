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
    },
    errors::{ParseError, ParseResult},
    scalars::{dword, dword_size, word, Dword},
};

pub enum Chunk<'a> {
    Palette0004,
    Palette0011,
    Layer(LayerChunk),
    Cel(CelChunk<'a>),
    CelExtra(CelExtraChunk<'a>),
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
        // FIXME implement more chunks
        Ok(chunk_type) => Chunk::NotImplemented(chunk_type),
        Err(chunk_type) => Chunk::Unsupported(chunk_type),
    };
    Ok((rest, chunk))
}

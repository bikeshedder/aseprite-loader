use nom::{
    bytes::complete::take,
    combinator::{flat_map, map},
    multi::count,
};

use crate::binary::chunk_type::{parse_chunk_type, ChunkType};

use super::{
    chunks::layer::{parse_layer_chunk, LayerChunk},
    errors::{ParseError, ParseResult},
    scalars::{dword, dword_size, word, Dword},
};

pub enum Chunk {
    Palette0004,
    Palette0011,
    Layer(LayerChunk),
    NotImplemented(ChunkType),
    Unsupported(u16),
}

pub fn parse_chunks(input: &[u8], chunk_count: usize) -> ParseResult<Vec<Chunk>> {
    count(parse_chunk, chunk_count)(input)
}

pub fn parse_chunk(input: &[u8]) -> ParseResult<Chunk> {
    let (input, size) = dword_size(input, ParseError::InvalidFrameSize)?;
    let (rest, input) = take(size - 4)(input)?;
    let (chunk_data, chunk_type) = parse_chunk_type(input)?;
    let chunk = match chunk_type {
        Ok(ChunkType::Palette0004) => Chunk::Palette0004,
        Ok(ChunkType::Palette0011) => Chunk::Palette0011,
        Ok(ChunkType::Layer) => Chunk::Layer(parse_layer_chunk(chunk_data)?.1),
        // FIXME implement more chunks
        Ok(chunk_type) => Chunk::NotImplemented(chunk_type),
        Err(chunk_type) => Chunk::Unsupported(chunk_type),
    };
    Ok((rest, chunk))
}

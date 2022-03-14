use nom::{
    bytes::complete::{tag, take},
    combinator::{all_consuming, complete},
    multi::many1,
};

use crate::binary::scalars::dword_size;

use super::{
    chunk::parse_chunks,
    chunk::Chunk,
    errors::{ParseError, ParseResult},
    scalars::Word,
    scalars::{parse_dword_as_usize, word},
};

#[derive(Debug)]
pub struct Frame<'a> {
    pub duration: Word,
    pub chunks: Vec<Chunk<'a>>,
}

const FRAME_MAGIC_NUMBER: [u8; 2] = 0xF1FAu16.to_le_bytes();

pub fn parse_frames(input: &[u8]) -> ParseResult<Vec<Frame>> {
    complete(all_consuming(many1(parse_frame)))(input)
}

pub fn parse_frame(input: &[u8]) -> ParseResult<Frame> {
    let (input, size) = dword_size(input, ParseError::InvalidFrameSize)?;
    let (rest, input) = take(size - 4)(input)?;
    let (input, _) = tag(FRAME_MAGIC_NUMBER)(input)?;
    let (input, chunk_count) = word(input)?;
    let (input, duration) = word(input)?;
    let (input, _) = take(2usize)(input)?;
    let (input, chunk_count) = match parse_dword_as_usize(input)? {
        (input, 0) => (input, chunk_count.into()),
        (input, chunk_count) => (input, chunk_count),
    };
    let (_, chunks) = parse_chunks(input, chunk_count)?;
    Ok((rest, Frame { duration, chunks }))
}

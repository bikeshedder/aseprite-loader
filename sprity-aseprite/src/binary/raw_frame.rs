use nom::{
    bytes::complete::{tag, take},
    combinator::{all_consuming, complete},
    multi::many1,
};

use super::{
    chunk::parse_chunks,
    chunk::Chunk,
    chunks::cel::CelChunk,
    errors::{ParseError, ParseResult},
    scalars::dword_size,
    scalars::Word,
    scalars::{parse_dword_as_usize, word},
};

#[derive(Debug)]
pub struct RawFrame<'a> {
    pub duration: Word,
    pub chunks: Vec<Chunk<'a>>,
}

impl<'a> RawFrame<'a> {
    pub fn cels(&self) -> impl Iterator<Item = &CelChunk<'_>> {
        self.chunks.iter().filter_map(|chunk| {
            if let Chunk::Cel(cel) = chunk {
                Some(cel)
            } else {
                None
            }
        })
    }
}

const FRAME_MAGIC_NUMBER: [u8; 2] = 0xF1FAu16.to_le_bytes();

pub fn parse_frames(input: &[u8]) -> ParseResult<'_, Vec<RawFrame<'_>>> {
    complete(all_consuming(many1(parse_rawframe)))(input)
}

pub fn parse_rawframe(input: &[u8]) -> ParseResult<'_, RawFrame<'_>> {
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
    Ok((rest, RawFrame { duration, chunks }))
}

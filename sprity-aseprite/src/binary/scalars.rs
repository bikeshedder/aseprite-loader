//! This file contains all the scalars defined in the
//! specification file.
//!
//! The specification uses names like `WORD`, `SHORT`, etc. which
//! was resulted in some confusion while implementing this parser.
//! Therefore the parser uses the same types making it easy to
//! compare it to the specification.

use nom::{
    bytes::complete::take,
    combinator::{flat_map, map_res},
    number::complete::{le_i16, le_i32, le_u16, le_u32, le_u8},
};

pub type Byte = u8;
pub type Word = u16;
pub type Short = i16;
pub type Dword = u32;
pub type Long = i32;
pub struct Fixed(i64, i64);

use super::errors::{ParseError, ParseResult};

#[inline]
pub fn byte(input: &[u8]) -> ParseResult<Byte> {
    le_u8(input)
}

#[inline]
pub fn word(input: &[u8]) -> ParseResult<Word> {
    le_u16(input)
}

#[inline]
pub fn short(input: &[u8]) -> ParseResult<Short> {
    le_i16(input)
}

#[inline]
pub fn dword(input: &[u8]) -> ParseResult<Dword> {
    le_u32(input)
}

#[inline]
pub fn long(input: &[u8]) -> ParseResult<Long> {
    le_i32(input)
}

/// Parse a DWORD as size information and make sure the
/// parsed size no less than 4. The latter is important as
/// this function is used when parsing frames and
pub fn dword_size<'a>(input: &'a [u8], f: fn(Dword) -> ParseError<'a>) -> ParseResult<'a, Dword> {
    let (input, size) = dword(input)?;
    if size >= 4 {
        Ok((input, size))
    } else {
        Err(nom::Err::Failure(f(size)))
    }
}

pub fn parse_string(input: &[u8]) -> ParseResult<String> {
    map_res(flat_map(word, take), |data| {
        String::from_utf8(data.to_vec())
    })(input)
}

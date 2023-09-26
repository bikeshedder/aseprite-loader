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

#[derive(Debug)]
pub struct Fixed(u16, u16);

pub use sprity_core::Color;

#[derive(Debug)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

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
/// this function is used when parsing frames and chunks
/// where the size includes itself.
pub fn dword_size<'a>(input: &'a [u8], f: fn(Dword) -> ParseError<'a>) -> ParseResult<'a, Dword> {
    let (input, size) = dword(input)?;
    if size >= 4 {
        Ok((input, size))
    } else {
        Err(nom::Err::Failure(f(size)))
    }
}

pub fn parse_dword_as_usize(input: &[u8]) -> ParseResult<usize> {
    let (input, size) = dword(input)?;
    let size = size
        .try_into()
        .map_err(|_| nom::Err::Failure(ParseError::DwordToUsize(size)))?;
    Ok((input, size))
}

pub fn parse_dword_as_u8<'a>(input: &'a [u8], e: ParseError<'a>) -> ParseResult<'a, u8> {
    let (input, size) = dword(input)?;
    let size = size.try_into().map_err(|_| nom::Err::Failure(e))?;
    Ok((input, size))
}

pub fn parse_string(input: &[u8]) -> ParseResult<&str> {
    map_res(flat_map(word, take), std::str::from_utf8)(input)
}

pub fn fixed(input: &[u8]) -> ParseResult<Fixed> {
    let (input, low) = le_u16(input)?;
    let (input, high) = le_u16(input)?;
    Ok((input, Fixed(high, low)))
}

pub fn parse_color(input: &[u8]) -> ParseResult<Color> {
    let (input, red) = byte(input)?;
    let (input, green) = byte(input)?;
    let (input, blue) = byte(input)?;
    let (input, alpha) = byte(input)?;
    Ok((
        input,
        Color {
            red,
            green,
            blue,
            alpha,
        },
    ))
}

pub fn parse_rgb(input: &[u8]) -> ParseResult<RGB> {
    let (input, red) = byte(input)?;
    let (input, green) = byte(input)?;
    let (input, blue) = byte(input)?;
    Ok((input, RGB { red, green, blue }))
}

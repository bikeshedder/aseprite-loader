use std::fmt;

use nom::{error::FromExternalError, IResult};

use super::scalars::Dword;

#[derive(Debug, strum_macros::Display)]
pub enum ParseError<'a> {
    /// This error is returned when the conversion between
    /// DWORD (u32) to usize fails. The only way this can
    /// happen is when running this code on a 16-bit system.
    DwordToUsize(Dword),
    /// This error is returned when the frame size is <4
    InvalidFrameSize(usize),
    /// This error is returned when the chunk size is <4
    InvalidChunkSize(usize),
    Nom(nom::error::Error<&'a [u8]>),
}

impl<'a> std::error::Error for ParseError<'a> {}

impl<'a> nom::error::ParseError<&'a [u8]> for ParseError<'a> {
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        Self::Nom(nom::error::Error::from_error_kind(input, kind))
    }
    fn append(_input: &[u8], _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

pub type ParseResult<'a, O> = IResult<&'a [u8], O, ParseError<'a>>;

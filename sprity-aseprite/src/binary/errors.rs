use std::str::Utf8Error;

use nom::IResult;

use super::scalars::Dword;

#[derive(Debug, strum_macros::Display)]
pub enum ParseError<'a> {
    /// This variant is used when the conversion between
    /// DWORD (u32) to usize fails. The only way this can
    /// happen is when running this code on a 16-bit system.
    DwordToUsize(Dword),
    /// This variant is used when the frame size is <4
    InvalidFrameSize(Dword),
    /// This variant is used when the chunk size is <4
    InvalidChunkSize(Dword),
    /// This variant is used when a string does not contain
    /// valid UTF-8 data and `str::from_utf8` returned an error.
    Utf8Error(Utf8Error),
    /// This variant is used when the nom combinators return
    /// an error.
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

impl<'a> nom::error::FromExternalError<&'a [u8], Utf8Error> for ParseError<'a> {
    fn from_external_error(_input: &'a [u8], _kind: nom::error::ErrorKind, e: Utf8Error) -> Self {
        ParseError::Utf8Error(e)
    }
}

pub type ParseResult<'a, O> = IResult<&'a [u8], O, ParseError<'a>>;

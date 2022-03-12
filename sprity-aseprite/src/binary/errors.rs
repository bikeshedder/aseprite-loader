use std::{fmt, string::FromUtf8Error};

use nom::{error::FromExternalError, IResult};

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
    /// This variant is used when a String does not contain
    /// valid UTF-8 data and String::from_utf8 returned an error.
    FromUtf8Error(FromUtf8Error),
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

impl<'a> nom::error::FromExternalError<&'a [u8], FromUtf8Error> for ParseError<'a> {
    fn from_external_error(input: &'a [u8], kind: nom::error::ErrorKind, e: FromUtf8Error) -> Self {
        ParseError::FromUtf8Error(e)
    }
}

pub type ParseResult<'a, O> = IResult<&'a [u8], O, ParseError<'a>>;

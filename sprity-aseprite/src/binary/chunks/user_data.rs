use bitflags::bitflags;
use nom::combinator::cond;

use crate::binary::{
    errors::ParseResult,
    scalars::{dword, parse_color, parse_string, Color, Dword, Word},
};

bitflags! {
    pub struct UserDataFlags: Dword {
        const HAS_TEXT = 0x1;
        const HAS_COLOR = 0x2;
    }
}

#[derive(Debug)]
pub struct UserDataChunk<'a> {
    text: Option<&'a str>,
    color: Option<Color>,
}

pub fn parse_user_data_chunk(input: &[u8]) -> ParseResult<UserDataChunk> {
    let (input, flags) = dword(input)?;
    let flags = UserDataFlags::from_bits_truncate(flags);
    let (input, text) = cond(flags.contains(UserDataFlags::HAS_TEXT), parse_string)(input)?;
    let (input, color) = cond(flags.contains(UserDataFlags::HAS_COLOR), parse_color)(input)?;
    Ok((input, (UserDataChunk { text, color })))
}

use std::ops::RangeInclusive;

use bitflags::bitflags;
use nom::{bytes::complete::take, combinator::cond, multi::count, Parser};

use crate::binary::{
    errors::{ParseError, ParseResult},
    palette::PaletteError,
    scalars::{
        parse_color, parse_dword_as_u8, parse_dword_as_usize, parse_string, word, Color, Word,
    },
};

#[derive(Debug)]
pub struct PaletteChunk<'a> {
    pub indices: RangeInclusive<u8>,
    pub entries: Vec<PaletteEntry<'a>>,
}

#[derive(Debug)]
pub struct PaletteEntry<'a> {
    pub color: Color,
    pub name: Option<&'a str>,
}

bitflags! {
    pub struct PaletteEntryFlags: Word {
        const HAS_NAME = 0x1;
    }
}

pub fn parse_palette_chunk(input: &[u8]) -> ParseResult<'_, PaletteChunk<'_>> {
    let (input, palette_size) = parse_dword_as_usize(input)?;
    let (input, first_color_index) = parse_dword_as_u8(
        input,
        ParseError::PaletteError(PaletteError::FirstColorIndexOutOfBounds),
    )?;
    let (input, last_color_index) = parse_dword_as_u8(
        input,
        ParseError::PaletteError(PaletteError::LastColorIndexOutOfBounds),
    )?;
    if first_color_index > last_color_index {
        return Err(nom::Err::Failure(ParseError::PaletteError(
            PaletteError::FirstColorIndexGreaterThanLastColorIndex,
        )));
    }
    let (input, _) = take(8usize)(input)?;
    let (input, entries) = count(parse_palette_entry, palette_size).parse(input)?;
    Ok((
        input,
        PaletteChunk {
            indices: (first_color_index..=last_color_index),
            entries,
        },
    ))
}

pub fn parse_palette_entry(input: &[u8]) -> ParseResult<'_, PaletteEntry<'_>> {
    let (input, flags) = word(input)?;
    let flags = PaletteEntryFlags::from_bits_truncate(flags);
    let (input, color) = parse_color(input)?;
    let (input, name) =
        cond(flags.contains(PaletteEntryFlags::HAS_NAME), parse_string).parse(input)?;
    Ok((input, PaletteEntry { color, name }))
}

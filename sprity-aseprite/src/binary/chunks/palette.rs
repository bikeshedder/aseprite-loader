use bitflags::bitflags;
use nom::{bytes::complete::take, combinator::cond, multi::count};

use crate::binary::{
    errors::ParseResult,
    scalars::{
        dword, dword_size, parse_color, parse_dword_as_usize, parse_string, word, Color, Dword,
        Word,
    },
};

#[derive(Debug)]
pub struct PaletteChunk<'a> {
    first_color_index: Dword,
    last_color_index: Dword,
    entries: Vec<PaletteEntry<'a>>,
}

#[derive(Debug)]
pub struct PaletteEntry<'a> {
    color: Color,
    name: Option<&'a str>,
}

bitflags! {
    pub struct PaletteEntryFlags: Word {
        const HAS_NAME = 0x1;
    }
}

pub fn parse_palette_chunk<'a>(input: &'a [u8]) -> ParseResult<PaletteChunk<'a>> {
    let (input, palette_size) = parse_dword_as_usize(input)?;
    let (input, first_color_index) = dword(input)?;
    let (input, last_color_index) = dword(input)?;
    let (input, _) = take(8usize)(input)?;
    let (input, entries) = count(parse_palette_entry, palette_size)(input)?;
    Ok((
        input,
        PaletteChunk {
            first_color_index,
            last_color_index,
            entries,
        },
    ))
}

pub fn parse_palette_entry<'a>(input: &'a [u8]) -> ParseResult<PaletteEntry<'a>> {
    let (input, flags) = word(input)?;
    let flags = PaletteEntryFlags::from_bits_truncate(flags);
    let (input, color) = parse_color(input)?;
    let (input, name) = cond(flags.contains(PaletteEntryFlags::HAS_NAME), parse_string)(input)?;
    Ok((input, PaletteEntry { color, name }))
}

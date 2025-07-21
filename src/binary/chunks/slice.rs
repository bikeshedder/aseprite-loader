use bitflags::bitflags;
use nom::{combinator::cond, multi::count, Parser};

use crate::binary::{
    errors::ParseResult,
    scalars::{dword, long, parse_dword_as_usize, parse_string, Dword, Long},
};

#[derive(Debug)]
pub struct SliceChunk<'a> {
    pub name: &'a str,
    pub flags: SliceFlags,
    pub slice_keys: Vec<SliceKey>,
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct SliceFlags: Dword {
        const NINE_PATCH = 0x01;
        const PIVOT = 0x02;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SliceKey {
    pub frame_number: Dword,
    pub x: Long,
    pub y: Long,
    pub width: Dword,
    pub height: Dword,
    pub nine_patch: Option<NinePatch>,
    pub pivot: Option<Pivot>,
}

#[derive(Debug, Copy, Clone)]
pub struct NinePatch {
    pub x: Long,
    pub y: Long,
    pub width: Dword,
    pub height: Dword,
}

#[derive(Debug, Copy, Clone)]
pub struct Pivot {
    pub x: Long,
    pub y: Long,
}

pub fn parse_slice_chunk(input: &[u8]) -> ParseResult<'_, SliceChunk<'_>> {
    let (input, number_of_keys) = parse_dword_as_usize(input)?;
    let (input, flags) = dword(input)?;
    let flags = SliceFlags::from_bits_truncate(flags);
    let (input, _) = dword(input)?;
    let (input, name) = parse_string(input)?;
    let (input, slice_keys) =
        count(|input| parse_slice_key(input, flags), number_of_keys).parse(input)?;
    Ok((
        input,
        SliceChunk {
            name,
            flags,
            slice_keys,
        },
    ))
}

pub fn parse_slice_key(input: &[u8], flags: SliceFlags) -> ParseResult<'_, SliceKey> {
    let (input, frame_number) = dword(input)?;
    let (input, x) = long(input)?;
    let (input, y) = long(input)?;
    let (input, width) = dword(input)?;
    let (input, height) = dword(input)?;
    let (input, nine_patch) =
        cond(flags.contains(SliceFlags::NINE_PATCH), parse_nine_patch).parse(input)?;
    let (input, pivot) = cond(flags.contains(SliceFlags::PIVOT), parse_pivot).parse(input)?;
    Ok((
        input,
        SliceKey {
            frame_number,
            x,
            y,
            width,
            height,
            nine_patch,
            pivot,
        },
    ))
}

pub fn parse_nine_patch(input: &[u8]) -> ParseResult<'_, NinePatch> {
    let (input, x) = long(input)?;
    let (input, y) = long(input)?;
    let (input, width) = dword(input)?;
    let (input, height) = dword(input)?;
    Ok((
        input,
        NinePatch {
            x,
            y,
            width,
            height,
        },
    ))
}

pub fn parse_pivot(input: &[u8]) -> ParseResult<'_, Pivot> {
    let (input, x) = long(input)?;
    let (input, y) = long(input)?;
    Ok((input, Pivot { x, y }))
}

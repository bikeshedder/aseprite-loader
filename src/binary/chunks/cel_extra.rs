use bitflags::bitflags;

use crate::binary::{
    errors::ParseResult,
    scalars::{dword, fixed, Dword, Fixed},
};

#[derive(Debug)]
pub struct CelExtraChunk<'a> {
    pub flags: CelExtraFlags,
    pub precise_x_position: Fixed,
    pub precise_y_position: Fixed,
    pub width_of_the_cel: Fixed,
    pub height_of_the_cel: Fixed,
    pub future: &'a [u8],
}

bitflags! {
    #[derive(Debug)]
    pub struct CelExtraFlags: Dword {
        const PRECISE_BOUNDS_ARE_SET = 0x1;
    }
}

pub fn parse_cel_extra_chunk(input: &[u8]) -> ParseResult<'_, CelExtraChunk<'_>> {
    let (input, flags) = dword(input)?;
    let (input, precise_x_position) = fixed(input)?;
    let (input, precise_y_position) = fixed(input)?;
    let (input, width_of_the_cel) = fixed(input)?;
    let (input, height_of_the_cel) = fixed(input)?;
    Ok((
        &input[input.len()..],
        CelExtraChunk {
            flags: CelExtraFlags::from_bits_truncate(flags),
            precise_x_position,
            precise_y_position,
            width_of_the_cel,
            height_of_the_cel,
            future: input,
        },
    ))
}

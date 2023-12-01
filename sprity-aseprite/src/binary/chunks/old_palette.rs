use nom::multi::count;

use crate::binary::{
    errors::ParseResult,
    scalars::{byte, parse_rgb, word, Byte, RGB},
};

#[derive(Debug)]
/// Ignore this chunk if you find the new palette chunk (0x2019)
/// Aseprite v1.1 saves both chunks 0x0004 and 0x2019 just for
/// backward compatibility.
///
/// Both old palette chunks are stored in this structure as their
/// only difference is the color range. Colors in the 0x0004 version
/// of the chunk range from 0-255 while colors in the 0x0011 version
/// range from 0-63.
pub struct OldPaletteChunk {
    pub packets: Vec<Packet>,
}

#[derive(Debug)]
pub struct Packet {
    pub entries_to_skip: Byte,
    pub colors: Vec<RGB>,
}

pub fn parse_old_palette_chunk(input: &[u8]) -> ParseResult<'_, OldPaletteChunk> {
    let (input, number_of_packets) = word(input)?;
    let (input, packets) = count(parse_packet, number_of_packets.into())(input)?;
    Ok((input, OldPaletteChunk { packets }))
}

pub fn parse_packet(input: &[u8]) -> ParseResult<'_, Packet> {
    let (input, entries_to_skip) = byte(input)?;
    let (input, number_of_colors) = byte(input)?;
    let number_of_colors = if number_of_colors == 0 {
        256usize
    } else {
        number_of_colors.into()
    };
    let (input, colors) = count(parse_rgb, number_of_colors)(input)?;
    Ok((
        input,
        Packet {
            entries_to_skip,
            colors,
        },
    ))
}

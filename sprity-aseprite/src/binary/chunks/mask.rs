use nom::bytes::complete::take;

use crate::binary::{
    errors::ParseResult,
    scalars::{parse_string, short, word, Short, Word},
};

#[derive(Debug)]
pub struct MaskChunk<'a> {
    /// X position
    pub x: Short,
    /// Y position
    pub y: Short,
    /// Width
    pub width: Word,
    /// Height
    pub height: Word,
    /// Mask name
    pub name: &'a str,
    /// Bit map data (size = height*((width+7)/8))
    /// Each byte contains 8 pixels (the leftmost pixels are
    /// packed into the high order bits)
    ///
    /// Important: The length of the data is NOT checked by
    /// the parser. If you really need this you should make
    /// sure that `data.len() == height * ((width + 7) / 8`
    /// is true.
    pub data: &'a [u8],
}

pub fn parse_mask_chunk(input: &[u8]) -> ParseResult<MaskChunk<'_>> {
    let (input, x) = short(input)?;
    let (input, y) = short(input)?;
    let (input, width) = word(input)?;
    let (input, height) = word(input)?;
    let (input, _) = take(8usize)(input)?;
    let (input, name) = parse_string(input)?;
    // The parser should ensure that there are enough bytes
    // available to read. Since this is the only frame that
    // needs the `width` and `height` from the `Header` and
    // this frame is deprecated anyways this check was left
    // to the users of this library.
    //let size = height * ((width + 7) / 8);
    //let (input, data) = take(size)(input)?;
    Ok((
        &input[input.len()..],
        MaskChunk {
            x,
            y,
            width,
            height,
            name,
            data: input,
        },
    ))
}

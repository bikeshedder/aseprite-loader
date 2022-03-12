use nom::bytes::complete::{tag, take};

use super::{
    color_depth::{parse_color_depth, ColorDepth},
    errors::ParseResult,
    scalars::{byte, dword, short, word, Byte, Dword, Short, Word},
};

const HEADER_MAGIC_NUMBER: [u8; 2] = 0xA5E0u16.to_le_bytes();

/// A 128-byte header (same as FLC/FLI header, but with other magic number)
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    /// File size
    pub file_size: Dword,
    /// Amount of frames in the body of the file
    pub frames: Word,
    /// Width in pixels
    pub width: Word,
    /// Height in pixels
    pub height: Word,
    /// Color depth (bits per pixel)
    pub color_depth: ColorDepth,
    /// Flags:
    ///   1 = Layer opacity has valid value
    pub flags: Dword,
    /// Speed (milliseconds between frame, like in FLC files)
    /// DEPRECATED: You should use the frame duration field
    /// from each frame header
    #[deprecated = "You should use the frame durations instead"]
    pub speed: Word,
    /// Palette entry (index) which represent transparent color
    /// in all non-background layers (only for Indexed sprites).
    pub transparent_index: Byte,
    /// The amount of colors in the palette
    pub color_count: Word,
    /// Pixel width (pixel ratio is "pixel width/pixel height").
    /// If this or pixel height field is zero, pixel ratio is 1:1
    pub pixel_width: Byte,
    /// Pixel height
    pub pixel_height: Byte,
    /// X position of the grid
    pub grid_x: Short,
    /// Y position of the grid
    pub grid_y: Short,
    /// Grid width (zero if there is no grid, grid size
    /// is 16x16 on Aseprite by default)
    pub grid_width: Word,
    /// Grid height (zero if there is no grid)
    pub grid_height: Word,
}

pub fn parse_header(input: &[u8]) -> ParseResult<Header> {
    let (rest, input) = take(128usize)(input)?;
    let (input, file_size) = dword(input)?;
    let (input, _) = tag(HEADER_MAGIC_NUMBER)(input)?;
    let (input, frames) = word(input)?;
    let (input, width) = word(input)?;
    let (input, height) = word(input)?;
    let (input, color_depth) = parse_color_depth(input)?;
    let (input, flags) = dword(input)?;
    let (input, speed) = word(input)?;
    let (input, _) = tag([0u8; 4])(input)?;
    let (input, _) = tag([0u8; 4])(input)?;
    let (input, transparent_index) = byte(input)?;
    let (input, _) = take(3usize)(input)?;
    let (input, color_count) = word(input)?;
    let (input, pixel_width) = byte(input)?;
    let (input, pixel_height) = byte(input)?;
    let (input, grid_x) = short(input)?;
    let (input, grid_y) = short(input)?;
    let (input, grid_width) = word(input)?;
    let (input, grid_height) = word(input)?;
    let (input, future) = take(84usize)(input)?;
    // Sanity check: Did we consume all 128 bytes?
    assert_eq!(input.len(), 0);
    #[allow(deprecated)]
    Ok((
        rest,
        Header {
            file_size,
            frames,
            width,
            height,
            color_depth,
            flags,
            speed,
            transparent_index,
            color_count,
            pixel_width,
            pixel_height,
            grid_x,
            grid_y,
            grid_width,
            grid_height,
        },
    ))
}

#[test]
#[allow(deprecated)]
fn test_parse_header() {
    let input = std::fs::read("./tests/default.aseprite").unwrap();
    let (_, header) = parse_header(&input).unwrap();
    assert_eq!(
        header,
        Header {
            file_size: 573,
            frames: 1,
            width: 32,
            height: 32,
            color_depth: super::color_depth::ColorDepth::Rgba,
            flags: 1,
            speed: 100,
            transparent_index: 0,
            color_count: 32,
            pixel_width: 1,
            pixel_height: 1,
            grid_x: 0,
            grid_y: 0,
            grid_width: 16,
            grid_height: 16,
        },
    );
}

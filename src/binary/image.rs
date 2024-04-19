use super::scalars::Word;

#[derive(Clone, Debug)]
pub struct Image<'a> {
    /// Width in pixels
    pub width: Word,
    /// Height in pixels
    pub height: Word,
    /// Raw pixel data: row by row from top to bottom,
    /// for each scanline read pixels from left to right.
    /// --or--
    /// "Raw Cel" data compressed with ZLIB method (see NOTE.3)
    pub data: &'a [u8],
    /// True if the cel data is compressed
    pub compressed: bool,
}

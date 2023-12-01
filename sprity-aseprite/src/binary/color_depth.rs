use strum_macros::FromRepr;

use super::{
    errors::ParseResult,
    scalars::{word, Word},
};

/// Color depth (bits per pixel)
/// 16 bpp = Grayscale
/// 8 bpp = Indexed
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromRepr)]
pub enum ColorDepth {
    Rgba,
    Grayscale,
    Indexed,
    Unknown(Word),
}

impl ColorDepth {
    pub fn bpp(&self) -> Word {
        match self {
            Self::Rgba => 32,
            Self::Grayscale => 16,
            Self::Indexed => 8,
            Self::Unknown(bpp) => *bpp,
        }
    }
    pub fn pixel_size(&self) -> Option<usize> {
        match self {
            Self::Rgba => Some(4),
            Self::Grayscale => Some(2),
            Self::Indexed => Some(1),
            Self::Unknown(_) => None,
        }
    }
}

impl From<Word> for ColorDepth {
    fn from(bpp: Word) -> Self {
        match bpp {
            32 => ColorDepth::Rgba,
            16 => ColorDepth::Grayscale,
            8 => ColorDepth::Indexed,
            bpp => ColorDepth::Unknown(bpp),
        }
    }
}

pub fn parse_color_depth(input: &[u8]) -> ParseResult<'_, ColorDepth> {
    let (input, bpp) = word(input)?;
    Ok((input, bpp.into()))
}

#[test]
fn test_bpp() {
    assert_eq!(ColorDepth::Rgba.bpp(), 32);
    assert_eq!(ColorDepth::Grayscale.bpp(), 16);
    assert_eq!(ColorDepth::Indexed.bpp(), 8);
}

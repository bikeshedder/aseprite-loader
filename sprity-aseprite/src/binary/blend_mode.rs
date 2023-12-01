use strum_macros::FromRepr;

use super::{
    errors::ParseResult,
    scalars::{word, Word},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromRepr)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Addition,
    Subtract,
    Divide,
    Unknown(Word),
}

impl From<Word> for BlendMode {
    fn from(word: Word) -> Self {
        BlendMode::from_repr(word.into()).unwrap_or(BlendMode::Unknown(word))
    }
}

pub fn parse_blend_mode(input: &[u8]) -> ParseResult<'_, BlendMode> {
    let (input, blend_mode) = word(input)?;
    Ok((input, blend_mode.into()))
}

#[test]
fn test_parse_blend_mode() {
    assert_eq!(
        parse_blend_mode(b"\x07\x00").unwrap(),
        (&b""[..], BlendMode::ColorBurn)
    );
    assert_eq!(
        parse_blend_mode(b"\x12\x00").unwrap(),
        (&b""[..], BlendMode::Divide)
    );
    assert_eq!(
        parse_blend_mode(b"\x37\x13").unwrap(),
        (&b""[..], BlendMode::Unknown(0x1337))
    );
}

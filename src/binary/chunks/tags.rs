use std::ops::RangeInclusive;

use nom::{bytes::complete::take, multi::count, Parser};
use strum::FromRepr;

use crate::binary::{
    errors::{ParseError, ParseResult},
    scalars::{byte, parse_string, word, Byte, Word},
};

#[derive(Debug)]
pub struct TagsChunk<'a> {
    pub tags: Vec<Tag<'a>>,
}

#[allow(deprecated)]
#[derive(Debug)]
pub struct Tag<'a> {
    pub frames: RangeInclusive<Word>,
    pub animation_direction: AnimationDirection,
    pub animation_repeat: Word,
    #[deprecated]
    pub color: [u8; 3],
    pub name: &'a str,
}

#[derive(FromRepr, Debug, Copy, Clone)]
pub enum AnimationDirection {
    Forward,
    Reverse,
    PingPong,
    PingPongReverse,
    Unknown(Byte),
}

impl From<Byte> for AnimationDirection {
    fn from(byte: Byte) -> Self {
        AnimationDirection::from_repr(byte.into()).unwrap_or(AnimationDirection::Unknown(byte))
    }
}

pub fn parse_tags_chunk(input: &[u8]) -> ParseResult<'_, TagsChunk<'_>> {
    let (input, number_of_tags) = word(input)?;
    let (input, _) = take(8usize)(input)?;
    let (input, tags) = count(parse_tag, number_of_tags.into()).parse(input)?;
    Ok((input, TagsChunk { tags }))
}

pub fn parse_tag(input: &[u8]) -> ParseResult<'_, Tag<'_>> {
    let (input, from_frame) = word(input)?;
    let (input, to_frame) = word(input)?;
    if from_frame > to_frame {
        return Err(nom::Err::Failure(ParseError::InvalidFrameRange(
            from_frame, to_frame,
        )));
    }
    let (input, animation_direction) = byte(input)?;
    let animation_direction = AnimationDirection::from(animation_direction);
    let (input, animation_repeat) = word(input)?;
    let (input, _) = take(6usize)(input)?;
    let (input, color) = take(3usize)(input)?;
    let (input, _) = byte(input)?;
    let (input, name) = parse_string(input)?;
    #[allow(deprecated)]
    Ok((
        input,
        Tag {
            frames: (from_frame..=to_frame),
            animation_direction,
            animation_repeat,
            color: [color[0], color[1], color[2]],
            name,
        },
    ))
}

#[test]
fn test_tags() {
    use crate::binary::file::parse_file;
    let input = std::fs::read("./tests/tags.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.frames.len(), 1);
    assert_eq!(file.frames[0].duration, 100);
    assert_eq!(file.tags.len(), 3);
    assert_eq!(file.tags[0].name, "Tag 1");
    assert_eq!(file.tags[1].name, "Tag 2");
    assert_eq!(file.tags[2].name, "Tag 3");
}

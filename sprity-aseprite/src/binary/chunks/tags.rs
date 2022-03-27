use std::ops::Range;

use nom::{bytes::complete::take, multi::count};
use strum_macros::FromRepr;

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
    pub frames: Range<Word>,
    pub animation_direction: AnimationDirection,
    #[deprecated]
    pub color: [u8; 3],
    pub name: &'a str,
}

#[derive(FromRepr, Debug)]
pub enum AnimationDirection {
    Forward,
    Reverse,
    PingPong,
    Unknown(Byte),
}

impl From<Byte> for AnimationDirection {
    fn from(byte: Byte) -> Self {
        AnimationDirection::from_repr(byte.into()).unwrap_or(AnimationDirection::Unknown(byte))
    }
}

pub fn parse_tags_chunk(input: &[u8]) -> ParseResult<TagsChunk> {
    let (input, number_of_tags) = word(input)?;
    let (input, _) = take(8usize)(input)?;
    let (input, tags) = count(parse_tag, number_of_tags.into())(input)?;
    Ok((input, TagsChunk { tags }))
}

pub fn parse_tag(input: &[u8]) -> ParseResult<Tag> {
    let (input, from_frame) = word(input)?;
    let (input, to_frame) = word(input)?;
    if from_frame > to_frame {
        return Err(nom::Err::Failure(ParseError::InvalidFrameRange(
            from_frame, to_frame,
        )));
    }
    let (input, animation_direction) = byte(input)?;
    let animation_direction = AnimationDirection::from(animation_direction);
    let (input, _) = take(8usize)(input)?;
    let (input, color) = take(3usize)(input)?;
    let (input, _) = byte(input)?;
    let (input, name) = parse_string(input)?;
    #[allow(deprecated)]
    Ok((
        input,
        Tag {
            frames: (from_frame..to_frame + 1),
            animation_direction,
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

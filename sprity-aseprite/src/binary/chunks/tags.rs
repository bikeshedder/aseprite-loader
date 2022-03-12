use nom::{
    bytes::complete::take,
    combinator::{flat_map, map},
    multi::count,
};
use strum_macros::FromRepr;

use crate::binary::{
    errors::ParseResult,
    scalars::{byte, parse_string, word, Byte, Word},
};

#[derive(Debug)]
pub struct TagsChunk {
    tags: Vec<Tag>,
}

#[allow(deprecated)]
#[derive(Debug)]
pub struct Tag {
    from_frame: Word,
    to_frame: Word,
    animation_direction: AnimationDirection,
    #[deprecated]
    color: [u8; 3],
    name: String,
}

#[derive(FromRepr, Debug)]
enum AnimationDirection {
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
            from_frame,
            to_frame,
            animation_direction,
            color: [color[0], color[1], color[2]],
            name,
        },
    ))
}

#[test]
fn test_tags() {
    use crate::binary::{chunk::Chunk, file::parse_file};
    let input = std::fs::read("./tests/tags.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.frames.len(), 1);
    assert_eq!(file.frames[0].duration, 100);
    let tags = file.frames[0]
        .chunks
        .iter()
        .filter_map(|chunk| match chunk {
            Chunk::Tags(layer) => Some(layer),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].tags.len(), 3);
    assert_eq!(tags[0].tags[0].name, "Tag 1");
    assert_eq!(tags[0].tags[1].name, "Tag 2");
    assert_eq!(tags[0].tags[2].name, "Tag 3");
}

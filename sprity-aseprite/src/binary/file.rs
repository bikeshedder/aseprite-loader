use super::{
    errors::{ParseError, ParseResult},
    frame::{parse_frames, Frame},
    header::{parse_header, Header},
};

pub struct File {
    header: Header,
    frames: Vec<Frame>,
}

pub fn parse_file(input: &[u8]) -> Result<File, nom::Err<ParseError>> {
    let (input, header) = parse_header(input)?;
    let (_, frames) = parse_frames(input)?;
    Ok(File { header, frames })
}

#[test]
fn test_parse_file() {
    let input = std::fs::read("./tests/default.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.frames.len(), 1);
    assert_eq!(file.frames[0].duration, 100);
}

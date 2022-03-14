use super::{
    errors::{ParseError, ParseResult},
    frame::{parse_frames, Frame},
    header::{parse_header, Header},
};

#[derive(Debug)]
pub struct File<'a> {
    pub header: Header,
    pub frames: Vec<Frame<'a>>,
}

pub fn parse_file<'a>(input: &'a [u8]) -> Result<File<'a>, nom::Err<ParseError<'a>>> {
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

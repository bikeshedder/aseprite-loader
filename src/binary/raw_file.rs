use super::{
    errors::ParseError,
    header::{parse_header, Header},
    raw_frame::{parse_frames, RawFrame},
};

#[derive(Debug)]
pub struct RawFile<'a> {
    pub header: Header,
    pub frames: Vec<RawFrame<'a>>,
}

pub fn parse_raw_file(input: &[u8]) -> Result<RawFile<'_>, nom::Err<ParseError<'_>>> {
    let (input, header) = parse_header(input)?;
    let (_, frames) = parse_frames(input)?;
    Ok(RawFile { header, frames })
}

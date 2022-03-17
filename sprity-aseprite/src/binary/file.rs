use super::{
    chunk::Chunk,
    chunks::{
        cel::CelChunk,
        layer::{LayerChunk, LayerType},
        tags::Tag,
    },
    errors::ParseError,
    frame::{parse_frames, Frame},
    header::{parse_header, Header},
};

#[derive(Debug)]
pub struct File<'a> {
    pub header: Header,
    pub frames: Vec<Frame<'a>>,
}

impl<'a> File<'a> {
    pub fn normal_layers(&self) -> impl Iterator<Item = &LayerChunk> {
        self.frames.iter().flat_map(|frame| {
            frame.chunks.iter().filter_map(|chunk| match chunk {
                Chunk::Layer(chunk) if chunk.layer_type == LayerType::Normal => Some(chunk),
                _ => None,
            })
        })
    }
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.frames
            .iter()
            .flat_map(|frame| {
                frame.chunks.iter().filter_map(|chunk| match chunk {
                    Chunk::Tags(chunk) => Some(&chunk.tags),
                    _ => None,
                })
            })
            .flat_map(|tags| tags.iter())
    }
    pub fn image_cels(&self) -> impl Iterator<Item = (usize, &CelChunk)> {
        self.frames
            .iter()
            .enumerate()
            .flat_map(|(frame_index, frame)| frame.image_cels().map(move |cel| (frame_index, cel)))
    }
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

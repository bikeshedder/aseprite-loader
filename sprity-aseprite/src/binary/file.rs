use super::{
    chunk::Chunk,
    chunks::{
        cel::ImageCel,
        layer::{LayerChunk, LayerType},
        tags::Tag,
    },
    color_depth::ColorDepth,
    errors::ParseError,
    frame::{parse_frames, Frame},
    header::{parse_header, Header},
    palette::{create_palette, Palette},
};

#[derive(Debug)]
pub struct File<'a> {
    pub header: Header,
    pub frames: Vec<Frame<'a>>,
    pub palette: Option<Palette>,
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
    pub fn image_cels(&self) -> impl Iterator<Item = (usize, ImageCel)> {
        self.frames
            .iter()
            .enumerate()
            .flat_map(|(frame_index, frame)| frame.image_cels().map(move |cel| (frame_index, cel)))
    }
}

pub fn parse_file(input: &[u8]) -> Result<File, nom::Err<ParseError>> {
    let (input, header) = parse_header(input)?;
    let (_, frames) = parse_frames(input)?;
    let palette = match header.color_depth {
        ColorDepth::Indexed => Some(
            create_palette(&header, &frames)
                .map_err(|e| nom::Err::Failure(ParseError::PaletteError(e)))?,
        ),
        _ => None,
    };
    Ok(File {
        header,
        frames,
        palette,
    })
}

#[test]
fn test_parse_file() {
    let input = std::fs::read("./tests/default.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.frames.len(), 1);
    assert_eq!(file.frames[0].duration, 100);
}

#[test]
fn test_palette() {
    use sprity_core::Color;
    let input = std::fs::read("./tests/indexed.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.header.color_depth, ColorDepth::Indexed);
    let palette = file.palette.unwrap();
    assert_eq!(
        palette.colors[27],
        Color {
            red: 172,
            green: 50,
            blue: 50,
            alpha: 255
        }
    );
    assert_eq!(
        palette.colors[10],
        Color {
            red: 106,
            green: 190,
            blue: 48,
            alpha: 255
        }
    );
    assert_eq!(
        palette.colors[17],
        Color {
            red: 91,
            green: 110,
            blue: 225,
            alpha: 255
        }
    );
}

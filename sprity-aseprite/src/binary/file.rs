use itertools::Itertools;

use super::{
    chunk::Chunk,
    chunks::{cel::CelChunk, layer::LayerChunk, tags::Tag},
    color_depth::ColorDepth,
    errors::ParseError,
    frame::Frame,
    header::Header,
    palette::{create_palette, Palette},
    raw_file::parse_raw_file,
    scalars::Word,
};

#[derive(Debug)]
pub struct File<'a> {
    pub header: Header,
    pub palette: Option<Palette>,
    pub layers: Vec<LayerChunk<'a>>,
    pub frames: Vec<Frame<'a>>,
    pub tags: Vec<Tag<'a>>,
}

pub fn parse_file(input: &[u8]) -> Result<File<'_>, nom::Err<ParseError<'_>>> {
    let raw_file = parse_raw_file(input)?;
    let palette = match raw_file.header.color_depth {
        ColorDepth::Indexed => Some(
            create_palette(&raw_file.header, &raw_file.frames)
                .map_err(|e| nom::Err::Failure(ParseError::PaletteError(e)))?,
        ),
        _ => None,
    };
    let mut frames = Vec::<(Word, Vec<CelChunk<'_>>)>::new();
    let mut layers = Vec::<LayerChunk<'_>>::new();
    let mut tags = Vec::<Tag<'_>>::new();
    for raw_frame in raw_file.frames {
        let mut cels = Vec::<CelChunk<'_>>::new();
        for chunk in raw_frame.chunks {
            match chunk {
                Chunk::Palette0004(_) => {}
                Chunk::Palette0011(_) => {}
                Chunk::Layer(layer) => layers.push(layer),
                Chunk::Cel(cel) => cels.push(cel),
                Chunk::CelExtra(_) => {}
                Chunk::ColorProfile(_) => {}
                Chunk::ExternalFiles(_) => {}
                Chunk::Mask(_) => {}
                Chunk::Path => {}
                Chunk::Tags(tags_chunk) => tags.extend(tags_chunk.tags),
                Chunk::Palette(_) => {}
                Chunk::UserData(_) => {}
                Chunk::Slice(_) => {}
                Chunk::Tileset(_) => {}
                Chunk::Unsupported(_) => {}
            }
        }
        frames.push((raw_frame.duration, cels));
    }
    let frames = frames
        .into_iter()
        .map(|(duration, frame_cels)| {
            Ok(Frame {
                duration,
                cels: {
                    // Insert cels in the cels vector so that a direct lookup
                    // by layer index is possible.
                    let mut cels: Vec<Option<CelChunk<'_>>> = Vec::with_capacity(layers.len());
                    for _ in 0..layers.len() {
                        cels.push(None);
                    }
                    for cel in frame_cels {
                        let layer_index: usize = cel.layer_index.into();
                        if layer_index > layers.len() {
                            return Err(nom::Err::Failure(ParseError::LayerIndexOutOfBounds));
                        }
                        cels[layer_index] = Some(cel);
                    }
                    cels
                },
            })
        })
        .try_collect()?;
    Ok(File {
        header: raw_file.header,
        palette,
        layers,
        frames,
        tags,
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
    use crate::binary::scalars::Color;
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

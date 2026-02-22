use std::iter::Peekable;

use itertools::Itertools;

use crate::binary::chunks::user_data::UserDataChunk;

use super::{
    chunk::Chunk,
    chunks::{cel::CelChunk, layer::LayerChunk, slice::SliceChunk, tags::Tag},
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
    pub slices: Vec<SliceChunk<'a>>,
    /// Optional user data associated with the sprite
    pub user_data: Option<UserDataChunk<'a>>,
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
    let mut slices = Vec::<SliceChunk<'_>>::new();
    let mut user_data = None;
    for raw_frame in raw_file.frames {
        let mut cels = Vec::<CelChunk<'_>>::new();
        let mut chunks = raw_frame.chunks.into_iter().peekable();
        while let Some(chunk) = chunks.next() {
            match chunk {
                // In Aseprite v1.3 a sprite has associated user data, to consider this case there
                // is an User Data Chunk at the first frame after the Palette Chunk.
                Chunk::Palette0004(_) | Chunk::Palette0011(_) | Chunk::Palette(_) => {
                    if frames.is_empty() && user_data.is_none() {
                        user_data = next_user_data(&mut chunks);
                    }
                }
                Chunk::Layer(layer) => layers.push(LayerChunk {
                    user_data: next_user_data(&mut chunks),
                    ..layer
                }),
                Chunk::Cel(cel) => cels.push(CelChunk {
                    user_data: next_user_data(&mut chunks),
                    ..cel
                }),
                Chunk::CelExtra(_) => {}
                Chunk::ColorProfile(_) => {}
                Chunk::ExternalFiles(_) => {}
                Chunk::Mask(_) => {}
                Chunk::Path => {}
                Chunk::Tags(mut tags_chunk) => {
                    // After a Tags chunk, there will be several user data chunks, one for each
                    // tag, you should associate the user data in the same order as the tags
                    // are in the Tags chunk
                    for tag in &mut tags_chunk.tags {
                        tag.user_data = next_user_data(&mut chunks);
                        if tag.user_data.is_none() {
                            break;
                        }
                    }
                    tags.extend(tags_chunk.tags)
                }
                Chunk::UserData(_) => {}
                Chunk::Slice(slice) => slices.push(SliceChunk {
                    user_data: next_user_data(&mut chunks),
                    ..slice
                }),
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
        slices,
        user_data,
    })
}

/// Peeks ahead to see if the next chunk is a [`UserDataChunk`]. If it is, then the iterator is
/// advanced, and the owned user data is returned.
fn next_user_data<'a>(
    iter: &mut Peekable<impl Iterator<Item = Chunk<'a>>>,
) -> Option<UserDataChunk<'a>> {
    let chunk = iter.next_if(|c| matches!(c, Chunk::UserData(_)))?;
    let Chunk::UserData(user_data) = chunk else {
        unreachable!()
    };
    Some(user_data)
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

#[test]
fn test_user_data() {
    let input = std::fs::read("./tests/user_data.aseprite").unwrap();
    let file = parse_file(&input).unwrap();
    assert_eq!(file.user_data.unwrap().text.unwrap(), "sprite_data");
    assert_eq!(
        file.layers[0].user_data.as_ref().unwrap().text.unwrap(),
        "layer_data"
    );
    assert_eq!(
        file.slices[0].user_data.as_ref().unwrap().text.unwrap(),
        "slice_data"
    );
    assert_eq!(
        file.frames[0].cels[0]
            .as_ref()
            .unwrap()
            .user_data
            .as_ref()
            .unwrap()
            .text
            .unwrap(),
        "cel_data"
    );
    for tag in file.tags {
        let user_text = tag.user_data.unwrap().text.unwrap();
        match tag.name {
            "Tag 1" => assert_eq!(user_text, "tag_data_1"),
            "Tag 2" => assert_eq!(user_text, "tag_data_2"),
            _ => {}
        };
    }
}

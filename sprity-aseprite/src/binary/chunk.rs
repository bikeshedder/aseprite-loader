use nom::{bytes::complete::take, multi::count};

use super::{
    chunk_type::{parse_chunk_type, ChunkType},
    chunks::{
        cel::{parse_cel_chunk, CelChunk},
        cel_extra::{parse_cel_extra_chunk, CelExtraChunk},
        color_profile::{parse_color_profile, ColorProfileChunk},
        external_files::{parse_external_files_chunk, ExternalFilesChunk},
        layer::{parse_layer_chunk, LayerChunk},
        mask::{parse_mask_chunk, MaskChunk},
        old_palette::{parse_old_palette_chunk, OldPaletteChunk},
        palette::{parse_palette_chunk, PaletteChunk},
        slice::{parse_slice_chunk, SliceChunk},
        tags::{parse_tags_chunk, TagsChunk},
        tileset::{parse_tileset_chunk, TilesetChunk},
        user_data::{parse_user_data_chunk, UserDataChunk},
    },
    errors::{ParseError, ParseResult},
    scalars::dword_size,
};

#[derive(Debug)]
pub enum Chunk<'a> {
    Palette0004(OldPaletteChunk),
    Palette0011(OldPaletteChunk),
    Layer(LayerChunk<'a>),
    Cel(CelChunk<'a>),
    CelExtra(CelExtraChunk<'a>),
    ColorProfile(ColorProfileChunk<'a>),
    ExternalFiles(ExternalFilesChunk<'a>),
    Mask(MaskChunk<'a>),
    Path,
    Tags(TagsChunk<'a>),
    Palette(PaletteChunk<'a>),
    UserData(UserDataChunk<'a>),
    Slice(SliceChunk<'a>),
    Tileset(TilesetChunk<'a>),
    Unsupported(u16),
}

pub fn parse_chunks(input: &[u8], chunk_count: usize) -> ParseResult<'_, Vec<Chunk<'_>>> {
    count(parse_chunk, chunk_count)(input)
}

pub fn parse_chunk(input: &[u8]) -> ParseResult<'_, Chunk<'_>> {
    let (input, size) = dword_size(input, ParseError::InvalidFrameSize)?;
    let (rest, input) = take(size - 4)(input)?;
    let (chunk_data, chunk_type) = parse_chunk_type(input)?;
    let chunk = match chunk_type {
        Ok(ChunkType::Palette0004) => Chunk::Palette0004(parse_old_palette_chunk(chunk_data)?.1),
        Ok(ChunkType::Palette0011) => Chunk::Palette0011(parse_old_palette_chunk(chunk_data)?.1),
        Ok(ChunkType::Layer) => Chunk::Layer(parse_layer_chunk(chunk_data)?.1),
        Ok(ChunkType::Cel) => Chunk::Cel(parse_cel_chunk(chunk_data)?.1),
        Ok(ChunkType::CelExtra) => Chunk::CelExtra(parse_cel_extra_chunk(chunk_data)?.1),
        Ok(ChunkType::ColorProfile) => Chunk::ColorProfile(parse_color_profile(chunk_data)?.1),
        Ok(ChunkType::ExternalFile) => {
            Chunk::ExternalFiles(parse_external_files_chunk(chunk_data)?.1)
        }
        Ok(ChunkType::Mask) => Chunk::Mask(parse_mask_chunk(chunk_data)?.1),
        Ok(ChunkType::Path) => Chunk::Path,
        Ok(ChunkType::Tags) => Chunk::Tags(parse_tags_chunk(chunk_data)?.1),
        Ok(ChunkType::Palette) => Chunk::Palette(parse_palette_chunk(chunk_data)?.1),
        Ok(ChunkType::UserData) => Chunk::UserData(parse_user_data_chunk(chunk_data)?.1),
        Ok(ChunkType::Slice) => Chunk::Slice(parse_slice_chunk(chunk_data)?.1),
        Ok(ChunkType::Tileset) => Chunk::Tileset(parse_tileset_chunk(chunk_data)?.1),
        Err(chunk_type) => Chunk::Unsupported(chunk_type),
    };
    Ok((rest, chunk))
}

use bitflags::bitflags;
use nom::{
    bytes::complete::take,
    combinator::{cond, flat_map},
};

use crate::binary::{
    errors::ParseResult,
    scalars::{dword, parse_string, short, word, Dword, Short, Word},
};

#[derive(Debug)]
pub struct TilesetChunk<'a> {
    /// Tileset ID
    pub id: Dword,
    /// Tileset flags
    pub flags: TilesetFlags,
    /// Number of tiles
    pub number_of_tiles: Dword,
    /// Tile Width
    pub width: Word,
    /// Tile Height
    pub height: Word,
    /// Base Index: Number to show in the screen from the tile with
    /// index 1 and so on (by default this is field is 1, so the data
    /// that is displayed is equivalent to the data in memory). But it
    /// can be 0 to display zero-based indexing (this field isn't used
    /// for the representation of the data in the file, it's just for
    /// UI purposes).
    pub base_index: Short,
    /// Name of the tileset
    pub name: &'a str,
    /// Link to external file
    pub external_file: Option<TilesetExternalFile>,
    /// Tiles inside this file
    pub tiles: Option<TilesetTiles<'a>>,
}

#[derive(Debug, Copy, Clone)]
pub struct TilesetExternalFile {
    /// ID of the external file. This ID is one entry
    /// of the the External Files Chunk.
    pub external_file_id: Dword,
    /// Tileset ID in the external file
    pub tileset_id: Dword,
}

#[derive(Debug)]
pub struct TilesetTiles<'a> {
    /// Compressed Tileset image (see NOTE.3):
    /// (Tile Width) x (Tile Height x Number of Tiles)
    pub data: &'a [u8],
}

bitflags! {
    #[derive(Debug)]
    pub struct TilesetFlags: Dword {
        /// 1 - Include link to external file
        const EXTERNAL_FILE = 1;
        const TILES = 2;
        const TILE_0_EMPTY = 4;
    }
}

pub fn parse_tileset_chunk(input: &[u8]) -> ParseResult<'_, TilesetChunk<'_>> {
    let (input, id) = dword(input)?;
    let (input, flags) = dword(input)?;
    let flags = TilesetFlags::from_bits_truncate(flags);
    let (input, number_of_tiles) = dword(input)?;
    let (input, width) = word(input)?;
    let (input, height) = word(input)?;
    let (input, base_index) = short(input)?;
    let (input, _) = take(14usize)(input)?;
    let (input, name) = parse_string(input)?;
    let (input, external_file) = cond(
        flags.contains(TilesetFlags::EXTERNAL_FILE),
        parse_external_file,
    )(input)?;
    let (input, tiles) = cond(flags.contains(TilesetFlags::TILES), parse_tiles)(input)?;
    Ok((
        input,
        TilesetChunk {
            id,
            flags,
            number_of_tiles,
            width,
            height,
            base_index,
            name,
            external_file,
            tiles,
        },
    ))
}

pub fn parse_external_file(input: &[u8]) -> ParseResult<'_, TilesetExternalFile> {
    let (input, external_file_id) = dword(input)?;
    let (input, tileset_id) = dword(input)?;
    Ok((
        input,
        TilesetExternalFile {
            external_file_id,
            tileset_id,
        },
    ))
}

use nom::combinator::map;

pub fn parse_tiles(input: &[u8]) -> ParseResult<'_, TilesetTiles<'_>> {
    map(flat_map(dword, take), |data| TilesetTiles { data })(input)
}

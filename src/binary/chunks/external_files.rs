use nom::{bytes::complete::take, multi::count};
use strum::FromRepr;

use crate::binary::{
    errors::ParseResult,
    scalars::{byte, dword, parse_dword_as_usize, parse_string, Byte, Dword},
};

/// A list of external files linked with this file. It might be used to
/// reference external palettes or tilesets.
#[derive(Debug)]
pub struct ExternalFilesChunk<'a> {
    pub files: Vec<ExternalFile<'a>>,
}

#[derive(Debug)]
pub struct ExternalFile<'a> {
    /// Entry ID (this ID is referenced by tilesets or palettes)
    pub entry_id: Dword,
    /// Type
    pub file_type: ExternalFileType,
    /// External file name
    pub file_name: &'a str,
}

#[derive(FromRepr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ExternalFileType {
    ExternalPalette,
    ExternalTileset,
    ExtensionNameForProperties,
    Unknown(Byte),
}

impl From<Byte> for ExternalFileType {
    fn from(byte: Byte) -> Self {
        Self::from_repr(byte).unwrap_or(Self::Unknown(byte))
    }
}

pub fn parse_external_files_chunk(input: &[u8]) -> ParseResult<'_, ExternalFilesChunk<'_>> {
    let (input, number_of_entries) = parse_dword_as_usize(input)?;
    let (input, _) = take(8usize)(input)?;
    let (input, files) = count(parse_external_file, number_of_entries)(input)?;
    Ok((input, ExternalFilesChunk { files }))
}

pub fn parse_external_file(input: &[u8]) -> ParseResult<'_, ExternalFile<'_>> {
    let (input, entry_id) = dword(input)?;
    let (input, file_type) = byte(input)?;
    let file_type = ExternalFileType::from(file_type);
    let (input, _) = take(7usize)(input)?;
    let (input, file_name) = parse_string(input)?;
    Ok((
        input,
        ExternalFile {
            entry_id,
            file_type,
            file_name,
        },
    ))
}

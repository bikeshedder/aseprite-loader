use nom::{bytes::complete::take, multi::count};

use crate::binary::{
    errors::ParseResult,
    scalars::{dword, parse_dword_as_usize, parse_string, Dword},
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
    /// External file name
    pub file_name: &'a str,
}

pub fn parse_external_files_chunk(input: &[u8]) -> ParseResult<'_, ExternalFilesChunk<'_>> {
    let (input, number_of_entries) = parse_dword_as_usize(input)?;
    let (input, _) = take(8usize)(input)?;
    let (input, files) = count(parse_external_file, number_of_entries)(input)?;
    Ok((input, ExternalFilesChunk { files }))
}

pub fn parse_external_file(input: &[u8]) -> ParseResult<'_, ExternalFile<'_>> {
    let (input, entry_id) = dword(input)?;
    let (input, file_name) = parse_string(input)?;
    Ok((
        input,
        ExternalFile {
            entry_id,
            file_name,
        },
    ))
}

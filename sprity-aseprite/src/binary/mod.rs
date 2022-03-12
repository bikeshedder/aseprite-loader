//! This module contains a parser for the .aseprite file specification in
//! version 1.3 as described in the `aseprite/aseprite` repository on GitHub:
//! https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md

use nom::{
    number::complete::{le_i16, le_i32, le_u16, le_u32, le_u8},
    IResult,
};

pub mod blend_mode;
pub mod chunk;
pub mod chunk_type;
pub mod color_depth;
pub mod errors;
pub mod file;
pub mod frame;
pub mod header;
pub mod scalars;

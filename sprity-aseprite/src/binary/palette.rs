use thiserror::Error;

pub use sprity_core::Color;

use super::{
    chunk::Chunk,
    chunks::{old_palette::OldPaletteChunk, palette::PaletteChunk},
    frame::Frame,
    header::Header,
};

#[derive(Debug)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            colors: [Color::default(); 256],
        }
    }
}

pub fn create_palette(header: &Header, frames: &[Frame]) -> Result<Palette, PaletteError> {
    let mut palette = Palette::default();
    let palette_chunks: Vec<_> = frames
        .iter()
        .flat_map(|frame| {
            frame.chunks.iter().filter_map(|chunk| {
                if let Chunk::Palette(palette) = chunk {
                    Some(palette)
                } else {
                    None
                }
            })
        })
        .collect();
    if !palette_chunks.is_empty() {
        process_palette_chunks(header.transparent_index, &palette_chunks, &mut palette)?;
        return Ok(palette);
    }
    let palette0004_chunks: Vec<_> = frames
        .iter()
        .flat_map(|frame| {
            frame.chunks.iter().filter_map(|chunk| {
                if let Chunk::Palette0004(palette) = chunk {
                    Some(palette)
                } else {
                    None
                }
            })
        })
        .collect();
    if !palette0004_chunks.is_empty() {
        process_old_palette_chunks(header.transparent_index, &palette0004_chunks, &mut palette)?;
        return Ok(palette);
    }
    let palette0011_chunks: Vec<_> = frames
        .iter()
        .flat_map(|frame| {
            frame.chunks.iter().filter_map(|chunk| {
                if let Chunk::Palette0011(palette) = chunk {
                    Some(palette)
                } else {
                    None
                }
            })
        })
        .collect();
    if !palette0011_chunks.is_empty() {
        process_old_palette_chunks(header.transparent_index, &palette0011_chunks, &mut palette)?;
        return Ok(palette);
    }
    Err(PaletteError::Missing)
}

fn process_palette_chunks(
    transparent_index: u8,
    chunks: &[&PaletteChunk],
    palette: &mut Palette,
) -> Result<(), PaletteError> {
    let mut ok = false;
    for chunk in chunks.iter() {
        // The aseprite palette chunk is a bit weird. Both the palette size
        // and the color indices use `DWORD` (u32) as data type. Indexed
        // colors use `BYTE` (u8) though.
        for (entry, color_idx) in chunk.entries.iter().zip(chunk.indices.clone()) {
            palette.colors[usize::from(color_idx)] = entry.color;
            ok = true;
        }
    }
    if ok {
        palette.colors[usize::from(transparent_index)].alpha = 0;
        Ok(())
    } else {
        Err(PaletteError::Empty)
    }
}

fn process_old_palette_chunks(
    transparent_index: u8,
    chunks: &[&OldPaletteChunk],
    palette: &mut Palette,
) -> Result<(), PaletteError> {
    let mut ok = false;
    for chunk in chunks.iter() {
        let mut color_idx = 0usize;
        for packet in &chunk.packets {
            color_idx += usize::from(packet.entries_to_skip);
            if color_idx + packet.colors.len() > 256 {
                return Err(PaletteError::IndexOutOfBounds);
            }
            for color in &packet.colors {
                palette.colors[color_idx] = Color {
                    red: color.red,
                    green: color.green,
                    blue: color.blue,
                    alpha: 255,
                };
                color_idx += 1;
                ok = true;
            }
        }
    }
    if ok {
        palette.colors[usize::from(transparent_index)].alpha = 0;
        Ok(())
    } else {
        Err(PaletteError::Empty)
    }
}

#[derive(Debug, Error)]
pub enum PaletteError {
    #[error("Palette is missing")]
    Missing,
    #[error("Palette is empty")]
    Empty,
    #[error("First color index not in range 0..255")]
    FirstColorIndexOutOfBounds,
    #[error("Last color index not in range 0..255")]
    LastColorIndexOutOfBounds,
    #[error("First color index > last color index")]
    FirstColorIndexGreaterThanLastColorIndex,
    #[error("Palette index out of bounds")]
    IndexOutOfBounds,
}

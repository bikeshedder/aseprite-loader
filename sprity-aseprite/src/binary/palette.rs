pub use sprity_core::Palette;

use super::{chunk::Chunk, frame::Frame};

pub fn create_palette(frames: &[Frame]) -> Palette {
    let mut palette = Palette::default();
    for chunk in frames.iter().flat_map(|frame| {
        frame.chunks.iter().filter_map(|chunk| {
            if let Chunk::Palette(palette) = chunk {
                Some(palette)
            } else {
                None
            }
        })
    }) {
        // The aseprite palette chunk is a bit weird. Both the palette size
        // and the color indices use `DWORD` (u32) as data type. Indexed
        // colors use `BYTE` (u8) though. The following code just assumes
        for (entry, color_idx) in chunk.entries.iter().zip(chunk.indices.clone()) {
            palette.colors[usize::from(color_idx)] = entry.color;
        }
    }
    palette
}

pub enum PaletteError {
    IndexOutOfRange,
}

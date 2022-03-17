use crate::Color;

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

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub enum Format {
    // XXX both greyscale and indexed use this :thinking:
    I8,
    RGBA8888,
}

#[derive(Debug, Deserialize)]
pub enum BlendMode {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "darken")]
    Darken,
    #[serde(rename = "multiply")]
    Multiply,
    #[serde(rename = "color_burn")]
    ColorBurn,
    #[serde(rename = "lighten")]
    Lighten,
    #[serde(rename = "screen")]
    Screen,
    #[serde(rename = "color_dodge")]
    ColorDodge,
    #[serde(rename = "addition")]
    Addition,
    #[serde(rename = "overlay")]
    Overlay,
    #[serde(rename = "soft_light")]
    SoftLight,
    #[serde(rename = "hard_light")]
    HardLight,
    #[serde(rename = "difference")]
    Difference,
    #[serde(rename = "exclusion")]
    Exclusion,
    #[serde(rename = "subtract")]
    Subtract,
    #[serde(rename = "divide")]
    Divide,
    #[serde(rename = "hue")]
    Hue,
    #[serde(rename = "saturation")]
    Saturation,
    #[serde(rename = "color")]
    Color,
    #[serde(rename = "luminosity")]
    Luminosity,
}

#[derive(Debug, Deserialize)]
pub enum Direction {
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "reverse")]
    Reverse,
    #[serde(rename = "pingpong")]
    PingPong,
}

#[derive(Debug, Deserialize)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize)]
pub struct Rect {
    #[serde(flatten)]
    pub size: Size,
    #[serde(flatten)]
    pub point: Point,
}

#[derive(Debug, Deserialize)]
pub struct Layer {
    pub name: String,
    pub opacity: Option<u8>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub color: Option<String>,
    pub data: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FrameTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub direction: Direction,
}

#[derive(Debug, Deserialize)]
pub struct SliceKey {
    pub frame: usize,
    pub bounds: Rect,
    pub center: Rect,
    pub pivot: Rect,
}

#[derive(Debug, Deserialize)]
pub struct Slice {
    pub name: String,
    pub color: String,
    pub keys: Vec<SliceKey>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: Format,
    pub size: Size,
    pub scale: String,
    #[serde(rename = "frameTags")]
    pub frame_tags: Option<Vec<FrameTag>>,
    pub layers: Option<Vec<Layer>>,
    pub slices: Option<Vec<Slice>>,
}

#[derive(Debug, Deserialize)]
pub struct Frame {
    pub filename: Option<String>,
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    pub sprite_source_size: Rect,
    #[serde(rename = "sourceSize")]
    pub source_size: Size,
    pub duration: u16,
}

#[derive(Debug, Deserialize)]
pub struct Aseprite {
    pub frames: HashMap<String, Frame>,
    pub meta: Meta,
}

impl Aseprite {
    pub fn load(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

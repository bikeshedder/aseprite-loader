use sprity_aseprite::binary::loader::BinaryLoader;
use sprity_core::{Loader, SpriteSheetMeta};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = BinaryLoader {};
    let dir = "../examples/assets";
    let meta = loader.load_dir_meta(&dir)?;
    let dyn_meta: Vec<&dyn SpriteSheetMeta> =
        meta.iter().map(|s| s as &dyn SpriteSheetMeta).collect();
    let sheets = loader.load_dir(&dir, &dyn_meta)?;
    sheets[0].image.save("out/atlas.png")?;
    Ok(())
}

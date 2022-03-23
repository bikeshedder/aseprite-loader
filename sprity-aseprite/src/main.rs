use image::RgbaImage;
use sprity_aseprite::binary::loader::BinaryLoader;
use sprity_core::{Loader, SpriteSheetMeta};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = BinaryLoader {};
    let dir = "examples/assets";
    let meta = loader.load_dir_meta(&dir)?;
    let dyn_meta: Vec<&dyn SpriteSheetMeta> =
        meta.iter().map(|s| s as &dyn SpriteSheetMeta).collect();
    let files = loader.list_dir(&dir, &dyn_meta)?;
    let data = std::fs::read(&files[0])?;
    let sprite = loader.load_sprite(&data, dyn_meta[0])?;
    for (i, image) in sprite.images().enumerate() {
        let mut buf = vec![0u8; image.bytes()];
        let slice = image.load(&mut buf)?;
        let size = image.size();
        let image = RgbaImage::from_raw(size.0.into(), size.1.into(), buf).unwrap();
        image.save(format!("output/{i}.png"))?;
    }
    //sprite.image.save("out/atlas.png")?;
    Ok(())
}

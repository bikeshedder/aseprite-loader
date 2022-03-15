use sprity_aseprite::binary::loader::BinaryLoader;
use sprity_core::Loader;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = BinaryLoader {};
    let sprite = loader.load_dir(&"../examples/assets")?;
    println!("{:?}", sprite);
    Ok(())
}

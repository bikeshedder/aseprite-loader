use sprity_aseprite::binary::loader::BinaryLoader;
use sprity_core::Loader;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = BinaryLoader {};
    let meta = loader.load_dir_meta(&"../examples/assets")?;
    println!("{:#?}", meta);
    Ok(())
}

use sprity_aseprite::binary::loader::BinaryLoader;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = BinaryLoader {};
    let assert_dir = "../examples/assets";
    let code = sprity_codegen::aseprite_dir(&loader, &assert_dir)?;
    println!("{}", code);
    Ok(())
}

use clap::{Parser, Subcommand};

use sprity_aseprite::binary::loader::BinaryLoader;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Clones repos
    #[command()]
    Gen { asset_dir: String },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let loader = BinaryLoader {};
    match args.command {
        Commands::Gen { asset_dir } => {
            let code = sprity_codegen::aseprite_dir(&loader, &asset_dir)?;
            println!("{}", code);
        }
    }
    Ok(())
}

use std::fs::read_dir;
use std::path::Path;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::AsepriteFile;

static ASEPRITE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

pub fn aseprite_dir(dir: &Path) -> TokenStream {
    let mut files = read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {}", e, dir.to_string_lossy()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_stem()?;
            let ext = path.extension()?;
            if ASEPRITE_EXTENSIONS.contains(&ext.to_str()?.to_lowercase().as_ref()) {
                Some(AsepriteFile::load(name.to_str()?, &path))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        panic!(
            "Directory does not contain any aseprite files: {}",
            dir.to_string_lossy()
        );
    }

    files.sort();

    let sprite_structs = files.iter().map(gen_sprite).collect::<Vec<_>>();
    let sprite_enum = gen_enum(&files);

    quote! {
        #( #sprite_structs )*
        #sprite_enum
    }
}

fn gen_enum(files: &[AsepriteFile]) -> TokenStream {
    let sprite_idents = files
        .iter()
        .map(|f| format_ident!("{}", f.name))
        .collect::<Vec<_>>();
    let sprite_names = files.iter().map(|f| &f.name).collect::<Vec<_>>();
    let enum_variants = files.iter().map(gen_enum_variant).collect::<Vec<_>>();

    quote! {
        pub enum Sprite {
            #( #enum_variants ),*
        }
        impl ::sprity_core::Sprite for Sprite {
            fn name(&self) -> &'static str {
                match self {
                    #(
                        Self :: #sprite_idents (..) => #sprite_names
                    ),*
                }
            }
        }
    }
}

fn gen_enum_variant(file: &AsepriteFile) -> TokenStream {
    let ident = format_ident!("{}", file.name);
    quote! {
        #ident ( #ident )
    }
}

fn gen_sprite(file: &AsepriteFile) -> TokenStream {
    let ident = format_ident!("{}", file.name);
    let name = &file.name;
    quote! {
        pub struct #ident {}
        impl #ident {
            fn sprite(self) -> Sprite {
                Sprite :: #ident (self)
            }
        }
        impl ::sprity_core::Sprite for #ident {
            fn name(&self) -> &'static str {
                #name
            }
        }
    }
}

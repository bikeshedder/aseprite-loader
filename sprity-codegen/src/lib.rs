use std::path::Path;

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use sprity_core::SpriteMeta;

pub fn aseprite_dir(loader: &dyn sprity_core::Loader, dir: &dyn AsRef<Path>) -> TokenStream {
    let files = loader.load_dir_meta(dir).unwrap();

    if files.is_empty() {
        panic!(
            "Directory does not contain any aseprite files: {}",
            dir.as_ref().to_string_lossy()
        );
    }

    let sprite_structs = files.iter().map(gen_sprite_mod).collect::<Vec<_>>();
    let sprite_enum = gen_enum(&files);

    quote! {
        #( #sprite_structs )*
        #sprite_enum
    }
}

fn gen_enum(files: &[SpriteMeta]) -> TokenStream {
    let sprite_idents = files
        .iter()
        .map(|f| format_ident!("{}", f.name.to_upper_camel_case()))
        .collect::<Vec<_>>();
    let mod_idents = files
        .iter()
        .map(|f| format_ident!("{}", f.name.to_snake_case()))
        .collect::<Vec<_>>();
    let sprite_names = files.iter().map(|f| &f.name).collect::<Vec<_>>();
    quote! {
        pub enum Sprite {
            #( #sprite_idents ( self :: #mod_idents :: Sprite ) ),*
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

fn gen_sprite_mod(file: &SpriteMeta) -> TokenStream {
    let mod_ident = format_ident!("{}", file.name.to_snake_case());
    let sprite_ident = format_ident!("{}", file.name.to_upper_camel_case());
    let name = &file.name;
    quote! {
        pub mod #mod_ident {
            pub struct Sprite {}
            impl Sprite {
                fn sprite(self) -> super :: Sprite {
                    super :: Sprite :: #sprite_ident (self)
                }
            }
            impl ::sprity_core::Sprite for Sprite {
                fn name(&self) -> &'static str {
                    #name
                }
            }
        }
    }
}

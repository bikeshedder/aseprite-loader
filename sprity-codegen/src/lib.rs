use std::path::Path;

use heck::{ToSnakeCase, ToUpperCamelCase};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use sprity_core::{DynamicSpriteSheetMeta, LoadDirError};

pub fn aseprite_dir(
    loader: &dyn sprity_core::Loader,
    dir: &dyn AsRef<Path>,
) -> Result<TokenStream, LoadDirError> {
    let files = loader.load_dir_meta(dir)?;

    if files.is_empty() {
        return Err(LoadDirError::EmptyDirectory {
            ext: ".aseprite/.ase",
            dir: dir.as_ref().to_string_lossy().into(),
        });
    }

    let sprite_structs = files.iter().map(gen_sprite_mod).collect::<Vec<_>>();
    let sprite_enum = gen_enum(&files);

    Ok(quote! {
        #sprite_enum
        #( #sprite_structs )*
    })
}

fn gen_enum(files: &[DynamicSpriteSheetMeta]) -> TokenStream {
    let sprite_names = files
        .iter()
        .map(|f| f.name.to_upper_camel_case())
        .collect::<Vec<_>>();
    let sprite_idents = sprite_names
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect::<Vec<_>>();
    let tags = files
        .iter()
        .flat_map(|f| {
            f.tags
                .iter()
                .map(|tag| format_ident!("{}", tag.to_upper_camel_case()))
        })
        .sorted()
        .dedup()
        .collect::<Vec<_>>();
    let layers = files
        .iter()
        .flat_map(|f| {
            f.layers
                .iter()
                .map(|layer| format_ident!("{}", layer.to_upper_camel_case()))
        })
        .sorted()
        .dedup()
        .collect::<Vec<_>>();
    assert!(!tags.is_empty());
    quote! {
        #[allow(enum_variant_names)]
        pub enum Sprite {
            #( #sprite_idents ( #sprite_idents ) ),*
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
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[allow(enum_variant_names)]
        pub enum Layer {
            #( #layers , )*
        }
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[allow(enum_variant_names)]
        pub enum Tag {
            #( #tags , )*
        }
    }
}

fn gen_sprite_mod(file: &DynamicSpriteSheetMeta) -> TokenStream {
    let sprite_name = file.name.to_upper_camel_case();
    let sprite_ident = format_ident!("{}", sprite_name);
    let tag_ident = format_ident!("{}Tag", file.name.to_upper_camel_case());
    let layer_ident = format_ident!("{}Layer", file.name.to_upper_camel_case());
    let layers_ident = format_ident!("{}Layers", file.name.to_upper_camel_case());
    let tag_names = file
        .tags
        .iter()
        .map(|tag| tag.to_upper_camel_case())
        .collect::<Vec<_>>();
    let tag_idents = tag_names
        .iter()
        .map(|tag_name| format_ident!("{}", tag_name))
        .collect::<Vec<_>>();
    let tag_count = tag_idents.len();
    let layer_names = file
        .layers
        .iter()
        .map(|layer| layer.to_upper_camel_case())
        .collect::<Vec<_>>();
    let layer_idents = layer_names
        .iter()
        .map(|layer_name| format_ident!("{}", layer_name))
        .collect::<Vec<_>>();
    let layer_count = layer_idents.len();
    let layer_flags = file
        .layers
        .iter()
        .map(|layer| format_ident!("{}", layer.to_snake_case()))
        .collect::<Vec<_>>();
    quote! {
        /*
        pub const NAME: &str = #sprite_name;
        pub const TAGS: [super :: Tag; #tag_count] = [ #(super :: Tag :: #tag_idents , )* ];
        pub const LAYERS: [super :: Layer; #layer_count] = [ #(super :: Layer :: #layer_idents , )* ];
        */

        pub struct #sprite_ident {
            pub tag: #tag_ident,
            pub layers: #layers_ident,
        }
        impl #sprite_ident {
            fn sprite(self) -> Sprite {
                Sprite :: #sprite_ident (self)
            }
        }
        impl ::sprity_core::Sprite for #sprite_ident {
            fn name(&self) -> &'static str {
                #sprite_name
            }
        }
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[allow(enum_variant_names)]
        pub enum #tag_ident {
            #( #tag_idents , )*
        }
        impl #tag_ident {
            pub fn index(&self) -> usize {
                *self as usize
            }
        }
        impl ::sprity_core::Tag for #tag_ident {
            fn name(&self) -> &str {
                match self {
                    #( Self :: #tag_idents => #tag_names , )*
                }
            }
        }
        impl From<#tag_ident> for Tag {
            fn from(value: #tag_ident) -> Self {
                match value {
                    #( #tag_ident :: #tag_idents => Self :: #tag_idents , )*
                }
            }
        }
        impl std::convert::TryFrom<Tag> for #tag_ident {
            type Error = ();
            fn try_from(value: Tag) -> Result<Self, Self::Error> {
                match value {
                    #( Tag :: #tag_idents => Ok(Self :: #tag_idents) , )*
                    _ => Err(())
                }
            }
        }
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[allow(enum_variant_names)]
        pub enum #layer_ident {
            #( #layer_idents , )*
        }
        impl #layer_ident {
            pub fn index(&self) -> usize {
                *self as usize
            }
        }
        impl ::sprity_core::Layer for #layer_ident {
            fn name(&self) -> &str {
                match self {
                    #( Self :: #layer_idents => #layer_names , )*
                }
            }
        }
        impl From<#layer_ident> for Layer {
            fn from(value: #layer_ident) -> Self {
                match value {
                    #( #layer_ident :: #layer_idents => Self :: #layer_idents , )*
                }
            }
        }
        impl std::convert::TryFrom<Layer> for #layer_ident {
            type Error = ();
            fn try_from(value: Layer) -> Result<Self, Self::Error> {
                match value {
                    #( Layer :: #layer_idents => Ok(Self :: #layer_idents) , )*
                    _ => Err(())
                }
            }
        }
        #[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
        pub struct #layers_ident {
            #( pub #layer_flags : bool , )*
        }
    }
}

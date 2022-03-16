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
        #( #sprite_structs )*
        #sprite_enum
    })
}

fn gen_enum(files: &[DynamicSpriteSheetMeta]) -> TokenStream {
    let sprite_idents = files
        .iter()
        .map(|f| format_ident!("{}", f.name.to_upper_camel_case()))
        .collect::<Vec<_>>();
    let mod_idents = files
        .iter()
        .map(|f| format_ident!("{}", f.name.to_snake_case()))
        .collect::<Vec<_>>();
    let sprite_names = files.iter().map(|f| &f.name).collect::<Vec<_>>();
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
    let mod_ident = format_ident!("{}", file.name.to_snake_case());
    let sprite_ident = format_ident!("{}", file.name.to_upper_camel_case());
    let name = &file.name;
    let tag_idents = file
        .tags
        .iter()
        .map(|tag| format_ident!("{}", tag.to_upper_camel_case()))
        .collect::<Vec<_>>();
    let tag_count = tag_idents.len();
    let layer_idents = file
        .layers
        .iter()
        .map(|layer| format_ident!("{}", layer.to_upper_camel_case()))
        .collect::<Vec<_>>();
    let layer_count = layer_idents.len();
    let layer_flags = file
        .layers
        .iter()
        .map(|layer| format_ident!("{}", layer.to_snake_case()))
        .collect::<Vec<_>>();
    quote! {
        pub mod #mod_ident {

            pub const NAME: &str = #name;
            pub const TAGS: [super :: Tag; #tag_count] = [ #(super :: Tag :: #tag_idents , )* ];
            pub const LAYERS: [super :: Layer; #layer_count] = [ #(super :: Layer :: #layer_idents , )* ];

            pub struct Sprite {
                pub tag: Tag,
                pub layers: Layers,
            }
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
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            #[allow(enum_variant_names)]
            pub enum Tag {
                #( #tag_idents , )*
            }
            impl Tag {
                pub fn index(&self) -> usize {
                    *self as usize
                }
            }
            impl ::sprity_core::Tag for Tag {
                fn name(&self) -> &str {
                    #name
                }
            }
            impl From<Tag> for super::Tag {
                fn from(value: Tag) -> super::Tag {
                    match value {
                        #( Tag :: #tag_idents => Self :: #tag_idents , )*
                    }
                }
            }
            impl std::convert::TryFrom<super::Tag> for Tag {
                type Error = ();
                fn try_from(value: super::Tag) -> Result<Self, Self::Error> {
                    match value {
                        #( super :: Tag :: #tag_idents => Ok(Self :: #tag_idents) , )*
                        _ => Err(())
                    }
                }
            }
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            #[allow(enum_variant_names)]
            pub enum Layer {
                #( #layer_idents , )*
            }
            impl Layer {
                pub fn index(&self) -> usize {
                    *self as usize
                }
            }
            impl ::sprity_core::Layer for Layer {
                fn name(&self) -> &str {
                    #name
                }
            }
            impl From<Layer> for super::Layer {
                fn from(value: Layer) -> super::Layer {
                    match value {
                        #( Layer :: #layer_idents => Self :: #layer_idents , )*
                    }
                }
            }
            impl std::convert::TryFrom<super::Layer> for Layer {
                type Error = ();
                fn try_from(value: super::Layer) -> Result<Self, Self::Error> {
                    match value {
                        #( super :: Layer :: #layer_idents => Ok(Self :: #layer_idents) , )*
                        _ => Err(())
                    }
                }
            }
            #[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
            pub struct Layers {
                #( pub #layer_flags : bool , )*
            }
        }
    }
}

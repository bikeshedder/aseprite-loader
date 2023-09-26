use sprity_core::{
    sheet::{Sprite, SpriteSheet},
    Loader,
};

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    math::{Rect, Vec2, Vec3},
    prelude::{
        debug, AddAsset, Assets, BuildChildren, Bundle, Children, Commands, Component, Entity,
        GlobalTransform, Handle, Image, Plugin, PreUpdate, Query, Res, Transform, Visibility,
    },
    reflect::{TypePath, TypeUuid},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};

#[derive(Debug, Clone, TypeUuid, TypePath)]
#[uuid = "442cb6e1-0463-4d41-8e90-3b2c3b0a13a9"]
pub struct SprityAsset {
    pub atlas: Handle<TextureAtlas>,
    // FIXME add access to the tags, layers and frames
    pub sprites: Vec<Sprite>,
}

#[derive(Default)]
pub struct SprityAssetLoader {}

impl AssetLoader for SprityAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            debug!("Loading aseprite file: {:?}", load_context.path());
            let loader = sprity_aseprite::binary::loader::BinaryLoader::new();
            let sprite_loader = loader.load_sprite(bytes)?;
            let SpriteSheet { texture, sprites } = sprity_core::sheet::pack(&*sprite_loader)?;
            let width = texture.width();
            let height = texture.height();
            let texture_data = texture.into_raw();
            let texture: Handle<Image> = load_context.set_labeled_asset(
                "texture",
                LoadedAsset::new(Image::new(
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    texture_data,
                    TextureFormat::Rgba8UnormSrgb,
                )),
            );
            let atlas: Handle<TextureAtlas> = load_context.set_labeled_asset(
                "atlas",
                LoadedAsset::new(TextureAtlas {
                    texture,
                    size: Vec2::new(width as f32, height as f32),
                    textures: sprites
                        .iter()
                        .map(|sprite| Rect {
                            min: Vec2::new(sprite.x as f32, sprite.y as f32),
                            max: Vec2::new(
                                (sprite.x + sprite.width) as f32,
                                (sprite.y + sprite.height) as f32,
                            ),
                        })
                        .collect(),
                    texture_handles: None,
                }),
            );
            load_context.set_default_asset(LoadedAsset::new(SprityAsset { atlas, sprites }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ase", "aseprite"]
    }
}

#[derive(Debug, Component)]
pub struct SpritySprite {}

#[derive(Debug, Component)]
pub struct SprityLayer {
    pub tag_index: usize,
    pub layer_index: usize,
    pub frame: usize,
}

#[derive(Debug, Bundle, Default)]
pub struct SprityBundle {
    pub sprity_asset: Handle<SprityAsset>,
    pub texture_atlas: Handle<TextureAtlas>,
    pub sprite: TextureAtlasSprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

pub struct SprityPlugin;

impl Plugin for SprityPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<SprityAsset>()
            .init_asset_loader::<SprityAssetLoader>()
            .add_systems(PreUpdate, update_sprites);
        // FIXME
    }
}

pub(crate) fn update_sprites(
    mut commands: Commands,
    assets: Res<Assets<SprityAsset>>,
    mut q: Query<(Entity, Option<&Children>, &Handle<SprityAsset>)>,
    /*mut layer_query: Query<(
        &SprityLayer,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
    )>,*/
) {
    for (parent, children, asset_handle) in q.iter_mut() {
        if children.is_some() {
            continue;
        }
        if let Some(asset) = assets.get(asset_handle) {
            println!("Inserting children...");
            commands.entity(parent).with_children(|parent| {
                parent.spawn(SpriteSheetBundle {
                    texture_atlas: asset.atlas.clone(),
                    sprite: TextureAtlasSprite {
                        index: 0,
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(
                            asset.sprites[0].x as f32,
                            asset.sprites[0].y as f32,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                });
                parent.spawn(SpriteSheetBundle {
                    texture_atlas: asset.atlas.clone(),
                    sprite: TextureAtlasSprite {
                        index: 1,
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(
                            asset.sprites[1].x as f32,
                            asset.sprites[1].y as f32,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            });
        }
    }
    /*
    for (entity, children, asset_handle, mut atlas, mut sprite) in q.iter_mut() {
        // FIXME This code updates the atlas and sprite even if nothing has
        // changed. This code needs to be modified anyways as animation and
        // layers are the next thing to be implemented.
        if let Some(asset) = assets.get(asset_handle) {
            *atlas = asset.atlas.as_weak();
            *sprite = TextureAtlasSprite {
                ..Default::default()
            }
        }
    }
     */
}

#[cfg(test)]
mod tests {

    use sprity_core::{Layer, Sprite, Tag};

    pub mod sprites {
        use sprity_aseprite_macros::aseprite_dir;
        aseprite_dir!("../examples/assets");
    }

    #[test]
    fn test_basics() {
        let sprite = sprites::Sprite::Player(sprites::Player {
            tag: sprites::PlayerTag::WalkRight,
            layers: sprites::PlayerLayers {
                base: true,
                ..Default::default()
            },
        });
        assert_eq!(sprite.name(), "Player");
        let player = sprites::Player {
            tag: sprites::PlayerTag::WalkRight,
            layers: sprites::PlayerLayers {
                base: true,
                ..Default::default()
            },
        };
        assert_eq!(player.name(), "Player");
        let tag = sprites::PlayerTag::WalkRight;
        assert_eq!(tag.index(), 2);
        assert_eq!(tag.name(), "WalkRight");
        let layer = sprites::PlayerLayer::Base;
        assert_eq!(layer.index(), 0);
        assert_eq!(layer.name(), "Base");
    }

    #[test]
    fn test_layer_from() {
        assert_eq!(sprites::Layer::Base, sprites::PlayerLayer::Base.into());
    }

    #[test]
    fn test_layer_try_from() {
        assert_eq!(
            sprites::Layer::Base.try_into(),
            Ok(sprites::PlayerLayer::Base)
        );
    }

    #[test]
    fn test_tag_from() {
        assert_eq!(
            sprites::Tag::WalkRight,
            sprites::PlayerTag::WalkRight.into(),
        )
    }

    #[test]
    fn test_tag_try_from() {
        assert_eq!(
            sprites::Tag::WalkRight.try_into(),
            Ok(sprites::PlayerTag::WalkRight),
        )
    }
}

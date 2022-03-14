#[cfg(test)]
mod tests {

    use sprity_core::Sprite;

    pub mod sprites {
        use sprity_aseprite_macros::aseprite_dir;
        aseprite_dir!("../examples/assets");
    }

    #[test]
    fn it_works() {
        let sprite = sprites::Sprite::Player(sprites::player::Sprite {});
        assert_eq!(sprite.name(), "player");
        let player = sprites::player::Sprite {};
        assert_eq!(player.name(), "player");
        let tag = sprites::player::Tag::WalkRight;
        assert_eq!(tag.index(), 2);
        let layer = sprites::player::Layer::Base;
        assert_eq!(layer.index(), 0);
    }

    #[test]
    fn test_layer_from() {
        assert_eq!(sprites::Layer::Base, sprites::player::Layer::Base.into());
    }

    #[test]
    fn test_layer_try_from() {
        assert_eq!(
            sprites::Layer::Base.try_into(),
            Ok(sprites::player::Layer::Base)
        );
    }

    #[test]
    fn test_tag_from() {
        assert_eq!(
            sprites::Tag::WalkRight,
            sprites::player::Tag::WalkRight.into(),
        )
    }

    #[test]
    fn test_tag_try_from() {
        assert_eq!(
            sprites::Tag::WalkRight.try_into(),
            Ok(sprites::player::Tag::WalkRight),
        )
    }
}

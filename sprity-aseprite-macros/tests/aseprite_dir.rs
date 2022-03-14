use sprity_core::Sprite;

pub mod sprites {
    use sprity_aseprite_macros::aseprite_dir;
    aseprite_dir!("../examples/assets");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sprite = sprites::Sprite::Player(sprites::player::Sprite {});
        assert_eq!(sprite.name(), "player");
        let player = sprites::player::Sprite {};
        assert_eq!(player.name(), "player");
    }
}

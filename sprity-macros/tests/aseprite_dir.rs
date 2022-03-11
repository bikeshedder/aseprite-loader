use sprity_core::Sprite;

pub mod sprites {
    use sprity_macros::aseprite_dir;
    aseprite_dir!("../examples/assets");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sprite = sprites::Sprite::Player(sprites::Player {});
        assert_eq!(sprite.name(), "Player");
        let player = sprites::Player {};
        assert_eq!(player.name(), "Player");
    }
}

#[cfg(test)]
mod tests {
    use crate::loader::AsepriteFile;
    use image::RgbaImage;

    #[test]
    fn test_cell() {
        let path = "tests/combine.aseprite";
        let file = std::fs::read(path).unwrap();
        let file = AsepriteFile::load(&file).unwrap();

        for frame in file.frames().iter() {
            for (i, cell) in frame.cells.iter().enumerate() {
                let (width, height) = cell.size;
                println!("width: {}, height: {}", width, height);

                let mut target = vec![0; usize::from(width * height) * 4];
                file.load_image(cell.image_index, &mut target).unwrap();

                let image =
                    RgbaImage::from_raw(u32::from(width), u32::from(height), target).unwrap();

                //save
                let path = format!("out/cell_{}.png", i);
                image.save(path).unwrap();
            }
        }
    }

    #[test]
    fn test_combine() {
        let path = "tests/combine.aseprite";
        let file = std::fs::read(path).unwrap();
        let file = AsepriteFile::load(&file).unwrap();

        let (width, height) = file.size();
        for (index, _) in file.frames().iter().enumerate() {
            let mut target = vec![0; usize::from(width * height) * 4];
            let _ = file.combined_frame_image(index, &mut target).unwrap();
            let image = RgbaImage::from_raw(u32::from(width), u32::from(height), target).unwrap();
            let path = format!("tests/out/combined_{}.png", index);
            image.save(path).unwrap();
        }
    }
}

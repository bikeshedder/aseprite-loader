pub trait SpriteSheetMeta {
    fn name(&self) -> &str;
    fn tag_count(&self) -> usize;
    fn tag(&self, index: usize) -> &str;
    fn layer_count(&self) -> usize;
    fn layer(&self, index: usize) -> &str;
}

pub struct TagIterator<'a> {
    sprite_sheet: &'a dyn SpriteSheetMeta,
    index: usize,
    count: usize,
}

impl<'a> TagIterator<'a> {
    pub fn new(sheet: &'a dyn SpriteSheetMeta) -> Self {
        Self {
            sprite_sheet: sheet,
            count: sheet.tag_count(),
            index: 0,
        }
    }
}

impl<'a> Iterator for TagIterator<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let index = &mut self.index;
        if *index >= self.count {
            None
        } else {
            let item = self.sprite_sheet.tag(*index);
            *index += 1;
            Some(item)
        }
    }
}

pub struct LayerIterator<'a> {
    sprite_sheet: &'a dyn SpriteSheetMeta,
    index: usize,
    count: usize,
}

impl<'a> LayerIterator<'a> {
    pub fn new(sheet: &'a dyn SpriteSheetMeta) -> Self {
        Self {
            sprite_sheet: sheet,
            count: sheet.layer_count(),
            index: 0,
        }
    }
}

impl<'a> Iterator for LayerIterator<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let index = &mut self.index;
        if *index >= self.count {
            None
        } else {
            let item = self.sprite_sheet.layer(*index);
            *index += 1;
            Some(item)
        }
    }
}

#[derive(Debug)]
pub struct DynamicSpriteSheetMeta {
    pub name: String,
    pub tags: Vec<String>,
    pub layers: Vec<String>,
}

impl SpriteSheetMeta for DynamicSpriteSheetMeta {
    fn name(&self) -> &str {
        &self.name
    }
    fn tag_count(&self) -> usize {
        self.tags.len()
    }
    fn tag(&self, index: usize) -> &str {
        &self.tags[index]
    }
    fn layer_count(&self) -> usize {
        self.layers.len()
    }
    fn layer(&self, index: usize) -> &str {
        &self.layers[index]
    }
}

pub struct StaticSpriteSheetMeta<const TAG_COUNT: usize, const LAYER_COUNT: usize> {
    name: &'static str,
    tags: &'static [&'static str; TAG_COUNT],
    layers: &'static [&'static str; LAYER_COUNT],
}

impl<const TAG_COUNT: usize, const LAYER_COUNT: usize> SpriteSheetMeta
    for StaticSpriteSheetMeta<TAG_COUNT, LAYER_COUNT>
{
    fn name(&self) -> &str {
        self.name
    }
    fn tag_count(&self) -> usize {
        TAG_COUNT
    }
    fn tag(&self, index: usize) -> &str {
        self.tags[index]
    }
    fn layer_count(&self) -> usize {
        LAYER_COUNT
    }
    fn layer(&self, index: usize) -> &str {
        self.layers[index]
    }
}

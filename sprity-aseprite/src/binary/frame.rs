use super::{chunks::cel::CelChunk, scalars::Word};

#[derive(Debug)]
pub struct Frame<'a> {
    pub duration: Word,
    // The cel chunks can directly be indexed via the layer index
    pub cels: Vec<Option<CelChunk<'a>>>,
}

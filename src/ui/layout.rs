use euclid::default::Vector2D;
use super::frame::FrameId;

#[derive(Debug, Clone)]
pub struct FrameInfo {
    size: Vector2D<f32>,
    origin: Vector2D<f32>,
    frame: FrameId,
}
#[derive(Debug, Clone)]
pub enum Layout {
    Vertical,
    Horizontal,
    Frame(FrameInfo)
}

#[derive(Debug, Clone)]
pub struct FrameLayout {
    layout: Vec<Layout>
}

impl FrameLayout {
    pub fn new() -> Self {
        Self {
            layout: Vec::new(),
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Layout> {
        self.layout.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Layout> {
        self.layout.get_mut(idx)
    }
}
// use euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::Vector2F;
use super::frame::FrameId;

#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub frame: FrameId,
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

    pub fn push_frame(&mut self, frame_info: FrameInfo) {
        self.layout.push(Layout::Frame(frame_info));
    }
}

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

pub struct FrameIter<'a> {
    // parent: usize,
    // idx: usize,
    layout: std::slice::Iter<'a, Layout>
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

    pub fn frame_iter(&self) -> FrameIter {
        FrameIter {
                // parent: 0,
                // idx: 0,
                layout: self.layout.iter()
        }
    }
}

impl<'a> FrameIter<'a> {

}

impl<'a> std::iter::Iterator for FrameIter<'a> {
    type Item = &'a FrameInfo;

    fn next(&mut self) -> Option<Self::Item> {
        use Layout::*;
        loop {
            match self.layout.next() {
                Some(layout) => match layout {
                    Vertical | Horizontal => {
                        continue
                    }
                    Frame(info) => {
                        // if self.idx == self.parent * 2 + 1 {
                        //     self.idx = self.parent * 2 + 2;
                        // }
                        // else if self.idx == self.parent * 2 + 2 {
                        //     self.parent =
                        // }
                        return Some(info);
                    }
                }
                None => return None,
            }
        }
    }
}

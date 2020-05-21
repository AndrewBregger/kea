use std::sync::atomic::{AtomicUsize, Ordering};
use crate::core::ViewId;
use euclid::default::Vector2D;
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameId(usize);

pub struct Frame {
    id: FrameId,
    view: ViewId,
    size: Vector2D<f32>,
    origin: Vector2D<f32>,
}

impl Frame {
    pub fn new(view: ViewId, size: Vector2D<f32>, origin: Vector2D<f32>) -> Self {
        Self {
            id: FrameId(Self::next_id()),
            view,
            size,
            origin
        }
    }

    fn next_id() -> usize {
        static TOKEN: AtomicUsize = AtomicUsize::new(1);
        TOKEN.fetch_add(1, Ordering::SeqCst)
    }
}
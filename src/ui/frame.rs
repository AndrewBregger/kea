use std::sync::atomic::{AtomicUsize, Ordering};
// use crate::core::ViewId;
// use euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::Vector2F;
use crate::core;


#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameId(usize);

pub struct Frame {
    id: FrameId,
    buffer: core::BufferId,
    size: Vector2F,
    origin: Vector2F,
    active: bool,
}

impl Frame {
    pub fn new(buffer: core::BufferId, size: Vector2F, origin: Vector2F) -> Self {
        Self {
            id: FrameId(Self::next_id()),
            buffer,
            size,
            origin,
            active: false,
        }
    }

    pub fn id(&self) -> FrameId {
		self.id
    }

    fn next_id() -> usize {
        static TOKEN: AtomicUsize = AtomicUsize::new(1);
        TOKEN.fetch_add(1, Ordering::SeqCst)
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active
    }
}

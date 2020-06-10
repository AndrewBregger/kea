use std::sync::atomic::{AtomicUsize, Ordering};
// use crate::core::ViewId;
// use euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::Vector2F;
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameId(usize);

// pub struct Frame {
//     id: FrameId,
//     view: ViewId,
//     size: Vector2F,
//     origin: Vector2F,
// }
//
// impl Frame {
//     pub fn new(view: ViewId, size: Vector2F, origin: Vector2F) -> Self {
//         Self {
//             id: FrameId(Self::next_id()),
//             view,
//             size,
//             origin
//         }
//     }
//
//     fn next_id() -> usize {
//         static TOKEN: AtomicUsize = AtomicUsize::new(1);
//         TOKEN.fetch_add(1, Ordering::SeqCst)
//     }
// }
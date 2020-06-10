// use crate::euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::{vec2f, Vector2F};

use std::ops::Range;
use std::path::PathBuf;

use super::buffer::BufferId;
use super::CoreError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(usize);
pub const INVALID_VIEW_ID: ViewId = ViewId(0);

pub struct View {
    /// unique identifier for this view.
    id: ViewId,
    /// the size of the view is pixels.
    size: Vector2F,
    /// the buffer associated with this view.
    buffer: BufferId,
    /// the first line visible
    start_line: usize,
    /// the number of lines this view can show
    lines: usize,
}

impl View {
    pub fn new(id: usize, buffer: BufferId, size: Vector2F, start_line: usize, lines: usize) -> Result<Self, CoreError> {
        Ok(Self {
            id: ViewId(id),
            size,
            buffer,
            start_line,
            lines
        })
    }

    pub fn id(&self) -> ViewId {
        self.id
    }

    pub fn buffer(&self) -> BufferId {
        self.buffer
    }
}

#[derive(Debug, Clone)]
pub struct ViewInfo {
    /// view id
    pub view: ViewId,
    /// the pixel size of the new view
    pub size: Vector2F,
    /// the lines to be shown first
    pub start_line: usize,
    /// the number of lines show in this view
    pub lines: usize,
}

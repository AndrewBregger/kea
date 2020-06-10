// use crate::euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::{vec2f, Vector2F};

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;

use super::{Buffer, BufferId, View, ViewId, ViewInfo};
use super::buffer::{BufferResult};
use super::CoreError;

#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// buffer id
    pub path: Option<PathBuf>,
    /// the pixel size of the new view
    pub size: Vector2F,
    /// the lines to be shown first
    pub start_line: usize,
    /// the number of lines show in this view
    pub lines: usize,
}


struct Counter {
    counter: AtomicUsize,
}

impl Counter {
    fn new() -> Self {
        Self {
            counter: AtomicUsize::new(1)
        }
    }

    fn next(&self) -> usize {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}


pub struct Core {
    id_counter: Counter,
    buffers: BTreeMap<BufferId, Box<Buffer>>,
    views: BTreeMap<ViewId, Box<View>>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            id_counter: Counter::new(),
            buffers: BTreeMap::new(),
            views: BTreeMap::new(),
        }
    }

    fn next_id(&self) -> usize {
        self.id_counter.next()
    }

    fn insert_buffer(&mut self, buffer: Buffer) {
        assert!(self.buffers.insert(buffer.id(), Box::new(buffer)).is_none());
    }

    fn insert_view(&mut self, view: View) {
        assert!(self.views.insert(view.id(), Box::new(view)).is_none());
    }

    pub fn update_view(&self, view_id: ViewId) -> Update {
        unimplemented!()
    }

    pub fn open_file(&mut self, frame_info: FrameInfo) -> Result<ViewInfo, CoreError> {
        let FrameInfo { path, size, start_line, lines} = frame_info;

        let buffer_id = self.next_id();
        let buffer = match path {
            Some(path) => Buffer::from_path(path, buffer_id),
            None => Buffer::empty(buffer_id),
        }?;

        let buffer_id = buffer.id();
        self.insert_buffer(buffer);

        let view = View::new(self.next_id(), buffer_id, size, start_line, lines)?;
        let view_id = view.id();
        self.insert_view(view);

        Ok(ViewInfo {
            view: view_id,
            size,
            start_line,
            lines
        })
    }
}


#[derive(Debug, Clone)]
pub enum Edit {
    /// Opens a new view with a given file
    OpenFile(FrameInfo),
    Close,
}

#[derive(Debug, Clone)]
pub enum Update {
    View(ViewInfo),
    Error(CoreError)
}

impl Into<Update> for CoreError  {
    fn into(self) -> Update {
        Update::Error(self)
    }
}

impl Update {
    pub fn is_err(&self) -> bool {
        match self {
            Update::Error(_) => true,
            _ => false,
        }
    }
}
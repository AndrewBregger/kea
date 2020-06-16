// use crate::euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::{vec2f, Vector2F};

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;
use std::ops::Deref;

use super::{Buffer, BufferId};
use super::buffer::{BufferResult};
use super::CoreError;
use kea::{Ptr, ptr};
use std::borrow::Borrow;
use std::cell::{Ref, RefMut};


#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub path: Option<PathBuf>,
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
    buffers: BTreeMap<BufferId, Ptr<Buffer>>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            id_counter: Counter::new(),
            buffers: BTreeMap::new(),
        }
    }

    fn next_id(&self) -> usize {
        self.id_counter.next()
    }

    fn insert_buffer(&mut self, buffer: Buffer) {
        assert!(self.buffers.insert(buffer.id(), ptr(buffer)).is_none());
    }


    pub fn open_file(&mut self, buffer_info: BufferInfo) -> Result<BufferId, CoreError> {

        let buffer_id = self.next_id();
        let buffer = match buffer_info.path {
            Some(path) => Buffer::from_path(path, buffer_id),
            None => Buffer::empty(buffer_id),
        }?;

        let buffer_id = buffer.id();
        self.insert_buffer(buffer);

		Ok(buffer_id)
    }

    pub fn request_lines(&self, buffer: BufferId, start: usize, end: usize) -> Vec<String> {
        if let Some(buffer) = self.buffers.get(&buffer) {
            buffer.deref().borrow().request_lines(start, end)
            // buffer.borrow().request_lines(start, end)
            // buffer.borrow().request_lines(start, end)
        }
        else { unreachable!() }
    }

	/// retrieve the buffer of the given id.
	/// note: Because a buffer id can only be created
	/// 	  here, is is impossible for an invalid
	/// 	  id to be given.
    pub fn get_buffer(&self, id: &BufferId) -> Option<Ref<Buffer>> {
		if let Some(buff) = self.buffers.get(id) {
            Some(buff.deref().borrow())
        }
        else {
            None
        }

    }

    /// retrieve the buffer of the given id.
    /// note: Because a buffer id can only be created
    /// 	  here, is is impossible for an invalid
    /// 	  id to be given.
    pub fn get_buffer_mut(&mut self, id: &BufferId) -> Option<RefMut<Buffer>> {
        if let Some(buff) = self.buffers.get_mut(id) {
            if let Ok(buff) = buff.try_borrow_mut() {
                Some(buff)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Edit {
}

#[derive(Debug, Clone)]
pub enum Update {
}


use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Range;
// use crate::core::ViewId;
// use euclid::default::Vector2D;
use crate::pathfinder_geometry::vector::Vector2F;
use crate::core;
use crate::ui::line_cache::{LineCache, Text};
use crate::font::ScaledFontMetrics;

use log::error;


#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameId(usize);

#[derive(Debug, Clone)]
struct T {}

/// schemas for how the buffer can be invalidated
pub enum Invalidation {
    /// the frame is new and needs to be populated.
	Init,
	/// The user scrolled up this many pixels
	ScrollUp { pixels: usize },
	/// The user scrolled down this many pixels
	ScrollDown { pixels: usize }
}

pub struct Frame {
    /// id of this frame
    id: FrameId,
    /// the buffer this frame is rendering
    buffer: core::BufferId,
    /// the pixel size of the frame
    size: Vector2F,
    /// the lower left corner of this frame in pixels.
    origin: Vector2F,
    /// is this frame the focus of the user.
    active: bool,
    /// a cache of the current visable lines.
    cache: LineCache<T>,
    /// the metrics of the primary font to use when positioning the text.
    font_metrics: ScaledFontMetrics,
    /// the lines of the buffer this view is viewing.
    view: Range<usize>
}

impl Frame {
    pub fn new(buffer: core::BufferId, size: Vector2F, origin: Vector2F, metrics: ScaledFontMetrics) -> Self {
        let lines = (size.y().ceil() / metrics.line_height()) as usize;

        Self {
            id: FrameId(Self::next_id()),
            buffer,
            size,
            origin,
            active: false,
            cache: LineCache::new(lines),
            font_metrics: metrics,
            view: 0..lines
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

	pub fn update_line_cache(&mut self, invalidation: Invalidation, buffer: &core::Buffer) {
		match invalidation {
			Invalidation::Init => self.fill_cache(buffer),
			Invalidation::ScrollUp { .. } |
			Invalidation::ScrollDown { .. } => unimplemented!(),
		}
	}

	fn fill_cache(&mut self, buffer: &core::Buffer) {
		let lines = buffer.request_lines(self.view.start, self.view.end);
		for (idx, line) in lines.into_iter().enumerate() {
			self.set_line(idx, Text::from_string(idx + 1, line));
		}

		println!("{:#?}", self.cache);
	}

	fn set_line(&mut self, line_idx: usize, text: Text<T>) {
		if self.view.contains(&line_idx) {
        	// gets the index relative to the view of the buffer.
    		let line_cache_idx = line_idx - self.view.start;
    		self.cache.replace(line_cache_idx, text);
		}
		else {
    		error!("attempting an invalid line set: {}", line_idx);
		}
	}
}

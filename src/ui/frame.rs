use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
// use crate::core::ViewId;
// use euclid::default::Vector2D;
use crate::core;
use crate::font::ScaledFontMetrics;
use crate::pathfinder_geometry::vector::Vector2F;
use crate::renderer::{
    style::{Span, StyleId, StyleSpan},
    Color, Renderable, Renderer, TextLine,
};
use crate::ui::line_cache::{LineCache, Text};
use log::error;
use kea::{Ptr};
use std::cell::Ref;

pub enum CursorMotion {
    Left,
    Right,
    Up,
    Down,
}
/// cursor position, zero-indexed.
#[derive(Debug, Clone)]
pub struct Cursor {
    line: usize,
    column: usize,
}

impl Cursor {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    #[inline]
    pub fn line(&self) -> usize { self.line }

    #[inline]
    pub fn column(&self) -> usize { self.column }
}

#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameId(usize);

/// schemas for how the buffer can be invalidated
pub enum Invalidation {
    /// the frame is new and needs to be populated.
    Init,
    /// The user scrolled up this many pixels
    ScrollUp { pixels: usize, lines: usize },
    /// The user scrolled down this many pixels
    ScrollDown { pixels: usize, lines: usize },
}

pub struct Frame {
    /// id of this frame
    id: FrameId,
    /// the buffer this frame is rendering
    buffer: Ptr<core::Buffer>,
    /// the pixel size of the frame
    size: Vector2F,
    /// the lower left corner of this frame in pixels.
    origin: Vector2F,
    /// is this frame the focus of the user.
    active: bool,
    /// a cache of the current visible lines.
    cache: LineCache<TextLine>,
    // the metrics of the primary font to use when positioning the text.
    // font_metrics: ScaledFontMetrics,
    /// the lines of the buffer this view is viewing.
    view: Range<usize>,
    /// the position of the cursor in the buffer. For now there is only one, this will be expanded
    /// to allow multiple cursors.
    cursor: Cursor,
}

impl Frame {
    pub fn new(buffer: Ptr<core::Buffer>, size: Vector2F, origin: Vector2F, lines: usize) -> Self {
        Self {
            id: FrameId(Self::next_id()),
            buffer,
            size,
            origin,
            active: false,
            cache: LineCache::new(lines),
            view: 0..lines,
            cursor: Cursor::new(0, 0),
        }
    }

    pub fn compute_lines(height: f32, metrics: &ScaledFontMetrics) -> usize {
        (height / metrics.line_height()) as usize
    }

    fn next_id() -> usize {
        static TOKEN: AtomicUsize = AtomicUsize::new(1);
        TOKEN.fetch_add(1, Ordering::SeqCst)
    }

    pub fn buffer(&self) -> Ref<core::Buffer> {
        self.buffer.borrow()
    }

    pub fn id(&self) -> FrameId {
        self.id
    }

    pub fn width(&self) -> f32 {
        self.size.x()
    }

    pub fn height(&self) -> f32 {
        self.size.y()
    }

    pub fn origin(&self) -> &Vector2F {
        &self.origin
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active
    }

    pub fn lines(&self) -> &[Option<Text<TextLine>>] {
        self.cache.lines()
    }

    pub fn lines_mut(&mut self) -> &mut [Option<Text<TextLine>>] {
        self.cache.lines_mut()
    }

    pub fn update_line_cache(&mut self, invalidation: Invalidation) {
        match invalidation {
            Invalidation::Init => self.fill_cache(),
            Invalidation::ScrollUp { pixels, lines } => {},
            Invalidation::ScrollDown { pixels, lines } => {},
        }
    }

    fn fill_cache(&mut self) {
        let lines = self.buffer.borrow().request_lines(self.view.start, self.view.end);
        let mut populated_lines = 0;
        for (idx, line) in lines.into_iter().enumerate() {
            // if we have populated the line cache before viewing all of the lines
            // then ignore the rest of the given lines.
            if populated_lines >= self.view.len() {
                break;
            }

            // let cursor = self.get_cursors(self.view.start + idx);

            // layout the line
            for (offset, text) in self
                .layout_line(idx + 1, line, None)
                .into_iter()
                .enumerate()
            {
                self.set_line(idx + offset, text);
                populated_lines += 1;
            }
        }
    }

    fn set_line(&mut self, line_idx: usize, text: Text<TextLine>) {
        if self.view.contains(&line_idx) {
            // gets the index relative to the view of the buffer.
            let line_cache_idx = line_idx - self.view.start;
            self.cache.replace(line_cache_idx, text);
        } else {
            error!("attempting an invalid line set: {}", line_idx);
        }
    }

    fn layout_line(
        &mut self,
        line_number: usize,
        text: String,
        cursor: Option<Cursor>,
    ) -> Vec<Text<TextLine>> {
        let style = StyleSpan::new(StyleId(0), Span::new(0, text.len()));
        let cursors = if let Some(cursor) = cursor {
            vec![cursor.column]
        } else {
            Vec::new()
        };

        let line = Text::new(text, line_number, false, None, cursors, vec![style]);

        vec![line]
    }

    /// retreives all cursors for a given line.
    // pub fn get_cursors(&sself, line: usize) -> Vec<Cursor>
    pub fn get_cursors(&self, line: usize) -> Option<Cursor> {
        if self.cursor.line == line {
            Some(self.cursor.clone())
        } else {
            None
        }
    }
}

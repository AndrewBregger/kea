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
    buffer: core::BufferId,
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
    pub fn new(buffer: core::BufferId, size: Vector2F, origin: Vector2F, lines: usize) -> Self {
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

    pub fn buffer(&self) -> &core::BufferId {
        &self.buffer
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

    pub fn update_line_cache(&mut self, invalidation: Invalidation, buffer: &core::Buffer) {
        match invalidation {
            Invalidation::Init => self.fill_cache(buffer),
            Invalidation::ScrollUp { pixels, lines } => self.scroll_up(pixels, lines, buffer),
            Invalidation::ScrollDown { pixels, lines } => self.scroll_down(pixels, lines, buffer),
        }
    }

    fn fill_cache(&mut self, buffer: &core::Buffer) {
        let lines = buffer.request_lines(self.view.start, self.view.end);
        let mut populated_lines = 0;
        for (idx, line) in lines.into_iter().enumerate() {
            // if we have populated the line cache before viewing all of the lines
            // then ignore the rest of the given lines.
            if populated_lines >= self.view.len() {
                break;
            }

            let cursor = self.get_cursors(self.view.start + idx);

            // layout the line
            for (offset, text) in self
                .layout_line(idx + 1, line, cursor)
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

    pub fn move_cursor(&mut self, dir: CursorMotion, diff: usize) -> Option<isize> {
        match dir {
            CursorMotion::Up => {
                if self.cursor.line > 0 {
                    self.cursor.line -= diff;
                }
            }
            CursorMotion::Down => {
                self.cursor.line += diff;
            }
            CursorMotion::Left => {
                if self.cursor.column > 0 {
                    self.cursor.column -= diff;
                }
            }
            CursorMotion::Right => {
                self.cursor.column += diff;
            }
        }

        self.update_view_from_cursor()
    }

    /// attempts to update the view of the frame.
    /// It used the view according to the cursor.
    /// Returns the number of lines changes and the sign
    /// denotes the direction of the change.
    fn update_view_from_cursor(&mut self) -> Option<isize> {
        if self.view.contains(&self.cursor.line) {
            None
        } else {
            if self.cursor.line < self.view.start {
                let diff = self.view.start - self.cursor.line;
                self.view.start = self.cursor.line;
                Some(diff as isize)
            } else if self.cursor.line > self.view.end {
                let diff = (self.cursor.line - self.view.end) as isize;
                self.view.end = self.cursor.line;
                Some(-diff)
            } else {
                unreachable!()
            }
        }
    }

    pub fn scroll_up(&mut self, pixels: usize, lines: usize, buffer: &core::Buffer) {
        let view_length = self.view.len();
        let cache_lines = self.lines_mut();

        // move the still visable lines to the new line of the cache.
        for idx in 0..view_length - lines {
            cache_lines[idx] = cache_lines[idx + lines].clone();
        }

        // create a view into the buffer of only the needed lines.
        let mut updated_view = self.view.clone();
        updated_view.start = updated_view.end - lines;

        // update the line cache with the new lines.
        let buffer_lines = buffer.request_lines(updated_view.start, updated_view.end);
        let mut populated_lines = self.view.len() - lines;
        for (idx, line) in buffer_lines.into_iter().enumerate() {
            if populated_lines >= self.view.len() {
                break;
            }

            let cursor = self.get_cursors(self.view.end + idx - lines);

            for (offset, text) in self
                .layout_line(self.view.end + idx - lines, line, cursor)
                .into_iter()
                .enumerate()
            {
                self.set_line(self.view.len() + idx - lines + offset, text);
                populated_lines += 1;
            }
        }
    }

    pub fn scroll_down(&mut self, pixels: usize, lines: usize, buffer: &core::Buffer) {
        let view_length = self.view.len();

        let cache_lines = self.lines_mut();

        for idx in (0..view_length - lines).rev() {
            cache_lines[idx + lines] = cache_lines[idx].clone();
        }

        let mut updated_view = self.view.clone();
        updated_view.end = updated_view.start + lines;

        let mut updated_view = self.view.clone();
        updated_view.start = updated_view.end - lines;

        // update the line cache with the new lines.
        let buffer_lines = buffer.request_lines(updated_view.start, updated_view.end);
        let mut populated_lines = self.view.len() - lines;
        for (idx, line) in buffer_lines.into_iter().enumerate().rev() {
            if populated_lines >= self.view.len() {
                break;
            }

            let cursor = self.get_cursors(self.view.start + idx);

            for (offset, text) in self
                .layout_line(self.view.start + idx, line, cursor)
                .into_iter()
                .enumerate()
                .rev()
            {
                self.set_line(self.view.start + offset, text);
                populated_lines += 1;
            }
        }
    }
}

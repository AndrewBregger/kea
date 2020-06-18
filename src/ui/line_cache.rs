use crate::renderer::{style::StyleSpan, Renderable};
use log::error;

#[derive(Debug, Clone)]
pub struct Text<T> {
    /// the value
    pub text: String,
    /// the line number of this line in the text buffer.
    pub line_number: usize,
    /// is this line a visual line.
    pub visual_line: bool,
    /// optionally generated render data for this text
    pub assoc: Option<T>,
    /// the location of any cursors in this line.
    pub cursors: Vec<usize>,
    /// a list of styles to be used on text.
    /// the spans should be ordered and non overlapping. (if overlapping then the latter
    /// style will be used).
    // Ideally, the entire string should be represented by the spans but if parts are missing
    // then a default style will be used.
    pub styles: Vec<StyleSpan>,
}

impl<T> Text<T> {
    pub fn from_string(line: usize, text: String) -> Self {
        Self::new(text, line, false, None, Vec::new(), Vec::new())
    }

    pub fn new(
        text: String,
        line_number: usize,
        visual_line: bool,
        assoc: Option<T>,
        cursors: Vec<usize>,
        styles: Vec<StyleSpan>,
    ) -> Self {
        Self {
            text,
            line_number,
            visual_line,
            assoc,
            cursors,
            styles,
        }
    }

    pub fn set_assoc(&mut self, assoc: T) {
        self.assoc = Some(assoc);
    }
}

#[derive(Debug, Clone)]
pub struct LineCache<T> {
    lines: Vec<Option<Text<T>>>,
}

impl<T: Clone> LineCache<T> {
    pub fn new(size: usize) -> Self {
        Self {
            lines: vec![None; size],
        }
    }

    pub fn replace(&mut self, idx: usize, text: Text<T>) {
        if let Some(value) = self.lines.get_mut(idx) {
            *value = Some(text)
        } else {
            error!(
                "Requesting invalid line cache index: {} cache of size: {}",
                idx,
                self.lines.len()
            );
        }
    }

    pub fn lines(&self) -> &[Option<Text<T>>] {
        self.lines.as_slice()
    }

    pub fn lines_mut(&mut self) -> &mut [Option<Text<T>>] {
        self.lines.as_mut_slice()
    }
}

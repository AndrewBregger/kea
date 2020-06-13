use crate::renderer::Renderable;
use log::error;

#[derive(Debug, Clone)]
pub struct Text<T> {
    /// the value
    text: String,
    /// the line number of this line in the text buffer.
    line_number: usize,
    /// is this line a visual line.
    visual_line: bool,
    /// optionally generated render data for this text
	assoc: Option<T>,
	/// the location of any cursors in this line.
	cursors: Vec<usize>
}

impl<T> Text<T> {
	pub fn from_string(line: usize, text: String) -> Self {
    	Self::new(text, line, false, None, Vec::new())
	}

	pub fn new(text: String, line_number: usize, visual_line: bool, assoc: Option<T>, cursors: Vec<usize>) -> Self {
    	Self {
        	text,
        	line_number,
        	visual_line,
        	assoc,
        	cursors
    	}
	}
}

#[derive(Debug, Clone)]
pub struct LineCache<T> {
    lines: Vec<Option<Text<T>>>
}

impl<T: Clone> LineCache<T> {
	pub fn new(size: usize) -> Self {
    	Self {
        	lines: vec![None; size]
    	}
	}

	pub fn replace(&mut self, idx: usize, text: Text<T>) {
    	if let Some(value) = self.lines.get_mut(idx) {
        	*value = Some(text)
    	}
    	else {
        	error!("Requesting invalid line cache index: {} cache of size: {}", idx, self.lines.len());
    	}
	}
}

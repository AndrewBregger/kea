use super::Color;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct StyleId(pub usize);

impl StyleId {
    fn next() -> Self {
        static TOKEN: AtomicUsize = AtomicUsize::new(0);
        Self(TOKEN.fetch_add(1, Ordering::SeqCst))
    }
}

/// Span of a string in bytes [start..end)
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn contains(&self, idx: usize) -> bool {
        self.start <= idx && idx < self.end
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StyleSpan {
    style: StyleId,
    span: Span,
}

impl StyleSpan {
    pub fn new(style: StyleId, span: Span) -> Self {
        Self { style, span }
    }

    #[inline]
    pub fn contains(&self, idx: usize) -> bool {
        self.span.contains(idx)
    }

    pub fn style(&self) -> StyleId {
        self.style
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    id: StyleId,
    font_idx: usize,
    fg_color: Color,
    bg_color: Color,
    italic: bool,
    underline: bool,
}

impl Style {
    pub fn new(
        font_idx: usize,
        fg_color: Color,
        bg_color: Color,
        italic: bool,
        underline: bool,
    ) -> Self {
        Self {
            id: StyleId::next(),
            font_idx,
            fg_color,
            bg_color,
            italic,
            underline,
        }
    }

    pub fn id(&self) -> StyleId {
        self.id
    }

    pub fn font_idx(&self) -> usize {
        self.font_idx
    }

    pub fn text_color(&self) -> &Color {
        &self.fg_color
    }

    pub fn bg_color(&self) -> &Color {
        &self.bg_color
    }

    pub fn italic(&self) -> bool {
        self.italic
    }

    pub fn underline(&self) -> bool {
        self.underline
    }
}

#[derive(Debug, Clone)]
pub struct StyleMap {
    styles: BTreeMap<StyleId, Style>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self {
            styles: BTreeMap::new(),
        }
    }

    pub fn register_style(&mut self, style: Style) {
        self.styles.insert(style.id(), style);
    }

    pub fn style(&self, id: &StyleId) -> Option<&Style> {
        self.styles.get(id)
    }
}

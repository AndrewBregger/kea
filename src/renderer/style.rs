#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct StyleId(pub usize);

/// Span of a string in bytes [start..end)
#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end
        }
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
        Self {
            style,
            span
        }
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

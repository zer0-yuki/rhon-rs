use std::{cmp, fmt, ops::Range};

/// `[start, end)` range.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dummy() {
            write!(f, "DUMMY")
        } else {
            write!(f, "{}..{}", self.start, self.end)
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Span::DUMMY
    }
}

impl Span {
    pub const DUMMY: Span = Span {
        start: usize::MAX,
        end: usize::MAX,
    };

    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn is_dummy(&self) -> bool {
        *self == Span::DUMMY
    }

    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    pub fn contains_span(&self, span: Span) -> bool {
        span.start >= self.start && span.end <= self.end
    }

    pub fn merge(&self, span: Self) -> Self {
        Self {
            start: cmp::min(span.start, self.start),
            end: cmp::max(span.end, self.end),
        }
    }

    pub fn shift(&self, offset: isize) -> Self {
        Self {
            start: self.start.wrapping_add_signed(offset),
            end: self.end.wrapping_add_signed(offset),
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from(value: (usize, usize)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

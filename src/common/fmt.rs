use std::fmt;

pub struct DebugCompact<'a, T: fmt::Debug>(&'a T);

impl<'a, T: fmt::Debug> DebugCompact<'a, T> {
    pub fn new(val: &'a T) -> Self {
        Self(val)
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for DebugCompact<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

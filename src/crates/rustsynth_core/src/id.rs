//! Simple typed ID wrapper used to tag AST nodes, rules, and scene objects.

use std::marker::PhantomData;

/// A typed, opaque numeric identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T> {
    raw: u32,
    _phantom: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(raw: u32) -> Self {
        Self { raw, _phantom: PhantomData }
    }

    pub fn raw(self) -> u32 {
        self.raw
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

//! Abstract code iterator — wraps a source CodeIterator.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorImpl`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::StringLocated;

/// A code iterator that wraps another source iterator.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorImpl`.
pub struct AbstractCodeIterator<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> AbstractCodeIterator<S> {
    /// Creates a new `AbstractCodeIterator` wrapping the given source.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }

    /// Returns the source iterator.
    #[must_use]
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Returns the source iterator (mutable).
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }
}

impl<S: CodeIterator> CodeIterator for AbstractCodeIterator<S> {
    fn peek(&mut self) -> Option<StringLocated> {
        self.source.peek()
    }

    fn next(&mut self) -> Option<StringLocated> {
        self.source.next()
    }

    fn get_code_position(&self) -> Box<dyn CodePosition> {
        self.source.get_code_position()
    }

    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>) {
        self.source.jump_to_code_position(position);
    }
}

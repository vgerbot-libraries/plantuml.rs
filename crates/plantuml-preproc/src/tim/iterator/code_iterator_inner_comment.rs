//! Code iterator that removes inner comments.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorInnerComment`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::StringLocated;

/// A code iterator that removes inner comments (`/' ... '/` on the same line).
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorInnerComment`.
pub struct CodeIteratorInnerComment<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> CodeIteratorInnerComment<S> {
    /// Creates a new `CodeIteratorInnerComment`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorInnerComment<S> {
    fn peek(&mut self) -> Option<StringLocated> {
        let line = self.source.peek()?;
        Some(line.remove_inner_comment())
    }

    fn next(&mut self) -> Option<StringLocated> {
        let line = self.source.next()?;
        Some(line.remove_inner_comment())
    }

    fn get_code_position(&self) -> Box<dyn CodePosition> {
        self.source.get_code_position()
    }

    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>) {
        self.source.jump_to_code_position(position);
    }
}

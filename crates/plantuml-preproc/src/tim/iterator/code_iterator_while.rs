//! Code iterator that handles `!while`/`!endwhile` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorWhile`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::{StringLocated, TLineType};

/// A code iterator that handles `!while`/`!endwhile` loop blocks.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorWhile`.
pub struct CodeIteratorWhile<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> CodeIteratorWhile<S> {
    /// Creates a new `CodeIteratorWhile`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorWhile<S> {
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

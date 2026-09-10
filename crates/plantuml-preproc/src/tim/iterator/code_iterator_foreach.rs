//! Code iterator that handles `!foreach`/`!endfor` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorForeach`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::{StringLocated, TLineType};

/// A code iterator that handles `!foreach`/`!endfor` loop blocks.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorForeach`.
pub struct CodeIteratorForeach<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> CodeIteratorForeach<S> {
    /// Creates a new `CodeIteratorForeach`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorForeach<S> {
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

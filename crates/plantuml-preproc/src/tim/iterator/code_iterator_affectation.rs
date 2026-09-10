//! Code iterator that handles variable affectation directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorAffectation`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::{StringLocated, TLineType};

/// A code iterator that handles `!var = expr` affectation directives.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorAffectation`.
pub struct CodeIteratorAffectation<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> CodeIteratorAffectation<S> {
    /// Creates a new `CodeIteratorAffectation`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorAffectation<S> {
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

//! Code iterator that handles `!define`/`!definelong` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorLegacyDefine`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::{StringLocated, TLineType};

/// A code iterator that handles legacy `!define`/`!definelong` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorLegacyDefine`.
pub struct CodeIteratorLegacyDefine<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> CodeIteratorLegacyDefine<S> {
    /// Creates a new `CodeIteratorLegacyDefine`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorLegacyDefine<S> {
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

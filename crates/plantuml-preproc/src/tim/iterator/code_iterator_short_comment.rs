//! Code iterator that skips short comments.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorShortComment`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::StringLocated;

/// A code iterator that skips single-line comments (lines starting with `'`).
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorShortComment`.
pub struct CodeIteratorShortComment<S: CodeIterator> {
    source: S,
}

impl<S: CodeIterator> CodeIteratorShortComment<S> {
    /// Creates a new `CodeIteratorShortComment`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorShortComment<S> {
    fn peek(&mut self) -> Option<StringLocated> {
        loop {
            let line = self.source.peek()?;
            let s = line.get_string();
            let trimmed = s.trim_start();
            if trimmed.starts_with('\'') {
                self.source.next();
                continue;
            }
            return Some(line);
        }
    }

    fn next(&mut self) -> Option<StringLocated> {
        let line = self.peek()?;
        self.source.next();
        Some(line)
    }

    fn get_code_position(&self) -> Box<dyn CodePosition> {
        self.source.get_code_position()
    }

    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>) {
        self.source.jump_to_code_position(position);
    }
}

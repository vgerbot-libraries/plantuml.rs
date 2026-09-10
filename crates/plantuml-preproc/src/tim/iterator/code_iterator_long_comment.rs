//! Code iterator that skips long comments.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorLongComment`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::{StringLocated, TLineType};

/// A code iterator that skips lines inside long comments (`/' ... '/`).
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorLongComment`.
pub struct CodeIteratorLongComment<S: CodeIterator> {
    source: S,
    in_long_comment: bool,
}

impl<S: CodeIterator> CodeIteratorLongComment<S> {
    /// Creates a new `CodeIteratorLongComment`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self {
            source,
            in_long_comment: false,
        }
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorLongComment<S> {
    fn peek(&mut self) -> Option<StringLocated> {
        loop {
            let line = self.source.peek()?;
            if self.in_long_comment {
                if line.get_string().contains("'/") {
                    self.in_long_comment = false;
                }
                // Skip line in long comment
                self.source.next();
                continue;
            }
            let s = line.get_string();
            if s.contains("/'") && !s.contains("'/") {
                self.in_long_comment = true;
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

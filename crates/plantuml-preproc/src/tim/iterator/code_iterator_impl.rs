//! Code iterator implementation — basic line iterator.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorImpl`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::StringLocated;

/// A basic code iterator over a list of lines.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorImpl`.
pub struct CodeIteratorImpl {
    pub lines: Vec<StringLocated>,
    pub pos: usize,
}

impl CodeIteratorImpl {
    /// Creates a new `CodeIteratorImpl` from a list of lines.
    #[must_use]
    pub fn new(lines: Vec<StringLocated>) -> Self {
        Self { lines, pos: 0 }
    }
}

/// A code position tracking a line index.
#[derive(Debug, Clone)]
pub struct Position {
    pub pos: usize,
}

impl CodePosition for Position {
    fn clone_box(&self) -> Box<dyn CodePosition> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CodeIterator for CodeIteratorImpl {
    fn peek(&mut self) -> Option<StringLocated> {
        self.lines.get(self.pos).cloned()
    }

    fn next(&mut self) -> Option<StringLocated> {
        let result = self.lines.get(self.pos).cloned();
        self.pos += 1;
        result
    }

    fn get_code_position(&self) -> Box<dyn CodePosition> {
        Box::new(Position { pos: self.pos })
    }

    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>) {
        if let Some(pos) = position.as_any().downcast_ref::<Position>() {
            self.pos = pos.pos;
        }
    }
}

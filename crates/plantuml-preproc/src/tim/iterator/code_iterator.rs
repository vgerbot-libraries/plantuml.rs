//! CodeIterator trait for iterating over preprocessor lines.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIterator`.

use super::code_position::CodePosition;
use crate::StringLocated;

/// Trait for iterating over preprocessor lines with position tracking.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIterator`.
pub trait CodeIterator {
    /// Peeks at the next line without consuming it.
    fn peek(&mut self) -> Option<StringLocated>;

    /// Consumes and returns the next line.
    fn next(&mut self) -> Option<StringLocated>;

    /// Returns the current code position.
    fn get_code_position(&self) -> Box<dyn CodePosition>;

    /// Jumps to the given code position.
    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>);
}

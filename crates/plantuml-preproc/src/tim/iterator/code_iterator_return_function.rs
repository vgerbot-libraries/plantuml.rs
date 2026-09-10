//! Code iterator that handles `!function` declarations.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorReturnFunction`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::{StringLocated, TLineType};

/// A code iterator that collects `!function` body lines.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorReturnFunction`.
pub struct CodeIteratorReturnFunction<S: CodeIterator> {
    source: S,
    pending: Vec<StringLocated>,
}

impl<S: CodeIterator> CodeIteratorReturnFunction<S> {
    /// Creates a new `CodeIteratorReturnFunction`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self {
            source,
            pending: Vec::new(),
        }
    }

    /// Returns the pending function body lines.
    #[must_use]
    pub fn pending(&self) -> &[StringLocated] {
        &self.pending
    }

    /// Takes the pending function body lines.
    pub fn take_pending(&mut self) -> Vec<StringLocated> {
        std::mem::take(&mut self.pending)
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorReturnFunction<S> {
    fn peek(&mut self) -> Option<StringLocated> {
        self.source.peek()
    }

    fn next(&mut self) -> Option<StringLocated> {
        let line = self.source.next()?;
        let t = line.get_type();
        if t == TLineType::DeclareReturnFunction || t == TLineType::DeclareProcedure {
            self.pending.push(line.clone());
            // Collect body until !endfunction/!endprocedure
            loop {
                let body_line = self.source.next()?;
                let bt = body_line.get_type();
                if bt == TLineType::EndFunction {
                    break;
                }
                self.pending.push(body_line);
            }
        }
        Some(line)
    }

    fn get_code_position(&self) -> Box<dyn CodePosition> {
        self.source.get_code_position()
    }

    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>) {
        self.source.jump_to_code_position(position);
    }
}

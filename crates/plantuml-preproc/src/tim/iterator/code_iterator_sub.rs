//! Code iterator that handles `!startsub`/`!endsub` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorSub`.

use super::code_iterator::CodeIterator;
use super::code_position::CodePosition;
use crate::preproc::Sub;
use crate::{StringLocated, TLineType};

/// A code iterator that handles `!startsub`/`!endsub` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodeIteratorSub`.
pub struct CodeIteratorSub<S: CodeIterator> {
    source: S,
    subs: std::collections::HashMap<String, Vec<StringLocated>>,
    current_sub: Option<String>,
}

impl<S: CodeIterator> CodeIteratorSub<S> {
    /// Creates a new `CodeIteratorSub`.
    #[must_use]
    pub fn new(source: S) -> Self {
        Self {
            source,
            subs: std::collections::HashMap::new(),
            current_sub: None,
        }
    }

    /// Returns the subs collected.
    #[must_use]
    pub fn get_subs(&self) -> &std::collections::HashMap<String, Vec<StringLocated>> {
        &self.subs
    }
}

impl<S: CodeIterator> CodeIterator for CodeIteratorSub<S> {
    fn peek(&mut self) -> Option<StringLocated> {
        self.source.peek()
    }

    fn next(&mut self) -> Option<StringLocated> {
        let line = self.source.next()?;
        let t = line.get_type();
        match t {
            TLineType::Startsub => {
                // Parse sub name
                let s = line.get_string().trim();
                let name = s.strip_prefix("!startsub").unwrap_or("").trim().to_string();
                self.current_sub = Some(name);
                self.source.next() // Skip to next line
            }
            TLineType::Endsub => {
                self.current_sub = None;
                self.source.next()
            }
            _ => {
                if let Some(ref subname) = self.current_sub {
                    self.subs.entry(subname.clone()).or_default().push(line.clone());
                }
                Some(line)
            }
        }
    }

    fn get_code_position(&self) -> Box<dyn CodePosition> {
        self.source.get_code_position()
    }

    fn jump_to_code_position(&mut self, position: Box<dyn CodePosition>) {
        self.source.jump_to_code_position(position);
    }
}

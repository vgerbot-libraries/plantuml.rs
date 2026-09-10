//! A composite filter that chains multiple `ReadFilter`s.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.ReadFilterAnd`.

use crate::preproc::read_line::ReadLine;
use crate::preproc2::read_filter::ReadFilter;

/// A `ReadFilter` that applies multiple filters in sequence.
///
/// Ported from `net.sourceforge.plantuml.preproc2.ReadFilterAnd`.
pub struct ReadFilterAnd {
    filters: Vec<Box<dyn ReadFilter>>,
}

impl ReadFilterAnd {
    /// Creates a new empty `ReadFilterAnd`.
    #[must_use]
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    /// Adds a filter to the chain.
    pub fn add(&mut self, filter: Box<dyn ReadFilter>) {
        self.filters.push(filter);
    }
}

impl Default for ReadFilterAnd {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadFilter for ReadFilterAnd {
    fn apply_filter(&self, mut current: Box<dyn ReadLine>) -> Box<dyn ReadLine> {
        for f in &self.filters {
            current = f.apply_filter(current);
        }
        current
    }
}

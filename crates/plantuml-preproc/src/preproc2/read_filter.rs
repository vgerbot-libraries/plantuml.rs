//! Trait for line-reading filters.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.ReadFilter`.

use crate::preproc::read_line::ReadLine;

/// A filter that wraps a `ReadLine` and transforms its output.
///
/// Ported from `net.sourceforge.plantuml.preproc2.ReadFilter`.
pub trait ReadFilter {
    /// Applies this filter to the given source `ReadLine`, returning a new `ReadLine`.
    fn apply_filter(&self, source: Box<dyn ReadLine>) -> Box<dyn ReadLine>;
}

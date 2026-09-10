//! The main preprocessor that chains read filters.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.Preprocessor`.

use std::io;

use crate::preproc::read_line::ReadLine;
use crate::preproc::read_line_numbered::ReadLineNumbered;
use crate::preproc2::read_filter::ReadFilter;
use crate::preproc2::read_filter_add_config::ReadFilterAddConfig;
use crate::preproc2::read_filter_and::ReadFilterAnd;
use crate::preproc2::read_filter_merge_lines::ReadFilterMergeLines;
use crate::StringLocated;

/// The main preprocessor: applies config insertion and line merging filters.
///
/// Ported from `net.sourceforge.plantuml.preproc2.Preprocessor`.
pub struct Preprocessor {
    source: Box<dyn ReadLine>,
}

impl Preprocessor {
    /// Creates a new `Preprocessor` with the given config lines and reader.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.Preprocessor.Preprocessor`.
    #[must_use]
    pub fn new(config: Vec<String>, reader: Box<dyn ReadLine>) -> Self {
        let mut filters = ReadFilterAnd::new();
        filters.add(Box::new(ReadFilterAddConfig::new(config)));
        filters.add(Box::new(ReadFilterMergeLines));
        let source = filters.apply_filter(reader);
        Self { source }
    }
}

impl ReadLine for Preprocessor {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        self.source.read_line()
    }

    fn close(&mut self) -> io::Result<()> {
        self.source.close()
    }
}

impl ReadLineNumbered for Preprocessor {}

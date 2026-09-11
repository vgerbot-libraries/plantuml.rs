//! A filter that merges lines ending with backslash.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.ReadFilterMergeLines`.

use std::io;

use plantuml_core::DiagramType;

use crate::preproc::read_line::ReadLine;
use crate::preproc2::read_filter::ReadFilter;
use crate::stubs::{StartUtils, StringUtils};
use crate::StringLocated;

/// A `ReadFilter` that merges lines ending with a backslash continuation.
///
/// Ported from `net.sourceforge.plantuml.preproc2.ReadFilterMergeLines`.
pub struct ReadFilterMergeLines;

impl ReadFilter for ReadFilterMergeLines {
    fn apply_filter(&self, source: Box<dyn ReadLine>) -> Box<dyn ReadLine> {
        Box::new(MergeLinesReader {
            source,
            manage_ending_backslash: true,
        })
    }
}

struct MergeLinesReader {
    source: Box<dyn ReadLine>,
    manage_ending_backslash: bool,
}

impl ReadLine for MergeLinesReader {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        let mut result = self.source.read_line()?;

        if let Some(ref r) = result {
            if StartUtils::is_start_directive(r.get_string()) && self.is_ditaa(r.get_string()) {
                self.manage_ending_backslash = false;
            }
        }
        if let Some(ref r) = result {
            if StartUtils::is_end_directive(r.get_string()) {
                self.manage_ending_backslash = true;
            }
        }

        loop {
            let current = match &result {
                Some(r) if self.manage_ending_backslash
                    && StringUtils::ends_with_backslash(r.get_string()) =>
                {
                    r.clone()
                }
                _ => break,
            };

            let next = self.source.read_line()?;
            match next {
                None => break,
                Some(n) => {
                    result = Some(current.merge_end_backslash(&n));
                }
            }
        }

        Ok(result)
    }

    fn close(&mut self) -> io::Result<()> {
        self.source.close()
    }
}

impl MergeLinesReader {
    #[allow(clippy::unused_self)]
    fn is_ditaa(&self, string: &str) -> bool {
        let trimmed = string.trim();
        DiagramType::find_start_types(trimmed).contains(&DiagramType::Ditaa)
    }
}

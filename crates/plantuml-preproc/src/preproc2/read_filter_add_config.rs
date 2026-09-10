//! A filter that inserts config lines after `@start` directives.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.ReadFilterAddConfig`.

use std::io;
use std::sync::Mutex;

use crate::preproc::read_line::ReadLine;
use crate::preproc::read_line_list::ReadLineList;
use crate::stubs::{LineLocation, StartUtils};
use crate::StringLocated;
use crate::preproc2::read_filter::ReadFilter;
use crate::preproc2::read_filter_quote_comment::ReadFilterQuoteComment;

/// A `ReadFilter` that inserts configuration lines after each `@start` directive.
///
/// Ported from `net.sourceforge.plantuml.preproc2.ReadFilterAddConfig`.
pub struct ReadFilterAddConfig {
    config: Vec<String>,
}

impl ReadFilterAddConfig {
    /// Creates a new `ReadFilterAddConfig` with the given config lines.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.ReadFilterAddConfig.ReadFilterAddConfig`.
    #[must_use]
    pub fn new(config: Vec<String>) -> Self {
        Self { config }
    }
}

impl ReadFilter for ReadFilterAddConfig {
    fn apply_filter(&self, raw: Box<dyn ReadLine>) -> Box<dyn ReadLine> {
        Box::new(AddConfigReader {
            raw,
            config: self.config.clone(),
            inserted: None,
        })
    }
}

struct AddConfigReader {
    raw: Box<dyn ReadLine>,
    config: Vec<String>,
    inserted: Option<Box<dyn ReadLine>>,
}

impl ReadLine for AddConfigReader {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        // First, try to read from inserted lines
        if let Some(ref mut inserted) = self.inserted {
            if let Some(result) = inserted.read_line()? {
                return Ok(Some(result));
            }
            let _ = inserted.close();
            self.inserted = None;
        }

        let result = self.raw.read_line()?;
        if let Some(ref r) = result {
            if StartUtils::is_start_directive(r.get_string()) && !self.config.is_empty() {
                let location = r.get_location().clone();
                let config_reader = ReadLineList::new(self.config.clone(), location);
                let filter = ReadFilterQuoteComment;
                self.inserted = Some(filter.apply_filter(Box::new(config_reader)));
            }
        }
        Ok(result)
    }

    fn close(&mut self) -> io::Result<()> {
        self.raw.close()
    }
}

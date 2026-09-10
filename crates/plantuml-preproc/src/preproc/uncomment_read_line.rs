//! A `ReadLine` that removes comment prefixes.
//!
//! Ported from `net.sourceforge.plantuml.preproc.UncommentReadLine`.

use std::io;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;
use crate::stubs::StartUtils;

/// Wraps a `ReadLine` and removes comment prefixes (text before `@startuml`)
/// from each line.
///
/// Ported from `net.sourceforge.plantuml.preproc.UncommentReadLine`.
pub struct UncommentReadLine {
    raw: Box<dyn ReadLine>,
    header_to_remove: Option<String>,
    paused: bool,
}

impl UncommentReadLine {
    /// Creates a new `UncommentReadLine` wrapping the given source.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.UncommentReadLine.UncommentReadLine`.
    #[must_use]
    pub fn new(source: Box<dyn ReadLine>) -> Self {
        Self {
            raw: source,
            header_to_remove: None,
            paused: false,
        }
    }

    /// Sets whether the reader is paused (looking for `@unpause`).
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
}

impl ReadLine for UncommentReadLine {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        let result = match self.raw.read_line()? {
            Some(r) => r,
            None => return Ok(None),
        };

        // Check for text before @startuml
        if let Some(tmp) = StartUtils::before_start_uml(result.get_string()) {
            self.header_to_remove = Some(tmp);
        }

        // If paused, look for @unpause
        if self.paused {
            let pause_re = regex::Regex::new(StartUtils::PAUSE_PATTERN).ok();
            if let Some(re) = pause_re {
                if let Some(caps) = re.captures(result.get_string()) {
                    if let Some(g1) = caps.get(1) {
                        self.header_to_remove = Some(g1.as_str().to_string());
                    }
                }
            }
        }

        // Remove the header prefix
        if let Some(ref header) = self.header_to_remove {
            if header.starts_with(result.get_string()) {
                return Ok(Some(StringLocated::new("", result.get_location().clone())));
            }
            if result.get_string().starts_with(header.as_str()) {
                return Ok(Some(result.substring(header.len(), result.get_string().len())));
            }
        }

        Ok(Some(result))
    }

    fn close(&mut self) -> io::Result<()> {
        self.raw.close()
    }
}

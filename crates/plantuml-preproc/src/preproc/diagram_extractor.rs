//! Extracts a single diagram from a `ReadLine` by finding `@start`/`@end` blocks.
//!
//! Ported from `net.sourceforge.plantuml.preproc.DiagramExtractor`.

use std::io;
use std::sync::LazyLock;

use regex::Regex;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;
use crate::stubs::StartUtils;

#[allow(clippy::trivial_regex)]
static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap_or_else(|_| Regex::new("$").unwrap_or_else(|_| Regex::new("$").unwrap())));

/// A `ReadLine` that skips to a specific `@start` block (by index or uid)
/// and reads until the corresponding `@end`.
///
/// Ported from `net.sourceforge.plantuml.preproc.DiagramExtractor`.
pub struct DiagramExtractor {
    raw: Box<dyn ReadLine>,
    finished: bool,
}

impl DiagramExtractor {
    /// Creates a new `DiagramExtractor`. During construction, it reads from `raw`
    /// until it finds the target `@start` block (by block index or uid).
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DiagramExtractor.DiagramExtractor`.
    #[must_use]
    pub fn new(mut raw: Box<dyn ReadLine>, suf: Option<&str>) -> Self {
        let mut block = 0i32;
        let mut uid: Option<&str> = None;

        if let Some(s) = suf {
            if DIGITS.is_match(s) {
                block = s.parse::<i32>().unwrap_or(0);
            } else {
                uid = Some(s);
            }
        }

        if block < 0 {
            block = 0;
        }

        let mut s: Option<StringLocated>;
        loop {
            s = if let Ok(line) = raw.read_line() { line } else {
                break;
            };
            match &s {
                None => {
                    break;
                }
                Some(line) => {
                    if StartUtils::is_start_directive(line.get_string()) && Self::check_uid(uid, line) {
                        if block == 0 {
                            return Self { raw, finished: false };
                        }
                        block -= 1;
                    }
                }
            }
        }

        Self { raw, finished: true }
    }

    fn check_uid(uid: Option<&str>, s: &StringLocated) -> bool {
        uid.is_none_or(|u| {
            let pattern = format!(".*id={}\\W.*", regex::escape(u));
            Regex::new(&pattern)
                .is_ok_and(|re| re.is_match(&s.to_string()))
        })
    }
}

impl ReadLine for DiagramExtractor {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        if self.finished {
            return Ok(None);
        }
        let result = self.raw.read_line()?;
        match &result {
            Some(r) if StartUtils::is_end_directive(r.get_string()) => {
                self.finished = true;
                Ok(None)
            }
            _ => Ok(result),
        }
    }

    fn close(&mut self) -> io::Result<()> {
        self.raw.close()
    }
}

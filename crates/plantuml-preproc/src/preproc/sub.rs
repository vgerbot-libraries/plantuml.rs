//! A named sub-block of lines for `!startsub`/`!includesub`.
//!
//! Ported from `net.sourceforge.plantuml.preproc.Sub`.

use std::io;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;
use crate::TLineType;

/// A named sub block containing a list of lines.
///
/// Ported from `net.sourceforge.plantuml.preproc.Sub`.
#[derive(Debug, Clone)]
pub struct Sub {
    name: String,
    lines: Vec<StringLocated>,
}

impl Sub {
    /// Creates a new `Sub` with the given name.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Sub.Sub`.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            lines: Vec::new(),
        }
    }

    /// Adds a line to this sub block.
    pub fn add(&mut self, s: StringLocated) {
        self.lines.push(s);
    }

    /// Returns the lines in this sub block.
    pub fn lines(&self) -> &[StringLocated] {
        &self.lines
    }

    /// Reads a sub block from a `ReadLine`, collecting lines between
    /// `!startsub <blocname>` and `!endsub`.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.Sub.fromFile`.
    pub fn from_file(
        reader: &mut dyn ReadLine,
        blocname: &str,
    ) -> io::Result<Option<Self>> {
        let mut result: Option<Self> = None;
        let mut skip = false;

        while let Some(s) = reader.read_line()? {
            let trimmed = s.get_trimmed();
            let line_type = trimmed.get_type();

            if line_type == TLineType::Startsub {
                // Parse the !startsub directive to get the sub name
                // In the full implementation, this would use EaterStartsub.
                // For now, we check if the line contains the blocname.
                let line_str = trimmed.get_string();
                if line_str.contains(blocname) {
                    skip = false;
                    if result.is_none() {
                        result = Some(Self::new(blocname));
                    }
                }
                continue;
            }

            if line_type == TLineType::Endsub && result.is_some() {
                skip = true;
            }

            if result.is_some() && !skip {
                if let Some(ref mut r) = result {
                    r.add(s);
                }
            }
        }

        Ok(result)
    }
}

impl std::fmt::Display for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sub({})", self.name)
    }
}

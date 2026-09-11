//! A filter that removes comment lines.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.ReadFilterQuoteComment`.

use std::io;

use crate::preproc::read_line::ReadLine;
use crate::preproc2::read_filter::ReadFilter;
use crate::StringLocated;

/// A `ReadFilter` that skips comment lines (lines starting with `'` or `/' ... '/`).
///
/// Ported from `net.sourceforge.plantuml.preproc2.ReadFilterQuoteComment`.
pub struct ReadFilterQuoteComment;

impl ReadFilter for ReadFilterQuoteComment {
    fn apply_filter(&self, source: Box<dyn ReadLine>) -> Box<dyn ReadLine> {
        Box::new(QuoteCommentReader { source })
    }
}

struct QuoteCommentReader {
    source: Box<dyn ReadLine>,
}

impl ReadLine for QuoteCommentReader {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        let mut long_comment = false;
        loop {
            let result = self.source.read_line()?;
            let Some(line) = result else { return Ok(None) };

            let trim = line.get_string().replace('\t', " ").trim().to_string();

            if long_comment && trim.ends_with("'/") {
                long_comment = false;
                continue;
            }
            if long_comment {
                continue;
            }
            if trim.starts_with('\'') {
                continue;
            }
            if trim.starts_with("/'") && trim.ends_with("'/") {
                continue;
            }
            if trim.starts_with("/'") && !trim.contains("'/") {
                long_comment = true;
                continue;
            }
            return Ok(Some(line.remove_inner_comment()));
        }
    }

    fn close(&mut self) -> io::Result<()> {
        self.source.close()
    }
}

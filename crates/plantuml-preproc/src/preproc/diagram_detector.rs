//! Detects diagrams in files, URLs, and byte arrays.
//!
//! Ported from `net.sourceforge.plantuml.preproc.DiagramDetector`.

use std::io;

use crate::StringLocated;
use crate::preproc::diagram_extractor::DiagramExtractor;
use crate::preproc::read_line::ReadLine;
use crate::preproc::read_line_reader::ReadLineReader;
use crate::preproc::read_line_simple::ReadLineSimple;
use crate::preproc::uncomment_read_line::UncommentReadLine;
use crate::stubs::{InputFile, SURL, StartUtils};

/// Detects whether a source contains `@start`/`@end` diagram blocks
/// and wraps the reader in a `DiagramExtractor` if so.
///
/// Ported from `net.sourceforge.plantuml.preproc.DiagramDetector`.
pub struct DiagramDetector;

impl DiagramDetector {
    /// Extracts a diagram from a URL.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DiagramDetector.extractFromUrl`.
    pub fn extract_from_url(
        url: &SURL,
        s: StringLocated,
        uid: Option<&str>,
    ) -> io::Result<Option<Box<dyn ReadLine>>> {
        let raw = Self::new_read_line_from_url(url, &s)?;
        if Self::contains_start_diagram(raw)? {
            let raw2 = Self::new_read_line_from_url(url, &s)?;
            return Ok(Some(Box::new(DiagramExtractor::new(raw2, uid))));
        }
        Ok(None)
    }

    /// Extracts a diagram from a byte array.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DiagramDetector.extractFromBytes`.
    pub fn extract_from_bytes(
        puml: &[u8],
        description: &str,
    ) -> io::Result<Option<Box<dyn ReadLine>>> {
        let raw = Self::new_read_line_from_input(puml, description);
        if Self::contains_start_diagram(raw)? {
            let raw1 = Self::new_read_line_from_input(puml, description);
            return Ok(Some(Box::new(DiagramExtractor::new(raw1, None))));
        }
        Ok(None)
    }

    /// Extracts a diagram from a file.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DiagramDetector.extractFromFile`.
    pub fn extract_from_file(
        f2: &InputFile,
        description: &str,
    ) -> io::Result<Option<Box<dyn ReadLine>>> {
        let data = std::fs::read(&f2.path).unwrap_or_default();
        Self::extract_from_bytes(&data, description)
    }

    fn new_read_line_from_input(data: &[u8], description: &str) -> Box<dyn ReadLine> {
        let raw = ReadLineReader::from_bytes(data, description);
        Self::uncomment_and_merge(Box::new(raw))
    }

    fn new_read_line_from_url(url: &SURL, s: &StringLocated) -> io::Result<Box<dyn ReadLine>> {
        match url.open_stream() {
            Some(data) => {
                let raw = ReadLineReader::from_bytes(&data, &url.to_string());
                Ok(Self::uncomment_and_merge(Box::new(raw)))
            }
            None => Ok(Box::new(ReadLineSimple::new(s.clone(), "Cannot connect"))),
        }
    }

    fn uncomment_and_merge(reader: Box<dyn ReadLine>) -> Box<dyn ReadLine> {
        // In Java, this also applies ReadFilterMergeLines before UncommentReadLine.
        // We apply UncommentReadLine directly; the merge filter is applied by the Preprocessor.
        Box::new(UncommentReadLine::new(reader))
    }

    /// Consumes the reader and checks if any line is a `@start` directive.
    fn contains_start_diagram(mut r: Box<dyn ReadLine>) -> io::Result<bool> {
        while let Some(line) = r.read_line()? {
            if StartUtils::is_start_directive(line.get_string()) {
                let _ = r.close();
                return Ok(true);
            }
        }
        let _ = r.close();
        Ok(false)
    }
}

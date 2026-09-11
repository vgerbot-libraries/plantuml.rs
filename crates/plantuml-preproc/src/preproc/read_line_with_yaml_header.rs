//! A `ReadLine` that detects and strips YAML headers.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLineWithYamlHeader`.

use std::collections::HashMap;
use std::io;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;

/// Wraps a `ReadLine` and detects YAML headers (delimited by `---`).
///
/// The header lines are consumed and stored as metadata.
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLineWithYamlHeader`.
pub struct ReadLineWithYamlHeader {
    source: Box<dyn ReadLine>,
    yaml_header: Option<Vec<String>>,
    metadata: HashMap<String, String>,
}

impl ReadLineWithYamlHeader {
    /// Creates a new `ReadLineWithYamlHeader` wrapping the given source.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineWithYamlHeader.ReadLineWithYamlHeader`.
    #[must_use]
    pub fn new(source: Box<dyn ReadLine>) -> Self {
        Self {
            source,
            yaml_header: None,
            metadata: HashMap::new(),
        }
    }

    /// Returns the metadata extracted from the YAML header.
    #[must_use]
    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Removes the YAML header from a list of `StringLocated` lines.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineWithYamlHeader.removeYamlHeader`.
    #[must_use]
    pub fn remove_yaml_header(input: &[StringLocated]) -> Vec<StringLocated> {
        if input.len() > 1 && Self::is_separator(&input[1]) {
            let mut result = Vec::new();
            result.push(input[0].clone());
            for i in 2..input.len() {
                if Self::is_separator(&input[i]) {
                    result.extend(input[i + 1..].iter().cloned());
                    return result;
                }
            }
        }
        input.to_vec()
    }

    fn is_separator(line: &StringLocated) -> bool {
        line.get_string() == "---"
    }
}

impl ReadLine for ReadLineWithYamlHeader {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        let line = self.source.read_line()?;
        if self.yaml_header.is_none() {
            // First line of the file
            self.yaml_header = Some(Vec::new());
            if let Some(ref l) = line {
                if Self::is_separator(l) {
                    // Read until the second separator
                    let mut current: Option<StringLocated>;
                    loop {
                        current = self.source.read_line()?;
                        match &current {
                            None => break,
                            Some(l2) if Self::is_separator(l2) => {
                                // Skip the second separator, read one more line
                                current = self.source.read_line()?;
                                break;
                            }
                            Some(l2) => {
                                let tmp = l2.get_string();
                                if let Some(header) = self.yaml_header.as_mut() {
                                    header.push(tmp.to_string());
                                }
                                if let Some(idx) = tmp.find(':') {
                                    if idx > 0 {
                                        let key = tmp[..idx].trim().to_string();
                                        let value = tmp[idx + 1..].trim().to_string();
                                        self.metadata.insert(key, value);
                                    }
                                }
                            }
                        }
                    }
                    return Ok(current);
                }
            }
        }
        Ok(line)
    }

    fn close(&mut self) -> io::Result<()> {
        self.source.close()
    }
}

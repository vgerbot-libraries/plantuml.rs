//! Utility methods for the preprocessor.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils`.

use std::sync::LazyLock;

use regex::Regex;

use crate::preproc::diagram_detector::DiagramDetector;
use crate::preproc::read_line::ReadLine;
use crate::preproc::read_line_reader::ReadLineReader;
use crate::preproc::read_line_simple::ReadLineSimple;
use crate::preproc::stdlib::Stdlib;
use crate::stubs::SURL;
use crate::StringLocated;
use crate::tim::eater_exception::EaterException;

static ENV_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(clippy::trivial_regex)]
    Regex::new(r"%(\w+)%").unwrap_or_else(|_| Regex::new("$").unwrap_or_else(|_| Regex::new("$").unwrap()))
});

/// Utility methods for the preprocessor.
///
/// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils`.
pub struct PreprocessorUtils;

impl PreprocessorUtils {
    /// Replaces `%var%` patterns in the string with environment variable values.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils.withEnvironmentVariable`.
    #[must_use]
    pub fn with_environment_variable(s: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;
        for caps in ENV_PATTERN.captures_iter(s) {
            let m = caps.get(0).unwrap_or_else(|| panic!("regex match group 0"));
            result.push_str(&s[last_end..m.start()]);
            if let Some(var) = caps.get(1) {
                if let Some(value) = Self::getenv(var.as_str()) {
                    result.push_str(&value);
                }
            }
            last_end = m.end();
        }
        result.push_str(&s[last_end..]);
        result
    }

    /// Gets an environment variable value, checking both system properties and env vars.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils.getenv`.
    #[must_use]
    pub fn getenv(var: &str) -> Option<String> {
        // Check std::env first
        if let Ok(value) = std::env::var(var) {
            if !value.is_empty() {
                return Some(Self::remove_quotes(&value));
            }
        }
        None
    }

    /// Returns a `ReadLine` for a non-standard include (from `/stdlib/` resources).
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils.getReaderNonstandardInclude`.
    pub fn get_reader_nonstandard_include(
        _s: &StringLocated,
        filename: &str,
    ) -> Option<Box<dyn ReadLine>> {
        let filename = if std::path::Path::new(filename)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("puml"))
        {
            filename.to_string()
        } else {
            format!("{filename}.puml")
        };

        // Try to read from stdlib resources
        let path = format!("stdlib/{filename}");
        std::fs::read(&path).map_or_else(
            |_| None,
            |data| {
                let description = format!("[{filename}]");
                Some(Box::new(ReadLineReader::from_bytes(&data, &description)) as Box<dyn ReadLine>)
            },
        )
    }

    /// Returns a `ReadLine` for a stdlib include.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils.getReaderStdlibInclude`.
    pub fn get_reader_stdlib_include(
        s: &StringLocated,
        filename: &str,
    ) -> Result<Option<Box<dyn ReadLine>>, EaterException> {
        let puml = Stdlib::get_puml_resource(filename);
        if puml.is_none() {
            return Ok(None);
        }
        let puml = puml.unwrap_or_default();
        let description = format!("<{filename}>");
        match DiagramDetector::extract_from_bytes(&puml, &description) {
            Ok(Some(tmp)) => Ok(Some(tmp)),
            Ok(None) => Ok(Some(Box::new(ReadLineReader::from_bytes(&puml, &description)))),
            Err(e) => {
                let error_msg = e.to_string();
                Ok(Some(Box::new(ReadLineSimple::new(s.clone(), error_msg))))
            }
        }
    }

    /// Returns a `ReadLine` for a URL include.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils.getReaderIncludeUrl`.
    pub fn get_reader_include_url(
        url: &SURL,
        s: &StringLocated,
        suf: Option<&str>,
    ) -> Result<Box<dyn ReadLine>, EaterException> {
        match DiagramDetector::extract_from_url(url, s.clone(), suf) {
            Ok(Some(tmp)) => Ok(tmp),
            Ok(None) => Self::get_reader_include(url, s),
            Err(e) => Err(EaterException::new(
                format!("Cannot open URL {e}"),
                s,
            )),
        }
    }

    /// Returns a `ReadLine` for a URL.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorUtils.getReaderInclude`.
    pub fn get_reader_include(
        url: &SURL,
        s: &StringLocated,
    ) -> Result<Box<dyn ReadLine>, EaterException> {
        url.open_stream().map_or_else(
            || Err(EaterException::new("Cannot open URL", s)),
            |data| Ok(Box::new(ReadLineReader::from_bytes(&data, &url.to_string())) as Box<dyn ReadLine>),
        )
    }

    fn remove_quotes(s: &str) -> String {
        let s = s.trim();
        if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }
}

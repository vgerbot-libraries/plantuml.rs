//! A line of text with location information.
//!
//! Ported from `net.sourceforge.plantuml.text.StringLocated`.

use std::sync::LazyLock;

use crate::stubs::{FoxSignature, Jaws, LineLocation, StringUtils};
use crate::TLineType;

/// A line of text with location info (file, line number, error flag).
///
/// Ported from `net.sourceforge.plantuml.text.StringLocated`.
#[derive(Debug, Clone)]
pub struct StringLocated {
    s: String,
    location: LineLocation,
    preprocessor_error: Option<String>,
    trimmed: Option<Box<Self>>,
    fox: i64,
    #[allow(dead_code)]
    type_cache: Option<TLineType>,
}

impl StringLocated {
    /// Creates a new `StringLocated` with the given string and location.
    #[must_use]
    pub fn new(s: impl Into<String>, location: LineLocation) -> Self {
        Self {
            s: s.into(),
            location,
            preprocessor_error: None,
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }

    /// Creates a new `StringLocated` with a preprocessor error message.
    #[must_use]
    pub fn with_error(s: impl Into<String>, location: LineLocation, preprocessor_error: impl Into<String>) -> Self {
        Self {
            s: s.into(),
            location,
            preprocessor_error: Some(preprocessor_error.into()),
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }

    /// Returns the string content.
    #[must_use]
    pub fn get_string(&self) -> &str {
        &self.s
    }

    /// Returns the location info.
    #[must_use]
    pub fn get_location(&self) -> &LineLocation {
        &self.location
    }

    /// Returns the preprocessor error, if any.
    #[must_use]
    pub fn get_preprocessor_error(&self) -> Option<&str> {
        self.preprocessor_error.as_deref()
    }

    /// Returns a new `StringLocated` with the given preprocessor error.
    #[must_use]
    pub fn with_error_preprocessor(&self, preprocessor_error: impl Into<String>) -> Self {
        Self {
            s: self.s.clone(),
            location: self.location.clone(),
            preprocessor_error: Some(preprocessor_error.into()),
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }

    /// Returns a new `StringLocated` with the given string appended.
    #[must_use]
    pub fn append(&self, end_of_line: &str) -> Self {
        Self {
            s: format!("{}{}", self.s, end_of_line),
            location: self.location.clone(),
            preprocessor_error: self.preprocessor_error.clone(),
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }

    /// Returns a new `StringLocated` with the given char appended.
    #[must_use]
    pub fn append_char(&self, end_of_line: char) -> Self {
        self.append(&end_of_line.to_string())
    }

    /// Merges with the next line when the current line ends with a backslash.
    ///
    /// Ported from `StringLocated.mergeEndBackslash`.
    pub fn merge_end_backslash(&self, next: &Self) -> Self {
        Self {
            s: format!("{}{}", &self.s[..self.s.len() - 1], next.s),
            location: self.location.clone(),
            preprocessor_error: self.preprocessor_error.clone(),
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }

    /// Returns a substring of this `StringLocated`.
    #[must_use]
    pub fn substring(&self, start: usize, end: usize) -> Self {
        Self {
            s: self.s[start..end].to_string(),
            location: self.location.clone(),
            preprocessor_error: self.preprocessor_error.clone(),
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }

    /// Returns a substring from the given start position.
    #[must_use]
    pub fn substring_from(&self, start: usize) -> Self {
        self.substring(start, self.s.len())
    }

    /// Returns a trimmed version of this `StringLocated`.
    #[must_use]
    pub fn get_trimmed(&self) -> Self {
        if self.s.is_empty() {
            return self.clone();
        }
        if let Some(ref trimmed) = self.trimmed {
            return (**trimmed).clone();
        }
        let tmp = StringUtils::trin(&self.s);
        if tmp == self.s {
            self.clone()
        } else {
            Self {
                s: tmp,
                location: self.location.clone(),
                preprocessor_error: self.preprocessor_error.clone(),
                trimmed: None,
                fox: -1,
                type_cache: None,
            }
        }
    }

    /// Removes inner comments (`/' ... '/`).
    ///
    /// Ported from `StringLocated.removeInnerComment`.
    #[must_use]
    pub fn remove_inner_comment(&self) -> Self {
        let string = self.s.as_str();
        let replaced = string.replace('\t', " ");
        let trim = replaced.trim();
        if trim.starts_with("/'") {
            if let Some(idx) = string.find("'/") {
                return self.substring_from(idx + 2);
            }
        }
        if trim.ends_with("'/") {
            if let Some(idx) = string.rfind("/'") {
                return self.substring(0, idx);
            }
        }
        if trim.contains("/'''") && trim.contains("'''/") {
            return self.clone();
        }
        self.clone()
    }

    /// Returns the length of the string.
    #[must_use]
    pub fn length(&self) -> usize {
        self.s.len()
    }

    /// Returns the character at the given index.
    #[must_use]
    pub fn char_at(&self, i: usize) -> char {
        self.s.as_bytes().get(i).copied().map_or('\0', |b| b as char)
    }

    /// Returns the `TLineType` of this line.
    #[must_use]
    pub fn get_type(&self) -> TLineType {
        TLineType::get_from_line_internal(self)
    }

    /// Returns the fox signature of the string.
    #[must_use]
    pub fn get_fox_signature(&self) -> u64 {
        if self.fox == -1 {
            self.fox as u64
        } else {
            FoxSignature::get_fox_signature_from_real_string(&self.s)
        }
    }

    /// Returns `true` if the string contains an exclamation mark.
    #[must_use]
    pub fn contains_exclamation_mark(&self) -> bool {
        self.s.contains('!')
    }

    /// Finds the position of a multiline triple separator (`!!!`, `'''`, `"""`).
    #[must_use]
    pub fn find_multiline_triple_separator(&self) -> Option<usize> {
        let s = &self.s;
        for i in 0..s.len().saturating_sub(2) {
            let triple = &s[i..i + 3];
            if triple == "!!!" || triple == "'''" || triple == "\"\"\"" {
                return Some(i);
            }
        }
        None
    }

    /// Splits at a triple separator position.
    #[must_use]
    pub fn split_at_triple_separator(&self, x: usize) -> [Self; 2] {
        let s1 = &self.s[..x];
        let s2 = &self.s[x + 3..];
        [
            Self::new(s1, self.location.clone()),
            Self::new(s2, self.location.clone()),
        ]
    }

    /// Expands newlines (replaces `Jaws::BLOCK_E1_NEWLINE` with actual newlines).
    #[must_use]
    pub fn expands_newline(&self) -> Vec<Self> {
        self.s
            .split(Jaws::BLOCK_E1_NEWLINE)
            .map(|s| Self::new(s, self.location.clone()))
            .collect()
    }

    /// Replaces backslash with the Jaws real backslash character.
    #[must_use]
    pub fn jaws_hide_backslash(&self) -> Self {
        Self {
            s: self.s.replace('\\', &Jaws::BLOCK_E1_REAL_BACKSLASH.to_string()),
            location: self.location.clone(),
            preprocessor_error: self.preprocessor_error.clone(),
            trimmed: None,
            fox: -1,
            type_cache: None,
        }
    }
}

impl std::fmt::Display for StringLocated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.s.is_empty() {
            write!(f, "<<<EMPTY STRING>>>")
        } else {
            write!(f, "(SL) {}", self.s)
        }
    }
}

impl PartialEq for StringLocated {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}

impl Eq for StringLocated {}

impl std::hash::Hash for StringLocated {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.s.hash(state);
    }
}

#[allow(dead_code)]
static EXCLAMATION_MARK: LazyLock<u64> =
    LazyLock::new(|| FoxSignature::get_fox_signature_from_real_string("!"));

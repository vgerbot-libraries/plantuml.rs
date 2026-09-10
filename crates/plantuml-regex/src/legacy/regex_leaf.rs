//! RegexLeaf — a leaf regex pattern (single compiled pattern string).
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexLeaf` (Java).
//!
//! `RegexLeaf` wraps a raw regex pattern string with optional name and capture
//! count. It provides static factory methods for common patterns like
//! `space_one()`, `space_zero_or_more()`, `start()`, `end()`.

use std::collections::HashMap;

use super::iregex::IRegex;
use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// A leaf regex pattern.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexLeaf`.
pub struct RegexLeaf {
    pattern: String,
    name: Option<String>,
    count: usize,
}

impl RegexLeaf {
    /// Creates a leaf with the given pattern (no capture groups, no name).
    ///
    /// Ported from `RegexLeaf(String)`.
    pub fn new(regex: &str) -> Self {
        Self::with_count_name(0, None, regex)
    }

    /// Creates a leaf with the given capture count and pattern.
    ///
    /// Ported from `RegexLeaf(int, String)`.
    pub fn with_count(count: usize, regex: &str) -> Self {
        Self::with_count_name(count, None, regex)
    }

    /// Creates a leaf with the given capture count, name, and pattern.
    ///
    /// Ported from `RegexLeaf(int, String, String)`.
    pub fn with_count_name(count: usize, name: Option<&str>, regex: &str) -> Self {
        Self {
            pattern: regex.to_string(),
            name: name.map(|s| s.to_string()),
            count,
        }
    }

    /// `[%s]*` — zero or more spaces.
    ///
    /// Ported from `RegexLeaf.spaceZeroOrMore()`.
    pub fn space_zero_or_more() -> Self {
        Self::new("[%s]*")
    }

    /// `[%s]` — exactly one space.
    ///
    /// Ported from `RegexLeaf.spaceOne()`.
    pub fn space_one() -> Self {
        Self::new("[%s]")
    }

    /// `[%s]+` — one or more spaces.
    ///
    /// Ported from `RegexLeaf.spaceOneOrMore()`.
    pub fn space_one_or_more() -> Self {
        Self::new("[%s]+")
    }

    /// `^` — start of string.
    ///
    /// Ported from `RegexLeaf.start()`.
    pub fn start() -> Self {
        Self::new("^")
    }

    /// `$` — end of string.
    ///
    /// Ported from `RegexLeaf.end()`.
    pub fn end() -> Self {
        Self::new("$")
    }

    /// Returns the name of this leaf, if any.
    ///
    /// Ported from `RegexLeaf.getName()`.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl IRegex for RegexLeaf {
    fn get_pattern_as_string(&self) -> String {
        self.pattern.clone()
    }

    fn count(&self) -> usize {
        self.count
    }

    fn create_partial_match(
        &self,
        it: &mut dyn Iterator<Item = Option<String>>,
    ) -> HashMap<String, RegexPartialMatch> {
        let name = match &self.name {
            Some(n) => n.clone(),
            None => return HashMap::new(),
        };
        let mut m = RegexPartialMatch::new(&name);
        for _ in 0..self.count {
            m.add(it.next().flatten());
        }
        let mut result = HashMap::new();
        result.insert(name, m);
        result
    }

    fn r#match(&self, _full: &str) -> bool {
        // RegexLeaf.match(StringLocated) throws UnsupportedOperationException in Java
        unimplemented!("RegexLeaf::match is not supported")
    }

    fn matcher(&self, _full: &str) -> Option<RegexResult> {
        // RegexLeaf.matcher(String) throws UnsupportedOperationException in Java
        unimplemented!("RegexLeaf::matcher is not supported")
    }
}

impl std::fmt::Debug for RegexLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexLeaf")
            .field("pattern", &self.pattern)
            .field("name", &self.name)
            .field("count", &self.count)
            .finish()
    }
}

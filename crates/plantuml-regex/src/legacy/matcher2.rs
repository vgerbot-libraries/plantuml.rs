//! Matcher2 — wrapper around `regex::Captures` with position tracking.
//!
//! Ported from `net.sourceforge.plantuml.regex.Matcher2` (Java).
//!
//! `Matcher2` wraps a `regex::Regex` match against an input string. It provides
//! `find()`, `group(n)`, `group_count()`, `start()`, `end()` methods similar to
//! Java's `Matcher`.

use regex::{Captures, Regex};

/// Regex matcher wrapping `regex::Captures`.
///
/// Ported from `net.sourceforge.plantuml.regex.Matcher2`.
pub struct Matcher2<'a> {
    captures: Option<Captures<'a>>,
    input: &'a str,
    pos: usize,
    found: bool,
}

impl<'a> Matcher2<'a> {
    /// Builds a matcher from a compiled pattern and input.
    ///
    /// Ported from `Matcher2.build(Pattern, CharSequence, int)`.
    pub fn build(pattern: &Result<Regex, regex::Error>, input: &'a str, pos: usize) -> Matcher2<'a> {
        let captures = match pattern {
            Ok(re) => re.captures_at(input, pos),
            Err(_) => None,
        };
        Matcher2 {
            captures,
            input,
            pos,
            found: false,
        }
    }

    /// Attempts to find a match. Returns `true` if a match was found.
    ///
    /// Ported from `Matcher2.find()`.
    pub fn find(&mut self) -> bool {
        if self.captures.is_some() {
            self.found = true;
            true
        } else {
            false
        }
    }

    /// Returns `true` if the pattern matches the entire input.
    ///
    /// Ported from `Matcher2.matches()`.
    pub fn matches(&self) -> bool {
        match &self.captures {
            Some(c) => c.get(0).is_some_and(|m| m.start() == 0 && m.end() == self.input.len()),
            None => false,
        }
    }

    /// Returns the captured group at index `n`, or `None`.
    ///
    /// Ported from `Matcher2.group(int)`.
    pub fn group(&self, n: usize) -> Option<String> {
        self.captures
            .as_ref()
            .and_then(|c| c.get(n))
            .map(|m| m.as_str().to_string())
    }

    /// Returns the entire matched string, or `None`.
    ///
    /// Ported from `Matcher2.group()`.
    pub fn full_group(&self) -> Option<String> {
        self.group(0)
    }

    /// Returns the number of capture groups.
    ///
    /// Ported from `Matcher2.groupCount()`.
    pub fn group_count(&self) -> usize {
        self.captures
            .as_ref()
            .map(|c| c.len().saturating_sub(1))
            .unwrap_or(0)
    }

    /// Returns the start position of the match.
    ///
    /// Ported from `Matcher2.start()`.
    pub fn start(&self) -> usize {
        self.captures
            .as_ref()
            .and_then(|c| c.get(0))
            .map(|m| m.start())
            .unwrap_or(0)
    }

    /// Returns the end position of the match.
    ///
    /// Ported from `Matcher2.end()`.
    pub fn end(&self) -> usize {
        self.captures
            .as_ref()
            .and_then(|c| c.get(0))
            .map(|m| m.end())
            .unwrap_or(0)
    }
}

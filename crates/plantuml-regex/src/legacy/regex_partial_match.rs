//! Partial match result holding captured groups for a named capture.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexPartialMatch` (Java).

use std::fmt;

/// A collection of captured groups associated with a named capture group.
///
/// In the Java original, `RegexPartialMatch` stores a list of strings
/// (capture groups) and has a name (though the name is only used in the
/// constructor and not stored as a field — it is passed to the constructor
/// but never assigned). In Rust we follow the same semantics: the name is
/// not stored on the partial match itself; it is used as the key in the
/// `HashMap` that contains the partial match.
pub struct RegexPartialMatch {
    data: Vec<Option<String>>,
}

impl RegexPartialMatch {
    /// Creates a new, empty partial match.
    ///
    /// Ported from `RegexPartialMatch(String name)`. The Java constructor
    /// takes a name argument but does not store it.
    pub const fn new(_name: &str) -> Self {
        Self { data: Vec::new() }
    }

    /// Appends a captured group (which may be `None` if the group did not
    /// participate in the match).
    ///
    /// Ported from `RegexPartialMatch.add(String group)`.
    pub fn add(&mut self, group: Option<String>) {
        self.data.push(group);
    }

    /// Returns the number of captured groups.
    ///
    /// Ported from `RegexPartialMatch.size()`.
    pub const fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns the captured group at the given index, or `None` if the
    /// group did not participate in the match.
    ///
    /// Ported from `RegexPartialMatch.get(int i)`. In Java, `get` returns
    /// `null` for non-participating groups; in Rust we use `Option`.
    pub fn get(&self, i: usize) -> Option<&str> {
        self.data.get(i).and_then(|s| s.as_deref())
    }
}

impl fmt::Display for RegexPartialMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Ported from RegexPartialMatch.toString() which returns "{" + data + "}".
        write!(f, "{{")?;
        for (i, group) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            match group {
                Some(s) => write!(f, "{s}")?,
                None => write!(f, "null")?,
            }
        }
        write!(f, "}}")
    }
}

impl fmt::Debug for RegexPartialMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut m = RegexPartialMatch::new("test");
        assert_eq!(m.size(), 0);
        m.add(Some("hello".to_string()));
        m.add(None);
        m.add(Some("world".to_string()));
        assert_eq!(m.size(), 3);
        assert_eq!(m.get(0), Some("hello"));
        assert_eq!(m.get(1), None);
        assert_eq!(m.get(2), Some("world"));
    }

    #[test]
    fn test_display() {
        let mut m = RegexPartialMatch::new("test");
        m.add(Some("a".to_string()));
        m.add(Some("b".to_string()));
        assert_eq!(m.to_string(), "{a, b}");
    }
}

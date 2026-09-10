//! Pattern2 — compiled regex pattern with macro expansion.
//!
//! Ported from `net.sourceforge.plantuml.regex.Pattern2` (Java).
//!
//! `Pattern2` wraps a `regex::Regex` compiled from a pattern string that may
//! contain `%s`, `%q`, `%g`, `%pLN` macros. The macros are expanded before
//! compilation:
//! - `%s` → `\s\u{00A0}` (space or non-breaking space)
//! - `%q` → `'\u{2018}\u{2019}` (single quotes)
//! - `%g` → `"\u{201C}\u{201D}` (double quotes)
//! - `%pLN` → `\p{L}\p{N}` (Unicode letter or digit)

use std::sync::LazyLock;

use regex::Regex;

use super::matcher2::Matcher2;

/// Macro replacement table — longest macros first so `%pLN` is tried before
/// shorter prefixes.
static QUOTED_REPLACEMENTS: LazyLock<Vec<(&str, &str)>> = LazyLock::new(|| {
    vec![
        // %pLN → \p{L}\p{N} (Unicode letter or digit)
        ("%pLN", r"\p{L}\p{N}"),
        // %s → \s + non-breaking space (U+00A0)
        ("%s", r"\s\u{00A0}"),
        // %q → ' + left/right single quotes (U+2018, U+2019)
        ("%q", "\u{2018}\u{2019}"),
        // %g → " + left/right double quotes (U+201C, U+201D)
        // Using regular string with escapes because the pattern contains "
        ("%g", "\"\u{201C}\u{201D}"),
    ]
});

/// Transform pattern string, expanding `%s`, `%q`, `%g`, `%pLN` macros.
///
/// Ported from `Pattern2.transform(String)`.
pub fn transform(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input.as_bytes().get(i) == Some(&b'%') {
            let slice = &input[i..];
            let mut matched = false;
            for (macro_name, replacement) in QUOTED_REPLACEMENTS.iter() {
                if slice.starts_with(macro_name) {
                    result.push_str(replacement);
                    i += macro_name.len();
                    matched = true;
                    break;
                }
            }
            if !matched {
                let ch = input[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        } else {
            let ch = input[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    result
}

/// Compiled regex pattern with macro expansion.
///
/// Ported from `net.sourceforge.plantuml.regex.Pattern2`.
pub struct Pattern2 {
    pattern_string: String,
    pattern: Result<Regex, regex::Error>,
}

impl Pattern2 {
    /// Compiles a pattern string, expanding macros first.
    ///
    /// Ported from `Pattern2.cmpile(String)`.
    pub fn cmpile(p: &str) -> Pattern2 {
        let pattern_string = p.to_string();
        let pattern = Regex::new(&transform(&pattern_string));
        Pattern2 { pattern_string, pattern }
    }

    /// Returns the raw pattern string (before macro expansion).
    ///
    /// Ported from `Pattern2.pattern()`.
    pub fn pattern(&self) -> &str {
        &self.pattern_string
    }

    /// Creates a `Matcher2` for the given input at the given position.
    ///
    /// Ported from `Pattern2.matcher(CharSequence, int)`.
    pub fn matcher<'a>(&'a self, input: &'a str, pos: usize) -> Matcher2<'a> {
        Matcher2::build(&self.pattern, input, pos)
    }

    /// Compiles a pattern string into a `Regex`, expanding macros.
    ///
    /// Ported from `Pattern2.compileInternal(String)`.
    pub fn compile_internal(pattern_string: &str) -> Result<Regex, regex::Error> {
        Regex::new(&transform(pattern_string))
    }
}

impl Clone for Pattern2 {
    fn clone(&self) -> Self {
        Pattern2::cmpile(self.pattern())
    }
}

impl std::fmt::Debug for Pattern2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pattern2")
            .field("pattern_string", &self.pattern_string)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_percent_s() {
        // %s expands to \s + non-breaking space
        let result = transform("[%s]");
        assert_eq!(result, r"[\s\u{00A0}]");
    }

    #[test]
    fn transform_percent_q() {
        // %q expands to left/right single quotes
        let result = transform("%q");
        assert_eq!(result, "\u{2018}\u{2019}");
    }

    #[test]
    fn transform_percent_g() {
        // %g expands to " + left/right double quotes
        let result = transform("%g");
        assert_eq!(result, "\"\u{201C}\u{201D}");
    }

    #[test]
    fn transform_percent_pln() {
        assert_eq!(transform("%pLN"), r"\p{L}\p{N}");
    }

    #[test]
    fn transform_no_macros() {
        assert_eq!(transform("hello"), "hello");
    }

    #[test]
    fn transform_mixed() {
        let result = transform("[%s]+hello%q");
        // %s → \s\u{00A0}, %q → U+2018 U+2019
        assert_eq!(result, concat!(r"[\s\u{00A0}]+hello", "\u{2018}\u{2019}"));
    }

    #[test]
    fn compile_basic() {
        let p = Pattern2::cmpile("[%s]+");
        let mut m = p.matcher("  hello", 0);
        assert!(m.find());
    }
}

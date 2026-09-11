//! Start directive utilities.
//!
//! Ported from `net.sourceforge.plantuml.utils.StartUtils`.

/// Utilities for detecting `PlantUML` start/end/pause/exit directives.
///
/// Ported from `net.sourceforge.plantuml.utils.StartUtils`.
pub struct StartUtils;

impl StartUtils {
    /// Returns `true` if the line is a `@start` directive.
    ///
    /// A start directive begins with optional whitespace followed by `@` or `\`,
    /// then `start`, then at least one more character.
    pub fn is_start_directive(s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let mut i = 0;
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= n {
            return false;
        }
        let c = chars[i];
        if c != '@' && c != '\\' {
            return false;
        }
        // Need '@' + "start" + at least one char after
        i + 6 < n && s[i + 1..].starts_with("start")
    }

    /// Returns `true` if the line is an `@end` directive.
    pub fn is_end_directive(s: &str) -> bool {
        Self::starts_with_directive_keyword(s, "end")
    }

    /// Returns `true` if the line is a `@pause` directive.
    pub fn is_pause_directive(s: &str) -> bool {
        Self::starts_with_directive_keyword(s, "pause")
    }

    /// Returns `true` if the line is an `@unpause` directive.
    pub fn is_unpause_directive(s: &str) -> bool {
        Self::starts_with_directive_keyword(s, "unpause")
    }

    /// Returns `true` if the line is `!exit` (trimmed).
    pub fn is_exit(s: &str) -> bool {
        let trimmed = s.trim();
        trimmed == "!exit"
    }

    /// Extracts the text before `@start` in a line, or `None` if there's no
    /// valid start directive.
    pub fn before_start_uml(s: &str) -> Option<String> {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let mut inside = false;

        for i in 0..n {
            if Self::starts_with_directive_keyword_at(s, i, "start") {
                return Some(s[..i].to_string());
            }

            let c = chars[i];
            if inside {
                if c == '>' {
                    inside = false;
                }
                continue;
            }

            if c == '<' {
                inside = true;
            } else if Self::is_word_or_tilde(c) {
                return None;
            }
        }
        None
    }

    fn is_word_or_tilde(c: char) -> bool {
        c == '~' || c.is_alphanumeric() || c == '_'
    }

    fn starts_with_directive_keyword(text: &str, keyword: &str) -> bool {
        Self::starts_with_directive_keyword_at(text, 0, keyword)
    }

    fn starts_with_directive_keyword_at(text: &str, from: usize, keyword: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let mut i = from;

        while i < n {
            let c = chars[i];
            if c.is_whitespace() {
                i += 1;
                continue;
            }

            if c != '@' && c != '\\' {
                return false;
            }

            let kw_len = keyword.len();
            let start = i + 1;
            if start + kw_len > n {
                return false;
            }

            return text[start..].starts_with(keyword);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_start_directive() {
        assert!(StartUtils::is_start_directive("@startuml"));
        assert!(StartUtils::is_start_directive("  @startuml"));
        assert!(StartUtils::is_start_directive("@startjson"));
        assert!(!StartUtils::is_start_directive("@start"));
    }

    #[test]
    fn test_is_end_directive() {
        assert!(StartUtils::is_end_directive("@enduml"));
        assert!(StartUtils::is_end_directive("  @enduml"));
        assert!(!StartUtils::is_end_directive("@startuml"));
    }

    #[test]
    fn test_is_exit() {
        assert!(StartUtils::is_exit("!exit"));
        assert!(StartUtils::is_exit("  !exit  "));
        assert!(!StartUtils::is_exit("!exitnow"));
    }
}

//! Line type classification for preprocessor directives.
//!
//! Ported from `net.sourceforge.plantuml.text.TLineType`.

use std::sync::LazyLock;

use regex::Regex;

use crate::StringLocated;

/// The type of a preprocessor line.
///
/// Ported from `net.sourceforge.plantuml.text.TLineType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TLineType {
    Plain,
    AffectationDefine,
    Affectation,
    Assert,
    If,
    Ifdef,
    Undef,
    Ifndef,
    Else,
    ElseIf,
    Endif,
    While,
    Endwhile,
    Foreach,
    Endforeach,
    DeclareReturnFunction,
    DeclareProcedure,
    EndFunction,
    Return,
    LegacyDefine,
    LegacyDefineLong,
    Theme,
    Include,
    IncludeDef,
    Import,
    Startsub,
    Endsub,
    Includesub,
    Log,
    DumpMemory,
    CommentSimple,
    CommentLongStart,
    Option,
}

// ---------------------------------------------------------------------------
// Regex patterns for line type detection
// ---------------------------------------------------------------------------

static IDENTIFIER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\p{L}_][\p{L}_0-9]*$").unwrap_or_else(|_| Regex::new(r"^[A-Za-z_][A-Za-z_0-9]*$").unwrap())
});

static PATTERN_LEGACY_DEFINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!define\s+[\p{L}_][\p{L}_0-9]*\(")
        .unwrap_or_else(|_| Regex::new(r"^\s*!define\s+[A-Za-z_][A-Za-z_0-9]*\(").unwrap())
});

static PATTERN_LEGACY_DEFINELONG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!definelong\s+[\p{L}_][\p{L}_0-9]*\b")
        .unwrap_or_else(|_| Regex::new(r"^\s*!definelong\s+[A-Za-z_][A-Za-z_0-9]*").unwrap())
});

static PATTERN_AFFECTATION_DEFINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!define\s+[\p{L}_][\p{L}_0-9]*\b")
        .unwrap_or_else(|_| Regex::new(r"^\s*!define\s+[A-Za-z_][A-Za-z_0-9]*").unwrap())
});

static PATTERN_AFFECTATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!\s*(local|global)?\s*\$?[\p{L}_][\p{L}_0-9]*\s*\??=")
        .unwrap_or_else(|_| Regex::new(r"^\s*!\s*(local|global)?\s*\$?[A-Za-z_][A-Za-z_0-9]*\s*\??=").unwrap())
});

static PATTERN_COMMENT_SIMPLE1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*'").unwrap());

static PATTERN_COMMENT_SIMPLE2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*/'.*'/\s*$").unwrap());

static PATTERN_COMMENT_LONG_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*/'").unwrap());

static PATTERN_DECLARE_RETURN_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!(unquoted\s|final\s)*function\s+\$?[\p{L}_][\p{L}_0-9]*")
        .unwrap_or_else(|_| Regex::new(r"^\s*!(unquoted\s|final\s)*function\s+\$?[A-Za-z_][A-Za-z_0-9]*").unwrap())
});

static PATTERN_DECLARE_PROCEDURE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!(unquoted\s|final\s)*procedure\s+\$?[\p{L}_][\p{L}_0-9]*")
        .unwrap_or_else(|_| Regex::new(r"^\s*!(unquoted\s|final\s)*procedure\s+\$?[A-Za-z_][A-Za-z_0-9]*").unwrap())
});

static PATTERN_END_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!end\s*(function|definelong|procedure)\b")
        .unwrap_or_else(|_| Regex::new(r"^\s*!end\s*(function|definelong|procedure)").unwrap())
});

static PATTERN_INCLUDE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*!include\s*(url|_many|_once)?\b")
        .unwrap_or_else(|_| Regex::new(r"^\s*!include\s*(url|_many|_once)?").unwrap())
});

static ONLY_WHITESPACE_NON_EMPTY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s+$").unwrap());

/// Helper: matches a simple keyword pattern at the start of a line.
fn matches_keyword(s: &str, keyword: &str) -> bool {
    let trimmed = s.trim_start();
    if let Some(rest) = trimmed.strip_prefix(keyword) {
        rest.is_empty() || rest.starts_with(|c: char| !c.is_alphanumeric())
    } else {
        false
    }
}

impl TLineType {
    /// Determines the `TLineType` from a `StringLocated`.
    ///
    /// Ported from `TLineType.getFromLineInternal`.
    pub fn get_from_line_internal(sl: &StringLocated) -> Self {
        let s = sl.get_string();

        if PATTERN_COMMENT_SIMPLE1.is_match(s) {
            return Self::CommentSimple;
        }
        if PATTERN_COMMENT_SIMPLE2.is_match(s) {
            return Self::CommentSimple;
        }
        if PATTERN_COMMENT_LONG_START.is_match(s) && !s.contains("'/") {
            return Self::CommentLongStart;
        }
        if !sl.contains_exclamation_mark() {
            return Self::Plain;
        }
        if PATTERN_LEGACY_DEFINE.is_match(s) {
            return Self::LegacyDefine;
        }
        if PATTERN_LEGACY_DEFINELONG.is_match(s) {
            return Self::LegacyDefineLong;
        }
        if PATTERN_AFFECTATION_DEFINE.is_match(s) {
            return Self::AffectationDefine;
        }
        if PATTERN_AFFECTATION.is_match(s) {
            return Self::Affectation;
        }
        if matches_keyword(s, "!ifdef") {
            return Self::Ifdef;
        }
        if matches_keyword(s, "!undef") {
            return Self::Undef;
        }
        if matches_keyword(s, "!ifndef") {
            return Self::Ifndef;
        }
        if matches_keyword(s, "!assert") {
            return Self::Assert;
        }
        if matches_keyword(s, "!if") {
            return Self::If;
        }
        if PATTERN_DECLARE_RETURN_FUNCTION.is_match(s) {
            return Self::DeclareReturnFunction;
        }
        if PATTERN_DECLARE_PROCEDURE.is_match(s) {
            return Self::DeclareProcedure;
        }
        if matches_keyword(s, "!else") {
            return Self::Else;
        }
        if matches_keyword(s, "!elseif") {
            return Self::ElseIf;
        }
        if matches_keyword(s, "!endif") {
            return Self::Endif;
        }
        if matches_keyword(s, "!while") {
            return Self::While;
        }
        if matches_keyword(s, "!endwhile") {
            return Self::Endwhile;
        }
        if matches_keyword(s, "!foreach") {
            return Self::Foreach;
        }
        if matches_keyword(s, "!endfor") {
            return Self::Endforeach;
        }
        if PATTERN_END_FUNCTION.is_match(s) {
            return Self::EndFunction;
        }
        if matches_keyword(s, "!return") {
            return Self::Return;
        }
        if matches_keyword(s, "!theme") {
            return Self::Theme;
        }
        if PATTERN_INCLUDE.is_match(s) {
            return Self::Include;
        }
        if matches_keyword(s, "!includedef") {
            return Self::IncludeDef;
        }
        if matches_keyword(s, "!import") {
            return Self::Import;
        }
        if matches_keyword(s, "!startsub") {
            return Self::Startsub;
        }
        if matches_keyword(s, "!endsub") {
            return Self::Endsub;
        }
        if matches_keyword(s, "!includesub") {
            return Self::Includesub;
        }
        if matches_keyword(s, "!log") {
            return Self::Log;
        }
        if matches_keyword(s, "!dump_memory") {
            return Self::DumpMemory;
        }
        if matches_keyword(s, "!option") {
            return Self::Option;
        }
        Self::Plain
    }

    /// Returns `true` if the character is a quote (`"` or `'`).
    #[must_use]
    pub fn is_quote(ch: char) -> bool {
        ch == '"' || ch == '\''
    }

    /// Returns `true` if the character is a space character.
    #[must_use]
    pub fn is_space_char(ch: char) -> bool {
        ch.is_whitespace()
    }

    /// Returns `true` if the character is a Latin digit (0-9).
    #[must_use]
    pub fn is_latin_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    /// Returns `true` if the character is a letter, emoji, underscore, or digit.
    #[must_use]
    pub fn is_letter_or_emoji_or_underscore_or_digit(ch: char) -> bool {
        Self::is_letter_or_underscore(ch) || Self::is_latin_digit(ch) || Self::is_emoji(ch)
    }

    /// Returns `true` if the character is a letter, emoji, underscore, or dollar.
    #[must_use]
    pub fn is_letter_or_emoji_or_underscore_or_dollar(ch: char) -> bool {
        Self::is_letter_or_underscore(ch) || ch == '$' || Self::is_emoji(ch)
    }

    fn is_letter_or_underscore(ch: char) -> bool {
        Self::is_letter(ch) || ch == '_'
    }

    fn is_letter(ch: char) -> bool {
        ch.is_alphabetic()
    }

    fn is_emoji(ch: char) -> bool {
        // Surrogate range check (for Java compatibility)
        let code = ch as u32;
        (0xD800..=0xDFFF).contains(&code)
    }

    /// Returns `true` if the string is only whitespace (non-empty).
    #[must_use]
    pub fn is_only_whitespace_non_empty(s: &str) -> bool {
        ONLY_WHITESPACE_NON_EMPTY.is_match(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_quote() {
        assert!(TLineType::is_quote('"'));
        assert!(TLineType::is_quote('\''));
        assert!(!TLineType::is_quote('a'));
    }

    #[test]
    fn test_is_latin_digit() {
        assert!(TLineType::is_latin_digit('0'));
        assert!(TLineType::is_latin_digit('9'));
        assert!(!TLineType::is_latin_digit('a'));
    }
}

//! Raw character class definitions (e.g. `〴w` = word, `〴d` = digit).
//!
//! Ported from: `com/plantuml/ubrex/CharClassRaw.java`
use super::text_navigator::TextNavigator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharClassRaw {
    Any,
    Space,
    Guillemet,
    Word,
    Digit,
    AlphaNumeric,
    Letter,
}

impl CharClassRaw {
    /// Returns the syntax string for this class.
    const fn syntax(self) -> &'static str {
        match self {
            Self::Any => ".",
            Self::Space => "s",
            Self::Guillemet => "g",
            Self::Word => "w",
            Self::Digit => "d",
            Self::AlphaNumeric => "an",
            Self::Letter => "le",
        }
    }

    /// Tests whether `ch` matches this character class.
    pub fn internal_matches(&self, ch: char) -> bool {
        match self {
            Self::Any => true,
            Self::Space => ch == ' ',
            Self::Guillemet => ch == '"' || ch == '\u{201C}' || ch == '\u{201D}',
            Self::Word => {
                ch.is_ascii_lowercase()
                    || ch.is_ascii_uppercase()
                    || ch.is_ascii_digit()
                    || ch == '_'
            }
            Self::Digit => ch.is_ascii_digit() || ch == '_',
            Self::AlphaNumeric => ch.is_alphanumeric(),
            Self::Letter => ch.is_alphabetic(),
        }
    }

    /// Returns the number of characters in the definition syntax.
    pub fn definition_length(&self) -> usize {
        self.syntax().chars().count()
    }

    /// Parses a `CharClassRaw` from the first character(s) of `nav`.
    pub fn from_definition(nav: &TextNavigator) -> Self {
        let ch = nav.char_at(0);
        match ch {
            'S' | 's' => Self::Space,
            'G' | 'g' => Self::Guillemet,
            'D' | 'd' => Self::Digit,
            'W' | 'w' => Self::Word,
            'A' | 'a' => Self::AlphaNumeric,
            'L' | 'l' => Self::Letter,
            '.' => Self::Any,
            _ => panic!("wip01class {ch}"),
        }
    }

    /// Returns `true` if any class in `set` matches `ch`.
    pub fn internal_matches_any(set: &[Self], ch: char) -> bool {
        set.iter().any(|c| c.internal_matches(ch))
    }

    /// Returns the name for display.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Any => "ANY",
            Self::Space => "SPACE",
            Self::Guillemet => "GUILLEMET",
            Self::Word => "WORD",
            Self::Digit => "DIGIT",
            Self::AlphaNumeric => "ALPHA_NUMERIC",
            Self::Letter => "LETTER",
        }
    }
}

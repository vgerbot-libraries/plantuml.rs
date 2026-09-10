/// Raw character class definitions (e.g. `〴w` = word, `〴d` = digit).
///
/// Ported from: `com/plantuml/ubrex/CharClassRaw.java`

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
    fn syntax(&self) -> &'static str {
        match self {
            CharClassRaw::Any => ".",
            CharClassRaw::Space => "s",
            CharClassRaw::Guillemet => "g",
            CharClassRaw::Word => "w",
            CharClassRaw::Digit => "d",
            CharClassRaw::AlphaNumeric => "an",
            CharClassRaw::Letter => "le",
        }
    }

    /// Tests whether `ch` matches this character class.
    pub fn internal_matches(&self, ch: char) -> bool {
        match self {
            CharClassRaw::Any => true,
            CharClassRaw::Space => ch == ' ',
            CharClassRaw::Guillemet => ch == '"' || ch == '\u{201C}' || ch == '\u{201D}',
            CharClassRaw::Word => {
                ('a'..='z').contains(&ch)
                    || ('A'..='Z').contains(&ch)
                    || ('0'..='9').contains(&ch)
                    || ch == '_'
            }
            CharClassRaw::Digit => ('0'..='9').contains(&ch) || ch == '_',
            CharClassRaw::AlphaNumeric => ch.is_alphanumeric(),
            CharClassRaw::Letter => ch.is_alphabetic(),
        }
    }

    /// Returns the number of characters in the definition syntax.
    pub fn definition_length(&self) -> usize {
        self.syntax().chars().count()
    }

    /// Parses a `CharClassRaw` from the first character(s) of `nav`.
    pub fn from_definition(nav: &TextNavigator) -> CharClassRaw {
        let ch = nav.char_at(0);
        match ch {
            'S' | 's' => CharClassRaw::Space,
            'G' | 'g' => CharClassRaw::Guillemet,
            'D' | 'd' => CharClassRaw::Digit,
            'W' | 'w' => CharClassRaw::Word,
            'A' | 'a' => CharClassRaw::AlphaNumeric,
            'L' | 'l' => CharClassRaw::Letter,
            '.' => CharClassRaw::Any,
            _ => panic!("wip01class {ch}"),
        }
    }

    /// Returns `true` if any class in `set` matches `ch`.
    pub fn internal_matches_any(set: &[CharClassRaw], ch: char) -> bool {
        set.iter().any(|c| c.internal_matches(ch))
    }

    /// Returns the name for display.
    pub fn name(&self) -> &'static str {
        match self {
            CharClassRaw::Any => "ANY",
            CharClassRaw::Space => "SPACE",
            CharClassRaw::Guillemet => "GUILLEMET",
            CharClassRaw::Word => "WORD",
            CharClassRaw::Digit => "DIGIT",
            CharClassRaw::AlphaNumeric => "ALPHA_NUMERIC",
            CharClassRaw::Letter => "LETTER",
        }
    }
}

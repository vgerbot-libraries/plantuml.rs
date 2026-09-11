//! Challenge matching a character set (charset ranges + character classes).
//!
//! Ported from: `com/plantuml/ubrex/ChallengeCharSet.java`
use std::any::Any;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::char_class_raw::CharClassRaw;
use super::char_set::CharSet;
use super::text_navigator::TextNavigator;

pub struct ChallengeCharSet {
    char_classes: Vec<CharClassRaw>,
    char_set: CharSet,
    reversed: bool,
}

impl Default for ChallengeCharSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ChallengeCharSet {
    pub const fn new() -> Self {
        Self {
            char_classes: Vec::new(),
            char_set: CharSet::new(),
            reversed: false,
        }
    }

    /// Builds a `ChallengeCharSet` from a pattern string.
    pub fn build(pattern: &str) -> Self {
        assert!(!pattern.is_empty(), "Empty!");

        let pattern_chars: Vec<char> = pattern.chars().collect();
        assert!(pattern_chars.last() != Some(&'〜'), "Range operator '〜' must be followed by a character.");

        let mut result = Self::new();
        let mut nav = TextNavigator::build(pattern);

        while nav.length() > 0 {
            let ch = nav.char_at(0);
            if ch == '〤' {
                result.reversed = true;
                nav.jump(1);
                continue;
            }
            if ch == ' ' {
                nav.jump(1);
                continue;
            }

            if ch == '〴' {
                nav.jump(1);
                let char_class = CharClassRaw::from_definition(&nav);
                result.char_classes.push(char_class);
                nav.jump(char_class.definition_length());
            } else if nav.length() > 2 && nav.char_at(1) == '〜' {
                nav.jump(2);
                let end = nav.char_at(0);
                result.add_range(ch, end);
            } else if ch == '〃' {
                result.add_char('"');
            } else if ch == '∙' {
                result.add_char(' ');
            } else {
                result.add_char(ch);
            }
            nav.jump(1);
        }

        result
    }

    pub fn add_char(&mut self, ch: char) {
        self.char_set.add_char(ch);
    }

    pub fn add_range(&mut self, start: char, end: char) {
        self.char_set.add_range(start, end);
    }
}

impl Challenge for ChallengeCharSet {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        if string.length() == position {
            return ChallengeResult::no_match();
        }
        let ch = string.char_at(position);
        let matched = self.char_set.contains(ch) || CharClassRaw::internal_matches_any(&self.char_classes, ch);
        if matched == self.reversed {
            ChallengeResult::no_match()
        } else {
            ChallengeResult::one()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeCharSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeCharSet")
    }
}

//! Matches a single character (case-insensitive by default).
//!
//! Ported from: `com/plantuml/ubrex/ChallengeSingleChar.java`
use std::any::Any;

use super::case_mode::CaseMode;
use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::text_navigator::TextNavigator;

pub struct ChallengeSingleChar {
    ch: char,
}

impl ChallengeSingleChar {
    pub fn new(ch: char) -> Self {
        // Reject special bracket characters
        match ch {
            '┇' | '〴' | '「' | '」' | '〤' | '〜' | '〇' | '〄' | '〶' | '〘' | '〙' | '【' | '】' => {
                panic!("Illegal character in ChallengeSingleChar: {ch}");
            }
            _ => {}
        }
        // Case-insensitive
        let ch = CaseMode::ensure_lowercase(ch);
        Self { ch }
    }
}

impl Challenge for ChallengeSingleChar {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        if string.length() == position {
            return ChallengeResult::no_match();
        }
        let extract = string.char_at(position);
        let extract = CaseMode::ensure_lowercase(extract);
        if extract == self.ch {
            ChallengeResult::one()
        } else {
            ChallengeResult::no_match()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeSingleChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.ch)
    }
}

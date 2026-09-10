/// Challenge matching a character class (`〴w`, `〴d`, etc.).
///
/// Ported from: `com/plantuml/ubrex/ChallengeCharClass.java`

use std::any::Any;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::char_class::CharClass;
use super::text_navigator::TextNavigator;

pub struct ChallengeCharClass {
    char_class: CharClass,
}

impl ChallengeCharClass {
    pub fn new(char_class: CharClass) -> Self {
        ChallengeCharClass { char_class }
    }
}

impl Challenge for ChallengeCharClass {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        if string.length() == position {
            return ChallengeResult::no_match();
        }
        let ch = string.char_at(position);
        if self.char_class.matches(ch) {
            ChallengeResult::one()
        } else {
            ChallengeResult::no_match()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeCharClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char_class.name())
    }
}

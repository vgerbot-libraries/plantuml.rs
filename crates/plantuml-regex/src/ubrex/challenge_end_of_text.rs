/// End-of-text assertion: matches zero-width at the end of the text.
///
/// Ported from: `com/plantuml/ubrex/ChallengeEndOfText.java`

use std::any::Any;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::text_navigator::TextNavigator;

pub struct ChallengeEndOfText;

impl Challenge for ChallengeEndOfText {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        if position >= string.length() {
            ChallengeResult::zero()
        } else {
            ChallengeResult::no_match()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeEndOfText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeEndOfText")
    }
}

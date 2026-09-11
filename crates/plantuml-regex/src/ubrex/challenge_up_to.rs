//! Up-to challenge: skips characters until `origin` matches.
//!
//! Ported from: `com/plantuml/ubrex/ChallengeUpTo.java`
use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::text_navigator::TextNavigator;

pub struct ChallengeUpTo {
    origin: Rc<dyn Challenge>,
}

impl ChallengeUpTo {
    pub fn new(origin: Rc<dyn Challenge>) -> Self {
        Self { origin }
    }
}

impl Challenge for ChallengeUpTo {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut current_pos = position;
        loop {
            let tmp = self.origin.run_challenge(string, current_pos);
            if tmp.full_capture_length >= 0 {
                return ChallengeResult::new((current_pos - position) as i32);
            }
            current_pos += 1;
            if current_pos > string.length() {
                return ChallengeResult::no_match();
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeUpTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeUpTo:{}", self.origin)
    }
}

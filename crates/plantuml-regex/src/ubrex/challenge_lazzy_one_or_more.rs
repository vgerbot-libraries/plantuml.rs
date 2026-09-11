//! Lazy one-or-more quantifier: matches `origin` one or more times, stopping
//! as soon as `stop_condition` matches (without consuming it).
//!
//! Ported from: `com/plantuml/ubrex/ChallengeLazzyOneOrMore.java`
use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::text_navigator::TextNavigator;

pub struct ChallengeLazzyOneOrMore {
    origin: Rc<dyn Challenge>,
    stop_condition: Rc<dyn Challenge>,
}

impl ChallengeLazzyOneOrMore {
    pub fn new(origin: Rc<dyn Challenge>, stop_condition: Rc<dyn Challenge>) -> Self {
        Self {
            origin,
            stop_condition,
        }
    }
}

impl Challenge for ChallengeLazzyOneOrMore {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut capture = Capture::empty();
        let mut current_pos = position;

        loop {
            let match1 = self.origin.run_challenge(string, current_pos);
            if match1.full_capture_length < 0 {
                return ChallengeResult::no_match();
            }
            assert!(match1.full_capture_length != 0, "infinite loop in ChallengeLazzyOneOrMore");
            capture = capture.merge(&match1.capture);
            current_pos += match1.full_capture_length as usize;

            // Only peek at the stop condition, don't consume it.
            let match2 = self.stop_condition.run_challenge(string, current_pos);
            if match2.full_capture_length >= 0 {
                return ChallengeResult::with_capture(
                    (current_pos - position) as i32,
                    capture,
                );
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeLazzyOneOrMore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeLazzyOneOrMore:{}", self.origin)
    }
}

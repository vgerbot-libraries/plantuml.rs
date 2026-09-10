/// Zero-or-more quantifier: matches `origin` zero or more times (greedy).
///
/// Ported from: `com/plantuml/ubrex/ChallengeZeroOrMore.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::text_navigator::TextNavigator;

pub struct ChallengeZeroOrMore {
    origin: Rc<dyn Challenge>,
}

impl ChallengeZeroOrMore {
    pub fn new(origin: Rc<dyn Challenge>) -> Self {
        ChallengeZeroOrMore { origin }
    }
}

impl Challenge for ChallengeZeroOrMore {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut capture = Capture::empty();
        let mut current_pos = position;

        loop {
            let tmp = self.origin.run_challenge(string, current_pos);
            if tmp.full_capture_length < 0 {
                return ChallengeResult::with_capture(
                    (current_pos - position) as i32,
                    capture,
                );
            }
            if tmp.full_capture_length == 0 {
                panic!("infinite loop in ChallengeZeroOrMore");
            }
            capture = capture.merge(&tmp.capture);
            current_pos += tmp.full_capture_length as usize;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeZeroOrMore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeZeroOrMore:{}", self.origin)
    }
}

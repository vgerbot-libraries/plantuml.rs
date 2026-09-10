/// One-or-more quantifier: matches `origin` one or more times (greedy).
///
/// Ported from: `com/plantuml/ubrex/ChallengeOneOrMore.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::text_navigator::TextNavigator;

pub struct ChallengeOneOrMore {
    origin: Rc<dyn Challenge>,
}

impl ChallengeOneOrMore {
    pub fn new(origin: Rc<dyn Challenge>) -> Self {
        ChallengeOneOrMore { origin }
    }
}

impl Challenge for ChallengeOneOrMore {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut capture = Capture::empty();
        let mut current_pos = position;

        loop {
            let shall_we_pass = self.origin.run_challenge(string, current_pos);
            if shall_we_pass.full_capture_length < 0 {
                if current_pos > position {
                    return ChallengeResult::with_capture(
                        (current_pos - position) as i32,
                        capture,
                    );
                } else {
                    return ChallengeResult::no_match();
                }
            }
            if shall_we_pass.full_capture_length == 0 {
                panic!("infinite loop in ChallengeOneOrMore");
            }
            capture = capture.merge(&shall_we_pass.capture);
            current_pos += shall_we_pass.full_capture_length as usize;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeOneOrMore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeOneOrMore:{}", self.origin)
    }
}

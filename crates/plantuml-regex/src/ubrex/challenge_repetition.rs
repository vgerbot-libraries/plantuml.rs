/// Repetition quantifier: matches `origin` a number of times specified by a
/// `Repetition` spec (e.g. `{3}`, `{2-5}`, `{3+}`).
///
/// Ported from: `com/plantuml/ubrex/ChallengeRepetition.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::repetition::Repetition;
use super::text_navigator::TextNavigator;

pub struct ChallengeRepetition {
    origin: Rc<dyn Challenge>,
    repetition: Repetition,
}

impl ChallengeRepetition {
    pub fn new(repetition: Repetition, origin: Rc<dyn Challenge>) -> Self {
        ChallengeRepetition { origin, repetition }
    }
}

impl Challenge for ChallengeRepetition {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut capture = Capture::empty();
        let mut current_pos = position;
        let mut count: i32 = 0;

        loop {
            let shall_we_pass = self.origin.run_challenge(string, current_pos);
            if shall_we_pass.full_capture_length < 0 {
                if current_pos > position && self.repetition.matches(count) {
                    return ChallengeResult::with_capture(
                        (current_pos - position) as i32,
                        capture,
                    );
                } else {
                    return ChallengeResult::no_match();
                }
            }
            if shall_we_pass.full_capture_length == 0 {
                panic!("infinite loop in ChallengeRepetition");
            }
            capture = capture.merge(&shall_we_pass.capture);
            current_pos += shall_we_pass.full_capture_length as usize;
            count += 1;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeRepetition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeRepetition:{}", self.origin)
    }
}

/// One-or-more-up-to challenge (old version): matches `origin` one or more
/// times until `end` matches at the current position.
///
/// Ported from: `com/plantuml/ubrex/ChallengeOneOrMoreUpToOldVersion.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::text_navigator::TextNavigator;

pub struct ChallengeOneOrMoreUpToOldVersion {
    origin: Rc<dyn Challenge>,
    end: Rc<dyn Challenge>,
}

impl ChallengeOneOrMoreUpToOldVersion {
    pub fn new(origin: Rc<dyn Challenge>, end: Rc<dyn Challenge>) -> Self {
        ChallengeOneOrMoreUpToOldVersion { origin, end }
    }
}

impl Challenge for ChallengeOneOrMoreUpToOldVersion {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut capture = Capture::empty();
        let mut current_pos = position;

        loop {
            let shall_we_pass = self.origin.run_challenge(string, current_pos);
            if shall_we_pass.full_capture_length < 0 {
                return ChallengeResult::no_match();
            }
            if shall_we_pass.full_capture_length == 0 {
                panic!("infinite loop in ChallengeOneOrMoreUpToOldVersion");
            }
            capture = capture.merge(&shall_we_pass.capture);
            current_pos += shall_we_pass.full_capture_length as usize;

            let tmp = self.end.run_challenge(string, current_pos);
            if tmp.full_capture_length >= 0 {
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

impl std::fmt::Display for ChallengeOneOrMoreUpToOldVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChallengeOneOrMoreUpToOldVersion:{}->{}",
            self.origin, self.end
        )
    }
}

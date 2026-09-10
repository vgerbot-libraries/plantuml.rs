/// Lookahead assertion: zero-width check that `origin` matches (or doesn't)
/// at the current position.
///
/// Ported from: `com/plantuml/ubrex/ChallengeLookAhead.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::look_around::LookAround;
use super::text_navigator::TextNavigator;

pub struct ChallengeLookAhead {
    origin: Rc<dyn Challenge>,
    ahead: LookAround,
}

impl ChallengeLookAhead {
    pub fn new(origin: Rc<dyn Challenge>, ahead: LookAround) -> Self {
        ChallengeLookAhead { origin, ahead }
    }
}

impl Challenge for ChallengeLookAhead {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let tmp = self
            .origin
            .run_challenge(string, position)
            .full_capture_length;
        match self.ahead {
            LookAround::LookAheadPositive => {
                if tmp < 0 {
                    ChallengeResult::no_match()
                } else {
                    ChallengeResult::zero()
                }
            }
            LookAround::LookAheadNegative => {
                if tmp < 0 {
                    ChallengeResult::zero()
                } else {
                    ChallengeResult::no_match()
                }
            }
            _ => panic!("ChallengeLookAhead with non-lookahead mode"),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeLookAhead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.ahead as u8, self.origin)
    }
}

/// Lookbehind assertion: zero-width check that `origin` matches (or doesn't)
/// when the text before the current position is reversed.
///
/// Ported from: `com/plantuml/ubrex/ChallengeLookBehind.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::look_around::LookAround;
use super::text_navigator::TextNavigator;

pub struct ChallengeLookBehind {
    origin: Rc<dyn Challenge>,
    ahead: LookAround,
}

impl ChallengeLookBehind {
    pub fn new(origin: Rc<dyn Challenge>, ahead: LookAround) -> Self {
        ChallengeLookBehind { origin, ahead }
    }
}

impl Challenge for ChallengeLookBehind {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let string_reversed = string.reverse(position);
        let tmp = self
            .origin
            .run_challenge(&string_reversed, 0)
            .full_capture_length;
        match self.ahead {
            LookAround::LookBehindPositive => {
                if tmp < 0 {
                    ChallengeResult::no_match()
                } else {
                    ChallengeResult::zero()
                }
            }
            LookAround::LookBehindNegative => {
                if tmp < 0 {
                    ChallengeResult::zero()
                } else {
                    ChallengeResult::no_match()
                }
            }
            _ => panic!("ChallengeLookBehind with non-lookbehind mode"),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeLookBehind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.ahead as u8, self.origin)
    }
}

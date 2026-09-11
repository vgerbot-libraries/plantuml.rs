//! Optional quantifier: matches `origin` zero or one times.
//!
//! Ported from: `com/plantuml/ubrex/ChallengeOptional.java`
use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::text_navigator::TextNavigator;

pub struct ChallengeOptional {
    origin: Rc<dyn Challenge>,
}

impl ChallengeOptional {
    pub fn new(origin: Rc<dyn Challenge>) -> Self {
        Self { origin }
    }
}

impl Challenge for ChallengeOptional {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let tmp = self.origin.run_challenge(string, position);
        if tmp.full_capture_length < 0 {
            ChallengeResult::zero()
        } else {
            tmp
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeOptional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChallengeOptional:{}", self.origin)
    }
}

/// Alternation challenge: matches the first alternative that succeeds.
///
/// Ported from: `com/plantuml/ubrex/ChallengeAlternative.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::text_navigator::TextNavigator;

pub struct ChallengeAlternative {
    alternatives: Vec<Rc<dyn Challenge>>,
}

impl ChallengeAlternative {
    pub fn new() -> Self {
        ChallengeAlternative {
            alternatives: Vec::new(),
        }
    }

    pub fn add_alternative(&mut self, pattern: Rc<dyn Challenge>) {
        self.alternatives.push(pattern);
    }
}

impl Challenge for ChallengeAlternative {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        for pattern in &self.alternatives {
            let tmp = pattern.run_challenge(string, position);
            if tmp.full_capture_length >= 0 {
                return tmp;
            }
        }
        ChallengeResult::no_match()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for ChallengeAlternative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ALTERNATIVE")
    }
}

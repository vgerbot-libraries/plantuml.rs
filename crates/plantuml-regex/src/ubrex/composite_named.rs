/// Named group challenge: wraps a list of challenges and captures the matched
/// text under a named key.
///
/// Ported from: `com/plantuml/ubrex/CompositeNamed.java`

use std::any::Any;
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::text_navigator::TextNavigator;

pub struct CompositeNamed {
    name: String,
    challenges: Vec<Rc<dyn Challenge>>,
}

impl CompositeNamed {
    pub fn new(name: String, challenge: Rc<dyn Challenge>) -> Self {
        CompositeNamed {
            name,
            challenges: vec![challenge],
        }
    }

    pub fn from_challenges(name: String, challenges: Vec<Rc<dyn Challenge>>) -> Self {
        CompositeNamed { name, challenges }
    }
}

impl Challenge for CompositeNamed {
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult {
        let mut capture = Capture::empty();
        let mut current = position;

        for challenge in &self.challenges {
            let shall_we_pass = challenge.run_challenge(string, current);
            if shall_we_pass.full_capture_length < 0 {
                return ChallengeResult::no_match();
            }
            current += shall_we_pass.full_capture_length as usize;
            capture = capture.merge(&shall_we_pass.capture);
        }

        let value = string.sub_sequence(position, current).to_string();
        capture = capture.with_prefixed_keys(&self.name);
        capture = capture.with_entry(&self.name, &value);

        ChallengeResult::with_capture((current - position) as i32, capture)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for CompositeNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) [", self.name)?;
        for (i, c) in self.challenges.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c)?;
        }
        write!(f, "]")
    }
}

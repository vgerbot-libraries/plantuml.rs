/// Composite challenge: a sequence of challenges matched left-to-right.
///
/// Also serves as the top-level parser via `parse_and_build`.
///
/// Ported from: `com/plantuml/ubrex/CompositeList.java`

use std::any::Any;
use std::rc::Rc;

use super::atomic_parser::AtomicParser;
use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::capture::Capture;
use super::text_navigator::TextNavigator;

#[derive(Clone)]
pub struct CompositeList {
    challenges: Vec<Rc<dyn Challenge>>,
}

impl CompositeList {
    pub fn new() -> Self {
        CompositeList {
            challenges: Vec::new(),
        }
    }

    pub fn add_challenge(&mut self, p: Rc<dyn Challenge>) {
        self.challenges.push(p);
    }

    pub fn create_empty() -> CompositeList {
        CompositeList::new()
    }

    /// Parses a ubrex pattern string into a `CompositeList`.
    pub fn parse_and_build(input: &str) -> CompositeList {
        if input.is_empty() {
            panic!("Cannot parse empty pattern");
        }
        let mut nav = TextNavigator::build(input);
        CompositeList::parse_and_build_from_text_navigator(&mut nav)
    }

    /// Parses from a mutable `TextNavigator`, consuming characters.
    pub fn parse_and_build_from_text_navigator(input: &mut TextNavigator) -> CompositeList {
        let mut result = CompositeList::new();
        result.parse_and_consume_now(input);
        result
    }

    fn parse_and_consume_now(&mut self, input: &mut TextNavigator) {
        let builder = AtomicParser::new();
        while input.length() > 0 {
            let ch = input.char_at(0);
            if ch == ' ' {
                input.jump(1);
            } else {
                for c in builder.parse(input) {
                    self.add_challenge(c);
                }
            }
        }
    }

    /// Returns the internal list of challenges.
    pub fn get_internal_challenges_list(&self) -> &[Rc<dyn Challenge>] {
        &self.challenges
    }
}

impl Challenge for CompositeList {
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

        ChallengeResult::with_capture((current - position) as i32, capture)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for CompositeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, c) in self.challenges.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", c)?;
        }
        write!(f, "]")
    }
}

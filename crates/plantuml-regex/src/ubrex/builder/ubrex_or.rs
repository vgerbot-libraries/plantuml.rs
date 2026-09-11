//! Alternation builder: matches the first of several `UBrexPart`s.
//!
//! Ported from: `com/plantuml/ubrex/builder/UBrexOr.java`
use std::rc::Rc;

use crate::ubrex::challenge_alternative::ChallengeAlternative;
use super::ubrex_part::UBrexPart;

pub struct UBrexOr {
    part: UBrexPart,
}

impl UBrexOr {
    pub fn new(parts: &[UBrexPart]) -> Self {
        let mut alt = ChallengeAlternative::new();
        for element in parts {
            alt.add_alternative(Rc::clone(element.get_challenge()));
        }
        Self {
            part: UBrexPart::new(Rc::new(alt)),
        }
    }

    pub const fn as_part(&self) -> &UBrexPart {
        &self.part
    }

    pub fn into_part(self) -> UBrexPart {
        self.part
    }
}

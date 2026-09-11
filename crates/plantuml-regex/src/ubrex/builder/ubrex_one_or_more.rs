//! One-or-more builder: wraps a `UBrexPart` in a one-or-more quantifier.
//!
//! Ported from: `com/plantuml/ubrex/builder/UBrexOneOrMore.java`
use std::rc::Rc;

use crate::ubrex::challenge_one_or_more::ChallengeOneOrMore;
use super::ubrex_part::UBrexPart;

pub struct UBrexOneOrMore {
    part: UBrexPart,
}

impl UBrexOneOrMore {
    pub fn new(origin: &UBrexPart) -> Self {
        let challenge = Rc::new(ChallengeOneOrMore::new(Rc::clone(origin.get_challenge())));
        Self {
            part: UBrexPart::new(challenge),
        }
    }

    pub const fn as_part(&self) -> &UBrexPart {
        &self.part
    }

    pub fn into_part(self) -> UBrexPart {
        self.part
    }
}

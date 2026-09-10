/// Zero-or-more builder: wraps a `UBrexPart` in a zero-or-more quantifier.
///
/// Ported from: `com/plantuml/ubrex/builder/UBrexZeroOrMore.java`

use std::rc::Rc;

use crate::ubrex::challenge_zero_or_more::ChallengeZeroOrMore;
use super::ubrex_part::UBrexPart;

pub struct UBrexZeroOrMore {
    part: UBrexPart,
}

impl UBrexZeroOrMore {
    pub fn new(origin: &UBrexPart) -> Self {
        let challenge = Rc::new(ChallengeZeroOrMore::new(Rc::clone(origin.get_challenge())));
        UBrexZeroOrMore {
            part: UBrexPart::new(challenge),
        }
    }

    pub fn as_part(&self) -> &UBrexPart {
        &self.part
    }

    pub fn into_part(self) -> UBrexPart {
        self.part
    }
}

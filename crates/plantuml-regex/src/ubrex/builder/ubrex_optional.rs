/// Optional builder: wraps a `UBrexPart` in an optional quantifier.
///
/// Ported from: `com/plantuml/ubrex/builder/UBrexOptional.java`

use std::rc::Rc;

use crate::ubrex::challenge_optional::ChallengeOptional;
use super::ubrex_part::UBrexPart;

pub struct UBrexOptional {
    part: UBrexPart,
}

impl UBrexOptional {
    pub fn new(origin: &UBrexPart) -> Self {
        let challenge = Rc::new(ChallengeOptional::new(Rc::clone(origin.get_challenge())));
        UBrexOptional {
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

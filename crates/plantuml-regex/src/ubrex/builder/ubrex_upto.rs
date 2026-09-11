//! Up-to builder: matches one or more of `what` until `stop` matches.
//!
//! Ported from: `com/plantuml/ubrex/builder/UBrexUpto.java`
use std::rc::Rc;

use crate::ubrex::challenge_one_or_more_up_to_old_version::ChallengeOneOrMoreUpToOldVersion;
use crate::ubrex::composite_list::CompositeList;
use super::ubrex_part::UBrexPart;

pub struct UBrexUpto {
    part: UBrexPart,
}

impl UBrexUpto {
    pub fn new(what: &UBrexPart, stop: &UBrexPart) -> Self {
        let p1 = Rc::clone(what.get_challenge());
        let p2 = Rc::clone(stop.get_challenge());
        let mut result = CompositeList::create_empty();
        result.add_challenge(Rc::new(ChallengeOneOrMoreUpToOldVersion::new(
            Rc::clone(&p1),
            Rc::clone(&p2),
        )));
        result.add_challenge(p2);
        Self {
            part: UBrexPart::new(Rc::new(result)),
        }
    }

    pub const fn as_part(&self) -> &UBrexPart {
        &self.part
    }

    pub fn into_part(self) -> UBrexPart {
        self.part
    }
}

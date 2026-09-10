/// Named group builder: wraps a `UBrexPart` in a named capture group.
///
/// Ported from: `com/plantuml/ubrex/builder/UBrexNamed.java`

use std::rc::Rc;

use crate::ubrex::composite_list::CompositeList;
use crate::ubrex::composite_named::CompositeNamed;
use super::ubrex_part::UBrexPart;

pub struct UBrexNamed {
    part: UBrexPart,
}

impl UBrexNamed {
    pub fn new(name: &str, origin: &UBrexPart) -> Self {
        let challenge = create(name, origin);
        UBrexNamed {
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

fn create(name: &str, part: &UBrexPart) -> Rc<dyn crate::ubrex::challenge::Challenge> {
    let challenge = part.get_challenge();

    // Check if the part is a CompositeList (e.g. from UBrexUpto)
    if let Some(composite_list) = challenge.as_any().downcast_ref::<CompositeList>() {
        let challenges: Vec<Rc<dyn crate::ubrex::challenge::Challenge>> = composite_list
            .get_internal_challenges_list()
            .iter()
            .map(|c| Rc::clone(c))
            .collect();
        return Rc::new(CompositeNamed::from_challenges(name.to_string(), challenges));
    }

    Rc::new(CompositeNamed::new(name.to_string(), Rc::clone(challenge)))
}

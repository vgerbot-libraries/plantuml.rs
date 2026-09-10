/// Concatenation builder: sequences multiple `UBrexPart`s.
///
/// Ported from: `com/plantuml/ubrex/builder/UBrexConcat.java`

use std::rc::Rc;

use crate::ubrex::composite_list::CompositeList;
use super::ubrex_part::UBrexPart;

pub struct UBrexConcat {
    part: UBrexPart,
}

impl UBrexConcat {
    pub fn new(parts: &[UBrexPart]) -> Self {
        let mut result = CompositeList::create_empty();
        for element in parts {
            result.add_challenge(Rc::clone(element.get_challenge()));
        }
        UBrexConcat {
            part: UBrexPart::new(Rc::new(result)),
        }
    }

    pub fn as_part(&self) -> &UBrexPart {
        &self.part
    }

    pub fn into_part(self) -> UBrexPart {
        self.part
    }
}

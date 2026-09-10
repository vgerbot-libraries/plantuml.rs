/// Leaf builder: compiles a ubrex pattern string into a `UBrexPart`.
///
/// Ported from: `com/plantuml/ubrex/builder/UBrexLeaf.java`

use crate::ubrex::composite_list::CompositeList;
use super::ubrex_part::UBrexPart;

pub struct UBrexLeaf {
    part: UBrexPart,
}

impl UBrexLeaf {
    pub fn new(definition: &str) -> Self {
        let challenge = CompositeList::parse_and_build(definition);
        UBrexLeaf {
            part: UBrexPart::new(std::rc::Rc::new(challenge)),
        }
    }

    /// Creates a leaf matching end-of-text (`〒$`).
    pub fn end() -> UBrexLeaf {
        UBrexLeaf::new("〒$")
    }

    /// Creates a leaf matching one or more spaces (`〇+〴s`).
    pub fn space_one_or_more() -> UBrexLeaf {
        UBrexLeaf::new("〇+〴s")
    }

    /// Creates a leaf matching zero or more spaces (`〇*〴s`).
    pub fn space_zero_or_more() -> UBrexLeaf {
        UBrexLeaf::new("〇*〴s")
    }

    pub fn as_part(&self) -> &UBrexPart {
        &self.part
    }

    pub fn into_part(self) -> UBrexPart {
        self.part
    }
}

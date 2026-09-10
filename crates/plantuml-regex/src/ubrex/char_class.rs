/// Character class combining a `CharClassRaw` with a `CharClassType` (normal/negative).
///
/// Ported from: `com/plantuml/ubrex/CharClass.java`

use super::char_class_raw::CharClassRaw;
use super::char_class_type::CharClassType;
use super::text_navigator::TextNavigator;

pub struct CharClass {
    char_class_raw: CharClassRaw,
    class_type: CharClassType,
}

impl CharClass {
    pub fn new(char_class_raw: CharClassRaw, class_type: CharClassType) -> Self {
        CharClass {
            char_class_raw,
            class_type,
        }
    }

    /// Tests whether `ch` matches this class.
    pub fn matches(&self, ch: char) -> bool {
        let raw_match = self.char_class_raw.internal_matches(ch);
        match self.class_type {
            CharClassType::Normal => raw_match,
            CharClassType::Negative => !raw_match,
        }
    }

    /// Parses a `CharClass` from the start of `nav`.
    /// If the first character is uppercase, the class is negative.
    pub fn from_definition(nav: &TextNavigator) -> CharClass {
        let ch = nav.char_at(0);
        let class_type = if ch.is_uppercase() {
            CharClassType::Negative
        } else {
            CharClassType::Normal
        };
        let raw = CharClassRaw::from_definition(nav);
        CharClass::new(raw, class_type)
    }

    /// Returns the display name.
    pub fn name(&self) -> String {
        format!("{}-{}", self.char_class_raw.name(), self.class_type)
    }

    /// Returns the number of characters consumed by the definition.
    pub fn definition_length(&self) -> usize {
        self.char_class_raw.definition_length()
    }
}

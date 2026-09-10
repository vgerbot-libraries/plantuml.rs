/// Look-around mode for lookahead/lookbehind assertions.
///
/// Ported from: `com/plantuml/ubrex/LookAround.java`

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookAround {
    LookAheadPositive,
    LookAheadNegative,
    LookBehindPositive,
    LookBehindNegative,
    EndOfText,
}

impl LookAround {
    /// Parses a look-around definition from the start of `input`.
    /// Returns `None` if the syntax is unrecognised.
    pub fn from(input: &super::text_navigator::TextNavigator) -> Option<LookAround> {
        let ch0 = input.char_at(0);
        match ch0 {
            '$' => Some(LookAround::EndOfText),
            '=' => Some(LookAround::LookAheadPositive),
            '!' => Some(LookAround::LookAheadNegative),
            '<' => {
                if input.length() > 1 {
                    let ch1 = input.char_at(1);
                    match ch1 {
                        '=' => Some(LookAround::LookBehindPositive),
                        '!' => Some(LookAround::LookBehindNegative),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_look_behind(&self) -> bool {
        matches!(self, LookAround::LookBehindPositive | LookAround::LookBehindNegative)
    }

    pub fn is_look_ahead(&self) -> bool {
        matches!(self, LookAround::LookAheadPositive | LookAround::LookAheadNegative)
    }

    /// Returns the number of characters consumed by the definition.
    pub fn definition_size(&self) -> usize {
        match self {
            LookAround::LookAheadPositive => 1, // "="
            LookAround::LookAheadNegative => 1, // "!"
            LookAround::LookBehindPositive => 2, // "<="
            LookAround::LookBehindNegative => 2, // "<!"
            LookAround::EndOfText => 1,         // "$"
        }
    }
}

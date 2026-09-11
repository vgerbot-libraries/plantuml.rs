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
    pub fn from(input: &super::text_navigator::TextNavigator) -> Option<Self> {
        let ch0 = input.char_at(0);
        match ch0 {
            '$' => Some(Self::EndOfText),
            '=' => Some(Self::LookAheadPositive),
            '!' => Some(Self::LookAheadNegative),
            '<' => {
                if input.length() > 1 {
                    let ch1 = input.char_at(1);
                    match ch1 {
                        '=' => Some(Self::LookBehindPositive),
                        '!' => Some(Self::LookBehindNegative),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub const fn is_look_behind(&self) -> bool {
        matches!(self, Self::LookBehindPositive | Self::LookBehindNegative)
    }

    pub const fn is_look_ahead(&self) -> bool {
        matches!(self, Self::LookAheadPositive | Self::LookAheadNegative)
    }

    /// Returns the number of characters consumed by the definition.
    pub const fn definition_size(&self) -> usize {
        match self {
            Self::LookBehindPositive | Self::LookBehindNegative => 2,
            Self::LookAheadPositive | Self::LookAheadNegative | Self::EndOfText => 1,
        }
    }
}

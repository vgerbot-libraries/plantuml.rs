/// Character class type (normal or negative).
///
/// Ported from: `com/plantuml/ubrex/CharClassType.java`

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClassType {
    Normal,
    Negative,
}

impl std::fmt::Display for CharClassType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharClassType::Normal => write!(f, "NORMAL"),
            CharClassType::Negative => write!(f, "NEGATIVE"),
        }
    }
}

/// Case sensitivity mode for character matching.
///
/// Ported from: `com/plantuml/ubrex/CaseMode.java`

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    CaseSensitive,
    CaseInsensitive,
}

impl CaseMode {
    /// Converts an uppercase ASCII letter to lowercase; other characters are unchanged.
    pub fn ensure_lowercase(ch: char) -> char {
        if ch.is_ascii_uppercase() {
            char::from_u32(ch as u32 - 'A' as u32 + 'a' as u32).unwrap_or(ch)
        } else {
            ch
        }
    }
}

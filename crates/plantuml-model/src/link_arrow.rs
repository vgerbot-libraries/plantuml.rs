//! `LinkArrow` — arrow direction for links.
//!
//! Ported from: `net/sourceforge/plantuml/abel/LinkArrow.java`

/// Arrow direction for a link between entities.
///
/// Ported from: `net/sourceforge/plantuml/abel/LinkArrow.java`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LinkArrow {
    /// No arrow or multiple arrows.
    #[default]
    NoneOrSeveral,
    /// Normal direction (entity1 → entity2).
    DirectNormal,
    /// Backward direction (entity2 → entity1).
    Backward,
}

impl LinkArrow {
    /// Returns the reversed arrow direction.
    ///
    /// Ported from: `LinkArrow.reverse()`.
    #[must_use]
    pub const fn reverse(self) -> Self {
        match self {
            Self::DirectNormal => Self::Backward,
            Self::Backward => Self::DirectNormal,
            Self::NoneOrSeveral => Self::NoneOrSeveral,
        }
    }
}

//! `Position` — top/bottom/left/right position for notes and other elements.
//!
//! Ported from: `net/sourceforge/plantuml/utils/Position.java`

/// Position of an element relative to an entity.
///
/// Ported from: `net/sourceforge/plantuml/utils/Position.java`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Position {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl Position {
    /// Returns the opposite position.
    ///
    /// Ported from: `Position.getInv()`.
    #[must_use]
    pub const fn get_inv(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

//! `CucaNote` — note attached to an entity or link.
//!
//! Ported from: `net/sourceforge/plantuml/abel/CucaNote.java`

use plantuml_klimt::Display;

use crate::entity::Colors;
use crate::position::Position;

/// Strategy for linking a note to an entity.
///
/// Ported from: `net/sourceforge/plantuml/abel/NoteLinkStrategy.java`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoteLinkStrategy {
    #[default]
    Normal,
    Hidden,
}

/// A note attached to an entity or link.
///
/// Ported from: `net/sourceforge/plantuml/abel/CucaNote.java`
#[derive(Debug, Clone, Default)]
pub struct CucaNote {
    display: Display,
    position: Position,
    colors: Colors,
    strategy: NoteLinkStrategy,
}

impl CucaNote {
    /// Builds a `CucaNote` with the given display, position, and colors.
    ///
    /// Ported from: `CucaNote.build(Display, Position, Colors)`.
    #[must_use]
    pub const fn build(display: Display, position: Position, colors: Colors) -> Self {
        Self {
            display,
            position,
            colors,
            strategy: NoteLinkStrategy::Normal,
        }
    }

    /// Returns a new `CucaNote` with the given strategy.
    ///
    /// Ported from: `CucaNote.withStrategy(NoteLinkStrategy)`.
    #[must_use]
    pub fn with_strategy(self, strategy: NoteLinkStrategy) -> Self {
        Self { strategy, ..self }
    }

    pub const fn get_display(&self) -> &Display {
        &self.display
    }

    pub const fn get_strategy(&self) -> NoteLinkStrategy {
        self.strategy
    }

    pub const fn get_colors(&self) -> &Colors {
        &self.colors
    }

    pub const fn get_position(&self) -> Position {
        self.position
    }
}

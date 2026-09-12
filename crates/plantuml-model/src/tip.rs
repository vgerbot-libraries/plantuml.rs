//! `Tip` — a note attached to a specific class member.
//!
//! Ported from: `net/sourceforge/plantuml/abel/Tip.java`

use plantuml_klimt::Display;

use crate::entity::{Colors, Stereotype};

/// A tip (note) attached to a single class member.
///
/// Ported from: `net/sourceforge/plantuml/abel/Tip.java`
#[derive(Debug, Clone, Default)]
pub struct Tip {
    display: Display,
    colors: Colors,
    stereotype: Option<Stereotype>,
}

impl Tip {
    /// Creates a new `Tip`.
    ///
    /// Ported from: `Tip(Display, Colors, Stereotype)`.
    #[must_use]
    pub const fn new(display: Display, colors: Colors, stereotype: Option<Stereotype>) -> Self {
        Self {
            display,
            colors,
            stereotype,
        }
    }

    pub const fn get_display(&self) -> &Display {
        &self.display
    }

    pub const fn get_colors(&self) -> &Colors {
        &self.colors
    }

    pub const fn get_stereotype(&self) -> Option<&Stereotype> {
        self.stereotype.as_ref()
    }
}

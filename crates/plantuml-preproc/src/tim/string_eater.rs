//! String eater — a simple eater that wraps a raw string.
//!
//! Ported from `net.sourceforge.plantuml.tim.StringEater`.

use super::eater::Eater;
use crate::stubs::LineLocation;
use crate::StringLocated;

/// An eater that wraps a raw string (no location info).
///
/// Ported from `net.sourceforge.plantuml.tim.StringEater`.
pub struct StringEater {
    eater: Eater,
}

impl StringEater {
    /// Creates a new `StringEater` from a string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            eater: Eater::new(StringLocated::new(s, LineLocation::default())),
        }
    }

    /// Returns the underlying eater.
    #[must_use]
    pub fn get_eater(&self) -> &Eater {
        &self.eater
    }

    /// Returns the underlying eater (mutable).
    pub fn get_eater_mut(&mut self) -> &mut Eater {
        &mut self.eater
    }
}

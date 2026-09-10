//! Error type for eater parsing.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterException`.

use thiserror::Error;

use crate::StringLocated;

/// Exception thrown during eater (directive parsing) operations.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterException`.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct EaterException {
    message: String,
    location: StringLocated,
}

impl EaterException {
    /// Creates a new `EaterException` with the given message and location.
    #[must_use]
    pub fn new(message: impl Into<String>, location: &StringLocated) -> Self {
        Self {
            message: message.into(),
            location: location.clone(),
        }
    }

    /// Returns the error message.
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// Returns the location where the error occurred.
    #[must_use]
    pub fn get_location(&self) -> &StringLocated {
        &self.location
    }
}

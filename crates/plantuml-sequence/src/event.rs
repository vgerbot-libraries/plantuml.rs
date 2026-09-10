//! Event — trait for sequence diagram events.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/Event.java`

use crate::participant::Participant;

/// A sequence diagram event (message, note, divider, grouping, life event, etc.).
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/Event.java`
pub trait Event {
    /// Returns `true` if this event involves the given participant.
    fn deal_with(&self, participant: &Participant) -> bool;

    /// Returns the URL associated with this event, if any.
    fn url(&self) -> Option<&str>;

    /// Returns `true` if this event has a URL.
    fn has_url(&self) -> bool {
        self.url().is_some()
    }

    /// Returns `true` if this event is parallel.
    fn is_parallel(&self) -> bool {
        false
    }

    /// Sets the Y position of this event.
    fn set_y(&mut self, y: f64);
}

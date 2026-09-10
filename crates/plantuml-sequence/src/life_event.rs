//! Life event — activate/deactivate/create/destroy on a participant.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/LifeEvent.java`

use crate::life_event_type::LifeEventType;
use crate::participant::Participant;

/// A life event on a participant's lifeline.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/LifeEvent.java`
#[derive(Debug, Clone)]
pub struct LifeEvent {
    participant: Participant,
    event_type: LifeEventType,
}

impl LifeEvent {
    /// Creates a new life event.
    #[must_use]
    pub fn new(participant: Participant, event_type: LifeEventType) -> Self {
        Self {
            participant,
            event_type,
        }
    }

    /// Returns the participant this life event applies to.
    #[must_use]
    pub fn participant(&self) -> &Participant {
        &self.participant
    }

    /// Returns the life event type.
    #[must_use]
    pub const fn event_type(&self) -> LifeEventType {
        self.event_type
    }

    #[must_use]
    pub const fn is_activate(&self) -> bool {
        self.event_type.is_activate()
    }

    #[must_use]
    pub const fn is_deactivate(&self) -> bool {
        self.event_type.is_deactivate()
    }

    #[must_use]
    pub const fn is_destroy(&self) -> bool {
        self.event_type.is_destroy()
    }

    #[must_use]
    pub const fn is_create(&self) -> bool {
        self.event_type.is_create()
    }
}

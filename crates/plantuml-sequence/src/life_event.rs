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
    /// Index of the message this `LifeEvent` is attached to (inline `++`/`--`).
    /// `None` for standalone `activate`/`deactivate`/`destroy` commands.
    message_index: Option<usize>,
}

impl LifeEvent {
    /// Creates a new life event.
    #[must_use]
    pub const fn new(participant: Participant, event_type: LifeEventType) -> Self {
        Self {
            participant,
            event_type,
            message_index: None,
        }
    }
    /// Creates a new life event attached to a message (inline `++`/`--`).
    #[must_use]
    pub const fn new_inline(participant: Participant, event_type: LifeEventType, message_index: usize) -> Self {
        Self {
            participant,
            event_type,
            message_index: Some(message_index),
        }
    }
    /// Returns the message index this life event is attached to, if inline.
    #[must_use]
    pub const fn message_index(&self) -> Option<usize> {
        self.message_index
    }

    /// Returns the participant this life event applies to.
    #[must_use]
    pub const fn participant(&self) -> &Participant {
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

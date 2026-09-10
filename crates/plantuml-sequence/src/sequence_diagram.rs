//! Sequence diagram — main model holding participants and events.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/SequenceDiagram.java`

use crate::event::Event;
use crate::life_event::LifeEvent;
use crate::life_event_type::LifeEventType;
use crate::message::Message;
use crate::participant::Participant;
use crate::participant_type::ParticipantType;

/// A sequence diagram model.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/SequenceDiagram.java`
#[derive(Debug)]
pub struct SequenceDiagram {
    participants: Vec<Participant>,
    events: Vec<SequenceEvent>,
    message_counter: i32,
}

/// A sequence diagram event — either a message or a life event.
#[derive(Debug, Clone)]
pub enum SequenceEvent {
    Message(Message),
    LifeEvent(LifeEvent),
}

impl SequenceDiagram {
    /// Creates a new empty sequence diagram.
    #[must_use]
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            events: Vec::new(),
            message_counter: 0,
        }
    }

    /// Returns the participants.
    #[must_use]
    pub fn participants(&self) -> &[Participant] {
        &self.participants
    }

    /// Returns the events.
    #[must_use]
    pub fn events(&self) -> &[SequenceEvent] {
        &self.events
    }

    /// Gets or creates a participant by code. If the participant already exists,
    /// it is returned without reordering the participants list.
    pub fn get_or_create_participant(&mut self, code: &str) -> Participant {
        // Check if participant already exists
        if let Some(pos) = self.participants.iter().position(|p| p.code() == code) {
            return self.participants[pos].clone();
        }
        let p = Participant::new(
            ParticipantType::Participant,
            code,
            code,
            self.participants.len() as i32,
        );
        self.participants.push(p.clone());
        p
    }

    /// Declares a participant with a display name. If the participant already
    /// exists, updates its display name (without reordering).
    pub fn declare_participant(&mut self, code: &str, display: &str) {
        if let Some(pos) = self.participants.iter().position(|p| p.code() == code) {
            self.participants[pos].set_display(display);
        } else {
            let p = Participant::new(
                ParticipantType::Participant,
                code,
                display,
                self.participants.len() as i32,
            );
            self.participants.push(p);
        }
    }

    /// Adds a message to the diagram.
    pub fn add_message(&mut self, msg: Message) {
        self.events.push(SequenceEvent::Message(msg));
    }

    /// Activates or deactivates a participant.
    pub fn activate(&mut self, participant: &Participant, event_type: LifeEventType) {
        let life_event = LifeEvent::new(participant.clone(), event_type);
        self.events.push(SequenceEvent::LifeEvent(life_event));
    }

    /// Returns the next message number.
    #[must_use]
    pub fn get_next_message_number(&mut self) -> String {
        self.message_counter += 1;
        self.message_counter.to_string()
    }
}

impl Default for SequenceDiagram {
    fn default() -> Self {
        Self::new()
    }
}

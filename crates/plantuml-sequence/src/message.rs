//! Message — a message between two participants.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/Message.java`

use crate::event::Event;
use crate::participant::Participant;
use plantuml_skin::ArrowConfiguration;

/// A message between two participants in a sequence diagram.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/Message.java`
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Message {
    p1: Participant,
    p2: Participant,
    label: String,
    arrow_config: ArrowConfiguration,
    message_number: String,
    url: Option<String>,
    parallel: bool,
    y: f64,
}

impl Message {
    /// Creates a new message from `p1` to `p2` with the given label and arrow configuration.
    #[must_use]
    pub fn new(
        p1: Participant,
        p2: Participant,
        label: impl Into<String>,
        arrow_config: ArrowConfiguration,
        message_number: impl Into<String>,
    ) -> Self {
        Self {
            p1,
            p2,
            label: label.into(),
            arrow_config,
            message_number: message_number.into(),
            url: None,
            parallel: false,
            y: 0.0,
        }
    }

    /// Returns the source participant.
    #[must_use]
    pub const fn p1(&self) -> &Participant {
        &self.p1
    }

    /// Returns the destination participant.
    #[must_use]
    pub const fn p2(&self) -> &Participant {
        &self.p2
    }

    /// Returns the message label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the arrow configuration.
    #[must_use]
    pub const fn arrow_config(&self) -> &ArrowConfiguration {
        &self.arrow_config
    }

    /// Returns the message number.
    #[must_use]
    pub fn message_number(&self) -> &str {
        &self.message_number
    }

    /// Sets the URL for this message.
    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = Some(url.into());
    }

    /// Marks this message as parallel.
    pub const fn go_parallel(&mut self) {
        self.parallel = true;
    }

    /// Returns `true` if this is a self-message (p1 == p2).
    #[must_use]
    pub fn is_self_message(&self) -> bool {
        self.p1 == self.p2
    }
}

impl Event for Message {
    fn deal_with(&self, participant: &Participant) -> bool {
        self.p1 == *participant || self.p2 == *participant
    }

    fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    fn is_parallel(&self) -> bool {
        self.parallel
    }

    fn set_y(&mut self, y: f64) {
        self.y = y;
    }
}

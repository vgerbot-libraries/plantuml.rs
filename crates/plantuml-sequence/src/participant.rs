//! Participant — a sequence diagram participant.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/Participant.java`

use crate::participant_type::ParticipantType;

/// A participant in a sequence diagram.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/Participant.java`
#[derive(Debug, Clone)]
pub struct Participant {
    code: String,
    display: String,
    ptype: ParticipantType,
    initial_life: i32,
    order: i32,
    uid: String,
}

impl Participant {
    /// Creates a new participant.
    #[must_use]
    pub fn new(ptype: ParticipantType, code: impl Into<String>, display: impl Into<String>, order: i32) -> Self {
        let code = code.into();
        let uid = format!("part_{}", code);
        Self {
            code,
            display: display.into(),
            ptype,
            initial_life: 0,
            order,
            uid,
        }
    }

    /// Returns the participant code (unique identifier).
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the display name.
    #[must_use]
    pub fn display(&self) -> &str {
        &self.display
    }

    /// Sets the display name.
    pub fn set_display(&mut self, display: impl Into<String>) {
        self.display = display.into();
    }

    /// Sets the participant type.
    pub fn set_ptype(&mut self, ptype: ParticipantType) {
        self.ptype = ptype;
    }

    /// Returns the participant type.
    #[must_use]
    pub const fn ptype(&self) -> ParticipantType {
        self.ptype
    }

    /// Returns the initial life level.
    #[must_use]
    pub const fn initial_life(&self) -> i32 {
        self.initial_life
    }

    /// Returns the order index.
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }

    /// Returns the unique identifier.
    #[must_use]
    pub fn uid(&self) -> &str {
        &self.uid
    }
}

impl PartialEq for Participant {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Eq for Participant {}

impl std::hash::Hash for Participant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}

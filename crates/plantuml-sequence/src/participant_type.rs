//! Participant type — enum of participant kinds.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/ParticipantType.java`

/// The type of a sequence diagram participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticipantType {
    Participant,
    Actor,
    Boundary,
    Control,
    Entity,
    Queue,
    Database,
    Collections,
}

impl ParticipantType {
    /// Returns the participant type from a string keyword.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "participant" => Some(Self::Participant),
            "actor" => Some(Self::Actor),
            "boundary" => Some(Self::Boundary),
            "control" => Some(Self::Control),
            "entity" => Some(Self::Entity),
            "queue" => Some(Self::Queue),
            "database" => Some(Self::Database),
            "collections" => Some(Self::Collections),
            _ => None,
        }
    }
}

impl std::fmt::Display for ParticipantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Participant => write!(f, "PARTICIPANT"),
            Self::Actor => write!(f, "ACTOR"),
            Self::Boundary => write!(f, "BOUNDARY"),
            Self::Control => write!(f, "CONTROL"),
            Self::Entity => write!(f, "ENTITY"),
            Self::Queue => write!(f, "QUEUE"),
            Self::Database => write!(f, "DATABASE"),
            Self::Collections => write!(f, "COLLECTIONS"),
        }
    }
}

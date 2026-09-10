//! Life event type — activate, deactivate, create, destroy.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/LifeEventType.java`

/// The type of a life event on a participant's lifeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifeEventType {
    Create,
    Activate,
    Deactivate,
    Destroy,
    Suspend,
    Resume,
    DoubleActivate,
}

impl LifeEventType {
    #[must_use]
    pub const fn is_activate(&self) -> bool {
        matches!(self, Self::Activate | Self::DoubleActivate)
    }

    #[must_use]
    pub const fn is_deactivate(&self) -> bool {
        matches!(self, Self::Deactivate)
    }

    #[must_use]
    pub const fn is_destroy(&self) -> bool {
        matches!(self, Self::Destroy)
    }

    #[must_use]
    pub const fn is_create(&self) -> bool {
        matches!(self, Self::Create)
    }
}

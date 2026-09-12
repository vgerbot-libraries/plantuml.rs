//! `Together` — grouping constraint for entities that should be laid out together.
//!
//! Ported from: `net/sourceforge/plantuml/abel/Together.java`

/// Groups entities that should be positioned together in the layout.
///
/// Ported from: `net/sourceforge/plantuml/abel/Together.java`
#[derive(Debug, Clone, Default)]
pub struct Together {
    parent: Option<Box<Self>>,
}

impl Together {
    /// Creates a new `Together` with an optional parent.
    ///
    /// Ported from: `Together(Together)`.
    #[must_use]
    pub fn new(parent: Option<Self>) -> Self {
        Self {
            parent: parent.map(Box::new),
        }
    }

    /// Returns the parent `Together`, if any.
    ///
    /// Ported from: `Together.getParent()`.
    #[must_use]
    pub fn get_parent(&self) -> Option<&Self> {
        self.parent.as_deref()
    }
}

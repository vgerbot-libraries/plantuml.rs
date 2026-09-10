//! Group type enum — group/package type variants.
//!
//! Ported from: `net/sourceforge/plantuml/abel/GroupType.java`

/// Group type variants for group entities (packages, states, etc.).
///
/// Ported from: `net/sourceforge/plantuml/abel/GroupType.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupType {
    /// Root group — the top-level container.
    Root,
    /// Package group — for grouping related entities.
    Package,
    /// State group — composite state in state diagrams.
    State,
    /// Concurrent state group — for parallel states.
    ConcurrentState,
    /// Inner activity group — nested activity within a state.
    InnerActivity,
    /// Concurrent activity group — for parallel activities.
    ConcurrentActivity,
    /// Domain group — for domain-driven design diagrams.
    Domain,
    /// Requirement group — for requirement diagrams.
    Requirement,
}

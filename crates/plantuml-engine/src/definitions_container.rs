//! Definitions container trait.
//!
//! Ported from `net.sourceforge.plantuml.DefinitionsContainer`.

/// Trait for objects that can provide `@startdef` definitions.
///
/// Ported from `net.sourceforge.plantuml.DefinitionsContainer`.
pub trait DefinitionsContainer {
    /// Returns the definition content for the given name, or empty if not found.
    fn get_definition(&self, name: &str) -> Vec<String>;
}

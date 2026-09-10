//! Trait for checking if a define name is truthy.
//!
//! Ported from `net.sourceforge.plantuml.preproc.Truth`.

/// Trait for checking if a variable name is defined/truthy.
///
/// Ported from `net.sourceforge.plantuml.preproc.Truth`.
pub trait Truth {
    /// Returns `true` if the given name is defined.
    fn is_true(&self, name: &str) -> bool;
}

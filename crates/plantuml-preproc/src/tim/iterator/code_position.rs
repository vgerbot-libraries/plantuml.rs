//! Code position trait for tracking position in code iteration.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator.CodePosition`.

/// Trait for code position tracking in while/foreach loops.
///
/// Ported from `net.sourceforge.plantuml.tim.iterator.CodePosition`.
pub trait CodePosition: Send + Sync + std::fmt::Debug {
    /// Returns the code position as a cloneable trait object.
    fn clone_box(&self) -> Box<dyn CodePosition>;

    /// Returns an `Any` reference for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Clone for Box<dyn CodePosition> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

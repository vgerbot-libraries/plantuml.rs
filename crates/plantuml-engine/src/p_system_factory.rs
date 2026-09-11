//! `PSystemFactory` trait — interface for diagram-type factories.
//!
//! Ported from: net/sourceforge/plantuml/api/PSystemFactory.java

use plantuml_core::{Diagram, DiagramType, PSystemError};

use crate::uml_source::UmlSource;

/// A factory that creates a [`Diagram`] from a [`UmlSource`].
///
/// Each diagram type (sequence, class, state, etc.) registers a
/// `PSystemFactory` implementation. The `PSystemBuilder` iterates factories
/// in registration order and returns the first one that succeeds.
///
/// Ported from: net/sourceforge/plantuml/api/PSystemFactory.java
pub trait PSystemFactory {
    /// Returns the diagram type this factory handles.
    ///
    /// Ported from: `PSystemFactory.getDiagramType()`.
    fn get_diagram_type(&self) -> DiagramType;

    /// Attempts to create a diagram from the given source.
    ///
    /// Returns `Ok` on success, or `Err(PSystemError)` on failure.
    ///
    /// Ported from: `PSystemFactory.createSystem(...)`.
    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError>;
}

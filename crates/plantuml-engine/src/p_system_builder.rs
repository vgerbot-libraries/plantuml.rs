//! `PSystemBuilder` — dispatches to the first factory that can create a diagram.
//!
//! Ported from: net/sourceforge/plantuml/PSystemBuilder.java

use plantuml_core::{Diagram, PSystemError};

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Builds a [`Diagram`] from a [`UmlSource`] by trying registered factories
/// in order.
///
/// Each factory is only tried if its `get_diagram_type()` matches one of the
/// candidate types in the source's `@start<keyword>` directive. The first
/// factory that succeeds wins.
///
/// Ported from: net/sourceforge/plantuml/PSystemBuilder.java
pub struct PSystemBuilder {
    factories: Vec<Box<dyn PSystemFactory>>,
}

impl PSystemBuilder {
    /// Creates a new `PSystemBuilder` with the given factories.
    ///
    /// Factories should be registered in Java's order (sequence first, then
    /// class, activity, description, state, etc.) for correct dispatch
    /// priority.
    #[must_use]
    pub fn new(factories: Vec<Box<dyn PSystemFactory>>) -> Self {
        Self { factories }
    }

    /// Creates the default `PSystemBuilder` with all currently-implemented
    /// factories registered in Java's order.
    ///
    /// Ported from: `PSystemBuilder` private constructor (static initializer).
    #[must_use]
    pub fn default_builder() -> Self {
        let factories: Vec<Box<dyn PSystemFactory>> = vec![
            // Position 1: Sequence (currently the only implemented factory).
            Box::new(crate::sequence_factory::SequenceDiagramFactory),
        ];
        Self::new(factories)
    }

    /// Attempts to create a diagram from the given source.
    ///
    /// Iterates factories whose `get_diagram_type()` is in
    /// `source.get_diagram_types()`, returning the first success.
    /// If no factory matches, returns a `PSystemError`.
    ///
    /// Ported from: `PSystemBuilder.createPSystem(...)`.
    pub fn create_p_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let diagram_types = source.get_diagram_types();

        let mut last_error: Option<PSystemError> = None;

        for factory in &self.factories {
            if !diagram_types.contains(&factory.get_diagram_type()) {
                continue;
            }

            match factory.create_system(source) {
                Ok(diagram) => return Ok(diagram),
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            PSystemError::syntax(
                "No matching factory for this diagram type",
                plantuml_core::DiagramType::Unknown,
            )
        }))
    }
}

//! State diagram factory.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/statediagram/StateDiagram.java`
//! - `net/sourceforge/plantuml/statediagram/StateDiagramFactory.java`
//!
//! State diagrams use `@startuml` (not `@startstate`), sharing the UML
//! diagram type set. The factory checks for state-related keywords.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_cuca::CucaDiagram;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Factory for State diagrams.
///
/// Ported from: `StateDiagramFactory.java`.
pub struct StateDiagramFactory;

/// Keywords that indicate a state diagram source.
const STATE_KEYWORDS: &[&str] = &["state ", "[*]", "hide empty description"];

impl PSystemFactory for StateDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::State
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .get_source()
            .iter()
            .map(|l| l.get_string().to_string())
            .collect();

        let has_state_content = lines
            .iter()
            .any(|line| STATE_KEYWORDS.iter().any(|kw| line.contains(kw)));

        if !has_state_content {
            return Err(PSystemError::syntax(
                "No state declarations found",
                DiagramType::State,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = CucaDiagram::from_lines(&line_refs, DiagramType::State);
        Ok(Box::new(diagram))
    }
}

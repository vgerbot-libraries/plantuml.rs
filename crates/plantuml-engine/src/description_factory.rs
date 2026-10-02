//! Description diagram factory (Component, Deployment, UseCase).
//!
//! Ported from:
//! - `net/sourceforge/plantuml/descdiagram/DescriptionDiagram.java`
//! - `net/sourceforge/plantuml/descdiagram/DescriptionDiagramFactory.java`
//!
//! Component, Deployment, and UseCase diagrams all map to
//! `DiagramType::Description` and share the same factory. They use
//! `@startuml` and are detected by content keywords.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_cuca::CucaDiagram;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Factory for Description diagrams (Component/Deployment/UseCase).
///
/// Ported from: `DescriptionDiagramFactory.java`.
pub struct DescriptionDiagramFactory;

/// Keywords that indicate a description diagram source.
const DESCRIPTION_KEYWORDS: &[&str] = &[
    "component ",
    "node ",
    "usecase ",
    "use case ",
    "actor ",
    "database ",
    "cloud ",
    "rectangle ",
    "frame ",
    "folder ",
    "artifact ",
    "queue ",
    "stack ",
    "storage ",
    "card ",
    "file ",
    "interface ",
    "port ",
    "hexagon ",
    "collections ",
    "boundary ",
    "control ",
    "entity ",
];
/// A trimmed line that begins with an anonymous component in brackets,
/// optionally followed by a link, e.g. `[Client] ..> API` or `[Server]`.
pub(crate) fn starts_with_bracket_component(line: &str) -> bool {
    let t = line.trim_start();
    let Some(close) = t.find(']') else {
        return false;
    };
    let inner = &t[1..close];
    if inner.is_empty() || inner.contains('[') || inner.contains(']') {
        return false;
    }
    let after = t[close + 1..].trim_start();
    after.is_empty() || after.starts_with(['-', '.'])
}

impl PSystemFactory for DescriptionDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Description
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .body_iter()
            .map(|l| l.get_string().to_string())
            .collect();

        let has_desc_content = lines.iter().any(|line| {
            let lower = line.to_lowercase();
            DESCRIPTION_KEYWORDS.iter().any(|kw| lower.contains(kw))
                || starts_with_bracket_component(line)
        });

        if !has_desc_content {
            return Err(PSystemError::syntax(
                "No description diagram elements found",
                DiagramType::Description,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = CucaDiagram::from_lines(&line_refs, DiagramType::Description);
        Ok(Box::new(diagram))
    }
}

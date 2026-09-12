//! Activity diagram factory.
//!
//! Ported from: `net/sourceforge/plantuml/activitydiagram/ActivityDiagramFactory.java`.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_activity::ActivityDiagram;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Factory for Activity diagrams.
pub struct ActivityDiagramFactory;

/// Keywords that indicate an activity diagram source.
const ACTIVITY_KEYWORDS: &[&str] = &[
    "start",
    "stop",
    ":",
    "if ",
    "else",
    "endif",
    "while ",
    "endwhile",
    "fork",
    "end fork",
    "endfork",
    "repeat",
    "backward",
    "kill",
];

impl PSystemFactory for ActivityDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Activity
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .get_source()
            .iter()
            .map(|l| l.get_string().to_string())
            .collect();

        // Check if source contains activity-related keywords.
        let has_activity_content = lines.iter().any(|line| {
            let trimmed = line.trim().to_lowercase();
            ACTIVITY_KEYWORDS.iter().any(|kw| trimmed.starts_with(kw))
        });

        if !has_activity_content {
            return Err(PSystemError::syntax(
                "No activity diagram keywords found",
                DiagramType::Activity,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = ActivityDiagram::from_lines(&line_refs);
        Ok(Box::new(diagram))
    }
}

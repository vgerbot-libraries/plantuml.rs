//! Gantt diagram factory.
//!
//! Ported from: `net/sourceforge/plantuml/ganttdiagram/GanttDiagramFactory.java`.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_gantt::GanttDiagram;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Factory for Gantt diagrams.
pub struct GanttDiagramFactory;

/// Keywords that indicate a gantt diagram source.
const GANTT_KEYWORDS: &[&str] = &[
    "task ",
    "milestone ",
    "depends on",
    "project starts",
    "lasts ",
    "happens at",
    "[",
];

impl PSystemFactory for GanttDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Gantt
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .get_source()
            .iter()
            .map(|l| l.get_string().to_string())
            .collect();

        // Check if source contains gantt-related keywords.
        let has_gantt_content = lines.iter().any(|line| {
            let lower = line.to_lowercase();
            GANTT_KEYWORDS.iter().any(|kw| lower.contains(kw))
        });

        if !has_gantt_content {
            return Err(PSystemError::syntax(
                "No gantt tasks found",
                DiagramType::Gantt,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = GanttDiagram::from_lines(&line_refs);
        Ok(Box::new(diagram))
    }
}

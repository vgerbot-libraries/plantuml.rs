//! Timing diagram factory.
//!
//! Ported from: `net/sourceforge/plantuml/timingdiagram/TimingDiagramFactory.java`.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_timing::TimingDiagram;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Factory for Timing diagrams.
pub struct TimingDiagramFactory;

/// Keywords that indicate a timing diagram source.
const TIMING_KEYWORDS: &[&str] = &[
    "binary ",
    "clock ",
    "analog ",
    "hexa ",
    "hexadecimal ",
    "digital ",
    "robust ",
    "concise ",
];

impl PSystemFactory for TimingDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Timing
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines: Vec<String> = source
            .get_source()
            .iter()
            .map(|l| l.get_string().to_string())
            .collect();

        // Check if source contains timing-related keywords.
        let has_timing_content = lines.iter().any(|line| {
            let lower = line.to_lowercase();
            TIMING_KEYWORDS.iter().any(|kw| lower.contains(kw))
        });

        if !has_timing_content {
            return Err(PSystemError::syntax(
                "No timing signals found",
                DiagramType::Timing,
            ));
        }

        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let diagram = TimingDiagram::from_lines(&line_refs);
        Ok(Box::new(diagram))
    }
}

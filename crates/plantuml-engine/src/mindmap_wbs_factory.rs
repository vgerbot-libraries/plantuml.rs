//! Mindmap and WBS diagram factories.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/mindmap/MindMapDiagramFactory.java`
//! - `net/sourceforge/plantuml/wbs/WBSDiagramFactory.java`
//!
//! These factories live in `plantuml-engine` to avoid circular dependencies.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_mindmap::{MindMapDiagram, WbsDiagram};

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

/// Factory for mindmap diagrams.
///
/// Ported from: `net/sourceforge/plantuml/mindmap/MindMapDiagramFactory.java`.
pub struct MindMapDiagramFactory;

impl PSystemFactory for MindMapDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Mindmap
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines = extract_body_lines(source, "@endmindmap");
        Ok(Box::new(MindMapDiagram::from_lines(&lines)))
    }
}

/// Factory for WBS diagrams.
///
/// Ported from: `net/sourceforge/plantuml/wbs/WBSDiagramFactory.java`.
pub struct WbsDiagramFactory;

impl PSystemFactory for WbsDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Wbs
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let lines = extract_body_lines(source, "@endwbs");
        Ok(Box::new(WbsDiagram::from_lines(&lines)))
    }
}

/// Extracts body lines from the source, skipping `@start`, `@end`, and `<style>` blocks.
fn extract_body_lines<'a>(source: &'a UmlSource, end_keyword: &str) -> Vec<&'a str> {
    let mut lines = Vec::new();
    let mut in_style = false;

    for line in source.body_iter() {
        let text = line.get_string();
        let trimmed = text.trim();

        if trimmed.to_lowercase().starts_with(end_keyword) {
            break;
        }

        // Skip <style> blocks.
        if trimmed.starts_with("<style>") {
            in_style = true;
            continue;
        }
        if trimmed.ends_with("</style>") {
            in_style = false;
            continue;
        }
        if in_style {
            continue;
        }

        // Skip `---` frontmatter lines.
        if trimmed == "---" {
            continue;
        }
        // Skip frontmatter key-value lines (e.g. `output: svg`).
        if trimmed.contains(':') && !trimmed.starts_with('*') && !trimmed.starts_with('+') && !trimmed.starts_with('-') && !trimmed.starts_with('#') {
            // Could be frontmatter or a mindmap directive — check if it looks like YAML.
            if !trimmed.starts_with("::") {
                continue;
            }
        }

        lines.push(text);
    }

    lines
}

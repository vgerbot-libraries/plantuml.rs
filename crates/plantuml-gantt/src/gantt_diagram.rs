//! Gantt diagram — implements the `Diagram` trait.
//!
//! Ported from: `net/sourceforge/plantuml/ganttdiagram/GanttDiagram.java`.

use std::io::Write;

use plantuml_core::{
    diagram::Diagram,
    diagram_type::DiagramType,
    file_format::FileFormat,
    file_format_option::FileFormatOption,
    PlantumlError,
};

use crate::gantt_parser::{parse_gantt_source, GanttSource};
use crate::gantt_renderer::render_gantt_svg;

/// A Gantt diagram.
pub struct GanttDiagram {
    source: GanttSource,
}

impl GanttDiagram {
    /// Creates a new `GanttDiagram` from source lines.
    #[must_use]
    pub fn from_lines(lines: &[&str]) -> Self {
        Self {
            source: parse_gantt_source(lines),
        }
    }
}

impl Diagram for GanttDiagram {
    fn export_diagram(
        &self,
        os: &mut dyn Write,
        _num: usize,
        file_format: &FileFormatOption,
    ) -> Result<(), PlantumlError> {
        if file_format.file_format() != FileFormat::Svg {
            return Err(PlantumlError::UnsupportedFormat(format!(
                "{:?}",
                file_format.file_format()
            )));
        }

        let svg = render_gantt_svg(&self.source, "(Gantt)");
        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some("(Gantt)".to_string())
    }

    fn get_warning_or_error(&self) -> Option<String> {
        if self.source.tasks.is_empty() {
            Some("No tasks found in gantt diagram".to_string())
        } else {
            None
        }
    }

    fn has_url(&self) -> bool {
        false
    }

    fn root_cause(&self) -> Option<String> {
        None
    }

    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Gantt
    }
}

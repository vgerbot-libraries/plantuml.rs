//! Activity diagram — implements the `Diagram` trait.
//!
//! Ported from: `net/sourceforge/plantuml/activitydiagram/ActivityDiagram.java`.

use std::io::Write;

use plantuml_core::{
    diagram::Diagram,
    diagram_type::DiagramType,
    file_format::FileFormat,
    file_format_option::FileFormatOption,
    PlantumlError,
};

use crate::activity_parser::{parse_activity_source, ActivitySource};
use crate::activity_renderer::render_activity_svg;

/// An activity diagram.
pub struct ActivityDiagram {
    source: ActivitySource,
}

impl ActivityDiagram {
    /// Creates a new `ActivityDiagram` from source lines.
    #[must_use]
    pub fn from_lines(lines: &[&str]) -> Self {
        Self {
            source: parse_activity_source(lines),
        }
    }
}

impl Diagram for ActivityDiagram {
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

        let svg = render_activity_svg(&self.source, "(Activity)");
        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some("(Activity)".to_string())
    }

    fn get_warning_or_error(&self) -> Option<String> {
        if self.source.nodes.is_empty() {
            Some("No activity nodes found".to_string())
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
        DiagramType::Activity
    }
}

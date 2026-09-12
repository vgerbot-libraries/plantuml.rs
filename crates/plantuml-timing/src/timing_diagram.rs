//! Timing diagram — implements the `Diagram` trait.
//!
//! Ported from: `net/sourceforge/plantuml/timingdiagram/TimingDiagram.java`.

use std::io::Write;

use plantuml_core::{
    diagram::Diagram,
    diagram_type::DiagramType,
    file_format::FileFormat,
    file_format_option::FileFormatOption,
    PlantumlError,
};

use crate::timing_parser::{parse_timing_source, TimingSource};
use crate::timing_renderer::render_timing_svg;

/// A timing diagram.
///
/// Ported from: `TimingDiagram.java`.
pub struct TimingDiagram {
    /// Parsed timing source.
    source: TimingSource,
}

impl TimingDiagram {
    /// Creates a new `TimingDiagram` from source lines.
    #[must_use]
    pub fn from_lines(lines: &[&str]) -> Self {
        Self {
            source: parse_timing_source(lines),
        }
    }
}

impl Diagram for TimingDiagram {
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

        let svg = render_timing_svg(&self.source, "(Timing)");
        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some("(Timing)".to_string())
    }

    fn get_warning_or_error(&self) -> Option<String> {
        if self.source.signals.is_empty() {
            Some("No signals found in timing diagram".to_string())
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
        DiagramType::Timing
    }
}

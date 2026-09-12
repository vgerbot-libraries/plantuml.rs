//! `WbsDiagram` — the WBS diagram model.
//!
//! Ported from: `net/sourceforge/plantuml/wbs/WBSDiagram.java` (258 lines).

use std::io::Write;

use plantuml_core::{
    diagram::Diagram,
    diagram_type::DiagramType,
    file_format::FileFormat,
    file_format_option::FileFormatOption,
    PlantumlError,
};
use plantuml_svg::{SvgGraphics, SvgOption};

use crate::wbs_element::{parse_wbs_tree, WElement};
use crate::wbs_renderer::render_wbs_svg;

/// A WBS (Work Breakdown Structure) diagram.
///
/// Ported from: `net/sourceforge/plantuml/wbs/WBSDiagram.java`.
pub struct WbsDiagram {
    /// The parsed WBS tree (or `None` if parsing failed).
    root: Option<WElement>,
    /// Warning/error message, if any.
    warning: Option<String>,
}

impl WbsDiagram {
    /// Creates a new `WbsDiagram` from a parsed tree.
    #[must_use]
    pub fn new(root: Option<WElement>) -> Self {
        let warning = if root.is_none() {
            Some("No root element found in WBS source".to_string())
        } else {
            None
        };
        Self { root, warning }
    }

    /// Creates a diagram from source body lines.
    #[must_use]
    pub fn from_lines(lines: &[&str]) -> Self {
        let root = parse_wbs_tree(lines);
        Self::new(root)
    }
}

impl Diagram for WbsDiagram {
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

        let svg = if let Some(root) = &self.root { render_wbs_svg(root) } else {
            let msg = self.warning.as_deref().unwrap_or("Invalid WBS data");
            render_error_svg(msg)
        };

        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some("Work Breakdown Structure".to_string())
    }

    fn get_warning_or_error(&self) -> Option<String> {
        self.warning.clone()
    }

    fn has_url(&self) -> bool {
        false
    }

    fn root_cause(&self) -> Option<String> {
        None
    }

    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Wbs
    }
}

fn render_error_svg(message: &str) -> String {
    let mut option = SvgOption::basic();
    option.set_title("(WBS)");

    let mut svg = SvgGraphics::new(0, option);

    let text_w = 14.0 * 0.6 * message.chars().count() as f64;
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), "#0000FF".to_string());
    svg.text(
        message,
        10.0,
        20.0,
        Some("monospace"),
        14,
        Some("normal"),
        Some("normal"),
        None,
        text_w,
        &attrs,
        None,
    );

    svg.create_xml()
}

//! `MindMapDiagram` — the mindmap diagram model.
//!
//! Ported from: `net/sourceforge/plantuml/mindmap/MindMapDiagram.java` (159 lines).

use std::io::Write;

use plantuml_core::{
    diagram::Diagram,
    diagram_type::DiagramType,
    file_format::FileFormat,
    file_format_option::FileFormatOption,
    PlantumlError,
};

use crate::idea::{parse_mindmap_orgmode, parse_mindmap_plus, Idea};
use crate::mindmap_renderer::render_mindmap_svg;

/// A mindmap diagram.
///
/// Ported from: `net/sourceforge/plantuml/mindmap/MindMapDiagram.java`.
pub struct MindMapDiagram {
    /// The parsed idea tree (or `None` if parsing failed).
    root: Option<Idea>,
    /// Warning/error message, if any.
    warning: Option<String>,
}

impl MindMapDiagram {
    /// Creates a new `MindMapDiagram` from a parsed idea tree.
    #[must_use]
    pub fn new(root: Option<Idea>) -> Self {
        let warning = if root.is_none() {
            Some("No root idea found in mindmap source".to_string())
        } else {
            None
        };
        Self { root, warning }
    }

    /// Creates a diagram from source body lines.
    ///
    /// Parses both org-mode (`*` prefix) and plus/minus (`+`/`-` prefix) syntax.
    #[must_use]
    pub fn from_lines(lines: &[&str]) -> Self {
        // Try org-mode syntax first, then plus/minus.
        let root = parse_mindmap_orgmode(lines).or_else(|| parse_mindmap_plus(lines));
        Self::new(root)
    }
}

impl Diagram for MindMapDiagram {
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

        let svg = match &self.root {
            Some(root) => render_mindmap_svg(root),
            None => render_error_svg(
                self.warning
                    .as_deref()
                    .unwrap_or("Invalid mindmap data"),
            ),
        };

        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some("(Mindmap)".to_string())
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
        DiagramType::Mindmap
    }
}

/// Renders an error message as SVG.
fn render_error_svg(message: &str) -> String {
    let mut option = SvgOption::basic();
    option.set_title("(Mindmap)");

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

use plantuml_svg::{SvgGraphics, SvgOption};

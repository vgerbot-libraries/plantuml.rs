//! `JsonDiagram` — the JSON diagram model.
//!
//! Ported from: `net/sourceforge/plantuml/jsondiagram/JsonDiagram.java`.
//!
//! Holds the parsed JSON value and renders it as SVG via `json_renderer`.

use std::io::Write;

use plantuml_core::{
    diagram_type::DiagramType,
    file_format_option::FileFormatOption,
    file_format::FileFormat,
    diagram::Diagram,
    PlantumlError,
};
use serde_json::Value;

use crate::json_renderer::{render_json_svg, Highlight};

/// A JSON (or YAML) diagram.
///
/// Ported from: `net/sourceforge/plantuml/jsondiagram/JsonDiagram.java`.
///
/// The Java class extends `TitledDiagram` and implements `TextBlock`.
/// In Rust, we hold the JSON data directly and implement `Diagram`.
pub struct JsonDiagram {
    /// The parsed JSON root value (or `None` if parsing failed).
    root: Option<Value>,
    /// Highlight specifications.
    highlights: Vec<Highlight>,
    /// The diagram type (JSON or YAML).
    diagram_type: DiagramType,
    /// Warning/error message, if any.
    warning: Option<String>,
}

impl JsonDiagram {
    /// Creates a new `JsonDiagram`.
    ///
    /// Ported from: `JsonDiagram(UmlSource, DiagramType, JsonValue, List<Highlighted>, ...)`.
    #[must_use]
    pub fn new(
        root: Option<Value>,
        highlights: Vec<Highlight>,
        diagram_type: DiagramType,
    ) -> Self {
        let warning = if root.is_none() {
            Some(format!(
                "Your data does not sound like {} data",
                match diagram_type {
                    DiagramType::Yaml => "YAML",
                    DiagramType::Hcl => "HCL",
                    _ => "JSON",
                }
            ))
        } else {
            None
        };

        Self {
            root,
            highlights,
            diagram_type,
            warning,
        }
    }

    /// Returns the diagram type label for SVG title.
    fn type_label(&self) -> &'static str {
        match self.diagram_type {
            DiagramType::Yaml => "yaml",
            DiagramType::Hcl => "hcl",
            _ => "json",
        }
    }
}

impl Diagram for JsonDiagram {
    fn export_diagram(
        &self,
        os: &mut dyn Write,
        _num: usize,
        file_format: &FileFormatOption,
    ) -> Result<(), PlantumlError> {
        if file_format.file_format() != FileFormat::Svg {
            return Err(PlantumlError::UnsupportedFormat(
                format!("{:?}", file_format.file_format()),
            ));
        }

        let svg = if let Some(value) = &self.root { render_json_svg(value, &self.highlights, self.type_label()) } else {
            // Render error message.
            let msg = self
                .warning
                .as_deref()
                .unwrap_or("Invalid JSON data");
            render_error_svg(msg, self.type_label())
        };

        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        let desc = match self.diagram_type {
            DiagramType::Yaml => "(Yaml)",
            DiagramType::Hcl => "(HCL)",
            _ => "(Json)",
        };
        Some(desc.to_string())
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
        self.diagram_type
    }
}

/// Renders an error message as SVG (when JSON parsing fails).
fn render_error_svg(message: &str, diagram_type: &str) -> String {
    use plantuml_svg::{SvgGraphics, SvgOption};

    let mut option = SvgOption::basic();
    option.set_title(format!("({})", diagram_type.to_pascal_case()));

    let mut svg = SvgGraphics::new(0, option);

    // Approximate text width (monospace 14px: ~8.4px per char).
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

trait DiagramTypeExt {
    fn to_pascal_case(&self) -> String;
}

impl DiagramTypeExt for str {
    fn to_pascal_case(&self) -> String {
        let mut result = String::new();
        let mut capitalize = true;
        for ch in self.chars() {
            if ch.is_ascii_alphabetic() {
                if capitalize {
                    result.push(ch.to_ascii_uppercase());
                    capitalize = false;
                } else {
                    result.push(ch);
                }
            } else {
                capitalize = true;
            }
        }
        result
    }
}

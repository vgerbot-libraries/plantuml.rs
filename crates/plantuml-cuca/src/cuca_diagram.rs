//! `CucaDiagram` — the base diagram for class, state, and description types.
//!
//! Ported from: `net/atmp/CucaDiagram.java` (959 lines).
//!
//! Manages entities, links, and groups. Provides the `Diagram` trait
//! implementation that renders via the simplified layout engine.

use std::collections::HashMap;
use std::io::Write;

use plantuml_core::{
    diagram::Diagram,
    diagram_type::DiagramType,
    file_format::FileFormat,
    file_format_option::FileFormatOption,
    PlantumlError,
};

use crate::cuca_layout::compute_layout;
use crate::cuca_renderer::render_cuca_svg;
use crate::entity_link_parser::{parse_entity_link_source, ParsedEntity};

/// A CucaDiagram — base for class, state, and description diagrams.
///
/// Ported from: `net/atmp/CucaDiagram.java`.
///
/// The Java class is abstract and extended by ClassDiagram, StateDiagram,
/// DescriptionDiagram, etc. In Rust, we use a concrete struct with a
/// `DiagramType` field to distinguish diagram kinds.
pub struct CucaDiagram {
    /// Parsed entities keyed by name.
    entities: HashMap<String, ParsedEntity>,
    /// Parsed links.
    links: Vec<crate::entity_link_parser::ParsedLink>,
    /// Parsed notes.
    notes: Vec<crate::entity_link_parser::ParsedNote>,
    /// Parsed packages.
    _packages: Vec<crate::entity_link_parser::ParsedPackage>,
    /// The diagram type (Class, State, Description, etc.).
    diagram_type: DiagramType,
    /// Warning/error message, if any.
    warning: Option<String>,
}

impl CucaDiagram {
    /// Creates a new `CucaDiagram` from parsed source lines.
    ///
    /// Ported from: `CucaDiagram` constructor + factory `createEmptyDiagram`.
    #[must_use]
    pub fn from_lines(lines: &[&str], diagram_type: DiagramType) -> Self {
        let parsed = parse_entity_link_source(lines);
        let warning = if parsed.entities.is_empty() {
            Some("No entities found in source".to_string())
        } else {
            None
        };
        Self {
            entities: parsed.entities.into_iter().collect(),
            links: parsed.links,
            notes: parsed.notes,
            _packages: parsed.packages,
            diagram_type,
            warning,
        }
    }

    /// Returns the diagram label for SVG title.
    fn diagram_label(&self) -> &str {
        match self.diagram_type {
            DiagramType::Class => "(Class)",
            DiagramType::Object => "(Object)",
            DiagramType::State => "(State)",
            DiagramType::Description => "(Component)",
            _ => "(Diagram)",
        }
    }
}

impl Diagram for CucaDiagram {
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

        let svg = if self.entities.is_empty() {
            render_error_svg(
                self.warning
                    .as_deref()
                    .unwrap_or("Empty diagram"),
                self.diagram_label(),
            )
        } else {
            let layout = compute_layout(&self.entities, &self.links);
            render_cuca_svg(&layout, &self.entities, &self.notes, self.diagram_type)
        };

        os.write_all(svg.as_bytes())
            .map_err(PlantumlError::from)?;
        Ok(())
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some(self.diagram_label().to_string())
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

/// Renders an error message as SVG.
fn render_error_svg(message: &str, label: &str) -> String {
    let mut option = SvgOption::basic();
    option.set_title(label.to_string());

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

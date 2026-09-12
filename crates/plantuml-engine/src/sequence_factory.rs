//! Sequence diagram factory — wraps the existing bypass pipeline as a
//! `PSystemFactory` + `Diagram` implementation.
//!
//! This is a transitional adapter: the bypass pipeline
//! (`parse_simple_sequence` → `render_sequence_svg`) is encapsulated behind
//! the `PSystemFactory` / `Diagram` traits so that `PSystemBuilder` can
//! dispatch to it uniformly. Eventually the sequence diagram will be ported
//! to the Command framework like the other diagram types.

use std::io::Write;

use plantuml_core::{
    Diagram, DiagramType, FileFormat, FileFormatOption, PlantumlError, PSystemError,
};
use plantuml_preproc::StringLocated;

use crate::p_system_factory::PSystemFactory;
use crate::sequence_renderer::ParsedSequence;
use crate::uml_source::UmlSource;

/// Factory that creates sequence diagrams via the bypass pipeline.
///
/// Registered as the first factory in `PSystemBuilder` (matching Java's
/// `SequenceDiagramFactory` at position 3, after welcome/colors eggs which
/// we don't implement).
pub struct SequenceDiagramFactory;

impl PSystemFactory for SequenceDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Sequence
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        // Reconstruct the full source text from the UmlSource lines.
        let source_text: String = source
            .get_source()
            .iter()
            .map(StringLocated::get_string)
            .collect::<Vec<_>>()
            .join("\n");

        // Reject sources that contain state diagram keywords — the state
        // factory should handle those instead.
        let has_state_keyword = source_text.lines().any(|line| {
            let t = line.trim();
            t.starts_with("state ") || t.contains("[*]") || t.contains("hide empty description")
        });
        if has_state_keyword {
            return Err(PSystemError::syntax(
                "Source contains state diagram keywords",
                DiagramType::Sequence,
            ));
        }

        // Reject sources that contain description diagram keywords
        // (usecase, actor, component, node, etc.) — the description
        // factory should handle those instead. In Java, the command-based
        // SequenceDiagramFactory simply fails to parse these lines, but our
        // permissive `parse_simple_sequence` accepts `A -> B` arrows even
        // when the source is a use case / component / deployment diagram.
        let has_desc_keyword = source_text.lines().any(|line| {
            let lower = line.to_lowercase();
            let t = lower.trim();
            t.starts_with("usecase ")
                || t.starts_with("use case ")
                || t.starts_with("actor ")
                || t.starts_with("component ")
                || t.starts_with("node ")
                || t.starts_with("database ")
                || t.starts_with("cloud ")
                || t.starts_with("rectangle ")
                || t.starts_with("frame ")
                || t.starts_with("folder ")
                || t.starts_with("artifact ")
                || t.starts_with("queue ")
                || t.starts_with("stack ")
                || t.starts_with("storage ")
                || t.starts_with("card ")
                || t.starts_with("file ")
                || t.starts_with("interface ")
                || t.starts_with("port ")
                || t.starts_with("hexagon ")
                || t.starts_with("collections ")
                || t.starts_with("boundary ")
                || t.starts_with("control ")
                || t.starts_with("entity ")
        });
        if has_desc_keyword {
            return Err(PSystemError::syntax(
                "Source contains description diagram keywords",
                DiagramType::Sequence,
            ));
        }

        // Reject sources that contain class/object diagram keywords.
        let has_class_keyword = source_text.lines().any(|line| {
            let t = line.trim();
            t.starts_with("class ")
                || t.starts_with("interface ")
                || t.starts_with("object ")
                || t.starts_with("enum ")
                || t.starts_with("abstract ")
                || t.starts_with("package ")
                || t.starts_with("namespace ")
        });
        if has_class_keyword {
            return Err(PSystemError::syntax(
                "Source contains class diagram keywords",
                DiagramType::Sequence,
            ));
        }

        match crate::sequence_renderer::parse_simple_sequence(&source_text) {
            Some(parsed) => Ok(Box::new(SequenceDiagramImpl { parsed })),
            None => Err(PSystemError::syntax(
                "Cannot parse as sequence diagram",
                DiagramType::Sequence,
            )),
        }
    }
}

/// A sequence diagram produced by the bypass pipeline.
///
/// Implements `Diagram` so it can be returned from `PSystemFactory::create_system`.
struct SequenceDiagramImpl {
    parsed: ParsedSequence,
}

impl Diagram for SequenceDiagramImpl {
    fn export_diagram(
        &self,
        os: &mut dyn Write,
        _num: usize,
        file_format: &FileFormatOption,
    ) -> Result<(), PlantumlError> {
        if file_format.file_format() == FileFormat::Svg {
            let svg = self.parsed.render();
            os.write_all(svg.as_bytes())
                .map_err(PlantumlError::from)?;
            Ok(())
        } else {
            Err(PlantumlError::UnsupportedFormat(format!(
                "{:?}",
                file_format.file_format()
            )))
        }
    }

    fn get_nb_images(&self) -> usize {
        1
    }

    fn get_description(&self) -> Option<String> {
        Some("Sequence".to_string())
    }

    fn get_warning_or_error(&self) -> Option<String> {
        None
    }

    fn has_url(&self) -> bool {
        false
    }

    fn root_cause(&self) -> Option<String> {
        None
    }

    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Sequence
    }
}

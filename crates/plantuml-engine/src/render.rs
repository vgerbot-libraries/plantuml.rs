//! Unified render API for FFI/WASM bindings.
//!
//! Provides a single entry point that dispatches to the appropriate renderer
//! based on the requested [`FileFormat`]. Currently supports SVG (sequence
//! diagrams) and PREPROC (preprocessed text).

use std::io::Cursor;

use plantuml_core::{FileFormat, FileFormatOption};

use crate::block_uml_builder::BlockUmlBuilder;
use crate::p_system_builder::PSystemBuilder;
use crate::uml_source::UmlSource;

/// Error returned by the unified render API.
#[derive(Debug)]
pub enum RenderError {
    /// The source could not be parsed as a supported diagram type.
    ParseFailed,
    /// The requested format is not yet implemented.
    UnsupportedFormat(FileFormat),
    /// The rendered output was not valid UTF-8.
    Utf8(std::string::FromUtf8Error),
    /// An error from the diagram export pipeline.
    Export(String),
}

/// Renders `PlantUML` `source` to the requested `format`.
///
/// Currently supports [`FileFormat::Svg`] and [`FileFormat::Preproc`]; all other
/// formats return [`RenderError::UnsupportedFormat`].
pub fn render(source: &str, format: FileFormat) -> Result<String, RenderError> {
    match format {
        FileFormat::Svg => render_svg(source),
        FileFormat::Preproc => render_preproc(source),
        f => Err(RenderError::UnsupportedFormat(f)),
    }
}

/// Renders `PlantUML` `source` to an SVG string.
///
/// Tries the `PSystemBuilder` pipeline first (preprocess → factory dispatch →
/// `Diagram::export_diagram`). Falls back to the legacy bypass pipeline
/// (`parse_simple_sequence`) for backward compatibility during the transition.
pub fn render_svg(source: &str) -> Result<String, RenderError> {
    // Try the PSystemBuilder pipeline first.
    if let Some(svg) = render_svg_via_pipeline(source) {
        return Ok(svg);
    }

    // Fallback: legacy bypass pipeline (direct parse without preprocessing).
    let parsed = crate::sequence_renderer::parse_simple_sequence(source)
        .ok_or(RenderError::ParseFailed)?;
    Ok(parsed.render())
}

/// Attempts to render SVG via the `PSystemBuilder` pipeline.
///
/// Returns `None` if no factory can handle the source (caller falls back to
/// the bypass pipeline).
fn render_svg_via_pipeline(source: &str) -> Option<String> {
    // 1. Preprocess source via BlockUmlBuilder → BlockUml → get_data().
    let builder = BlockUmlBuilder::new(source, &plantuml_preproc::preproc::Defines::new());
    let blocks = builder.get_block_umls();
    if blocks.is_empty() {
        return None;
    }

    let data = blocks[0].get_data();
    if data.is_empty() {
        return None;
    }

    // 2. Create UmlSource from preprocessed lines.
    let uml_source = UmlSource::new(data.to_vec());

    // 3. Create PSystemBuilder and dispatch.
    let psystem_builder = PSystemBuilder::default_builder();
    let diagram = psystem_builder.create_p_system(&uml_source).ok()?;

    // 4. Export as SVG.
    let option = FileFormatOption::new(FileFormat::Svg);
    let mut buf = Cursor::new(Vec::new());
    diagram
        .export_diagram(&mut buf, 0, &option)
        .ok()?;
    String::from_utf8(buf.into_inner()).ok()
}

/// Renders `PlantUML` `source` to PREPROC (preprocessed) text.
pub fn render_preproc(source: &str) -> Result<String, RenderError> {
    let reader = crate::source_string_reader::SourceStringReader::new(source);
    let mut buf = Vec::new();
    reader.output_image(&mut buf, 0, &FileFormatOption::new(FileFormat::Preproc));
    String::from_utf8(buf).map_err(RenderError::Utf8)
}

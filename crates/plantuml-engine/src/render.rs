//! Unified render API for FFI/WASM bindings.
//!
//! Provides a single entry point that dispatches to the appropriate renderer
//! based on the requested [`FileFormat`]. Currently supports SVG (sequence
//! diagrams) and PREPROC (preprocessed text).

use plantuml_core::FileFormat;

/// Error returned by the unified render API.
#[derive(Debug)]
pub enum RenderError {
    /// The source could not be parsed as a supported diagram type.
    ParseFailed,
    /// The requested format is not yet implemented.
    UnsupportedFormat(FileFormat),
    /// The rendered output was not valid UTF-8.
    Utf8(std::string::FromUtf8Error),
}

/// Renders PlantUML `source` to the requested `format`.
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

/// Renders PlantUML `source` to an SVG string.
///
/// Returns [`RenderError::ParseFailed`] when the source cannot be parsed as a
/// sequence diagram.
pub fn render_svg(source: &str) -> Result<String, RenderError> {
    let parsed = crate::sequence_renderer::parse_simple_sequence(source)
        .ok_or(RenderError::ParseFailed)?;
    Ok(parsed.render())
}

/// Renders PlantUML `source` to PREPROC (preprocessed) text.
pub fn render_preproc(source: &str) -> Result<String, RenderError> {
    use plantuml_core::FileFormatOption;
    let reader = crate::source_string_reader::SourceStringReader::new(source);
    let mut buf = Vec::new();
    reader.output_image(&mut buf, 0, &FileFormatOption::new(FileFormat::Preproc));
    String::from_utf8(buf).map_err(RenderError::Utf8)
}

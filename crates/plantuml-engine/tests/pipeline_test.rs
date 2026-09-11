//! Tests for the PSystemBuilder pipeline (Phase 0 verification).
//!
//! Verifies that the new PSystemBuilder dispatch produces identical SVG
//! output to the legacy bypass pipeline for sequence diagrams.

use plantuml_core::FileFormat;
use plantuml_engine::render_svg;

/// A simple sequence diagram that the bypass pipeline can parse.
const SIMPLE_SEQUENCE: &str = "@startuml\nAlice -> Bob: hello\n@enduml";

/// Renders via the new `render_svg` (which tries PSystemBuilder pipeline first,
/// falls back to bypass) and verifies it produces valid SVG.
#[test]
fn test_pipeline_renders_sequence_svg() {
    let svg = render_svg(SIMPLE_SEQUENCE).expect("render_svg should succeed");

    // Must be valid SVG.
    assert!(svg.contains("<svg"), "output must contain <svg: {svg}");
    assert!(svg.contains("</svg>"), "output must contain </svg: {svg}");
}

/// Verifies that the PSystemBuilder pipeline and the bypass pipeline produce
/// identical SVG for a simple sequence diagram.
#[test]
fn test_pipeline_matches_bypass() {
    // Render via the unified API (uses PSystemBuilder pipeline first).
    let pipeline_svg = render_svg(SIMPLE_SEQUENCE).expect("pipeline render should succeed");

    // Render via the bypass directly.
    let parsed = plantuml_engine::sequence_renderer::parse_simple_sequence(SIMPLE_SEQUENCE)
        .expect("bypass parse should succeed");
    let bypass_svg = parsed.render();

    assert_eq!(
        pipeline_svg, bypass_svg,
        "PSystemBuilder pipeline SVG must match bypass pipeline SVG"
    );
}

/// Verifies that the `render` function dispatches SVG correctly.
#[test]
fn test_render_dispatches_svg() {
    let result = plantuml_engine::render(SIMPLE_SEQUENCE, FileFormat::Svg);
    assert!(result.is_ok(), "render(Svg) should succeed");
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "must produce SVG");
}

/// Verifies that a non-sequence diagram type falls back to bypass and
/// returns ParseFailed (since no factory handles it yet).
#[test]
fn test_unsupported_type_returns_parse_failed() {
    let source = "@startjson\n{\"key\": \"value\"}\n@endjson";
    let result = render_svg(source);
    // No factory handles JSON yet, and bypass can't parse it either.
    assert!(result.is_err(), "JSON should fail until implemented");
}

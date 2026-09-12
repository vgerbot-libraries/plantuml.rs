//! Tests for the SvgCleaner test utility.
//!
//! Ported from: `test/vega/SvgCleaner.java` test behaviour.

mod support;

use support::svg_cleaner::SvgCleaner;

#[test]
fn test_clean_removes_processing_instructions() {
    let svg = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\">\n<?xml-stylesheet href=\"style.xsl\" type=\"text/xsl\"?>\n<rect x=\"0\" y=\"0\" width=\"100\" height=\"100\"/>\n</svg>";

    let cleaned = SvgCleaner::clean(svg);
    assert!(
        !cleaned.contains("xml-stylesheet"),
        "PI should be removed: {cleaned}"
    );
    assert!(
        !cleaned.contains("<?xml"),
        "XML declaration should be removed: {cleaned}"
    );
    assert!(cleaned.contains("<svg"), "svg element should remain");
    assert!(cleaned.contains("<rect"), "rect element should remain");
}

#[test]
fn test_normalise_pretty_prints() {
    let svg = r#"<svg><rect x="0" y="0"/></svg>"#;
    let normalised = SvgCleaner::normalise(svg);
    assert!(
        normalised.contains("\n  <rect"),
        "should be pretty-printed with 2-space indent: {normalised}"
    );
}

#[test]
fn test_clean_and_normalise_produce_same_structure() {
    let svg = r#"<svg><g><rect/></g></svg>"#;
    let cleaned = SvgCleaner::clean(svg);
    let normalised = SvgCleaner::normalise(svg);
    // Without PIs, clean and normalise should produce the same output.
    assert_eq!(cleaned, normalised);
}

#[test]
fn test_preserves_text_content() {
    let svg = r#"<svg><text>Hello World</text></svg>"#;
    let cleaned = SvgCleaner::clean(svg);
    assert!(
        cleaned.contains("Hello World"),
        "text content should be preserved: {cleaned}"
    );
}

#[test]
fn test_strips_whitespace_between_elements() {
    let svg = "<svg>\n  <rect/>\n</svg>";
    let cleaned = SvgCleaner::clean(svg);
    // The output should be properly pretty-printed regardless of input whitespace.
    assert!(cleaned.contains("<svg>"));
    assert!(cleaned.contains("<rect/>"));
    assert!(cleaned.contains("</svg>"));
    // Should have proper 2-space indentation.
    assert!(cleaned.contains("\n  <rect/>"), "rect should be indented: {cleaned}");
}

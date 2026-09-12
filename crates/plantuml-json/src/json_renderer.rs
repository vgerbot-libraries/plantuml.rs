//! JSON tree SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/jsondiagram/SmetanaForJson.java`,
//! `TextBlockJson.java`, `JsonCurve.java`.
//!
//! The Java version uses the Smetana (Graphviz) layout engine to position
//! nodes. This Rust implementation uses a simpler recursive tree layout that
//! produces valid SVG with the same visual structure: boxes for each JSON
//! node, curved connectors between parents and children.

use serde_json::Value;

use plantuml_svg::{SvgGraphics, SvgOption};

// ── Layout constants (from Java TextBlockJson/SmetanaForJson defaults) ────

/// Horizontal padding inside each box (each side).
const PADDING_H: f64 = 6.0;
/// Vertical padding inside each box (top + bottom).
const PADDING_V: f64 = 4.0;
/// Spacing between sibling boxes.
const SIBLING_SPACING: f64 = 8.0;
/// Vertical gap between parent box and children row.
const LEVEL_SPACING: f64 = 30.0;
/// Stroke color for boxes (matches Java default `#A80036`).
const STROKE_COLOR: &str = "#A80036";
/// Fill color for object/array boxes.
const FILL_CONTAINER: &str = "#F2F2F2";
/// Fill color for leaf value boxes.
const FILL_LEAF: &str = "#FFFFFF";
/// Fill color for highlighted boxes.
const FILL_HIGHLIGHT: &str = "#FFFACD";
/// Text color for keys.
const COLOR_KEY: &str = "#0000FF";
/// Text color for values.
const COLOR_VALUE: &str = "#000000";
/// Stroke width for box borders.
const STROKE_WIDTH: f64 = 1.0;
/// Font size for text.
const FONT_SIZE: i32 = 12;
/// Font family.
const FONT_FAMILY: &str = "monospace";
/// Page margin.
const PAGE_MARGIN: f64 = 5.0;

/// A highlight specification (key path → color).
#[derive(Debug, Clone)]
pub struct Highlight {
    /// Dot-separated path, e.g. `foo.bar.0`.
    pub path: String,
}

impl Highlight {
    /// Parses a `#highlight` line.
    ///
    /// Ported from: `net/sourceforge/plantuml/yaml/Highlighted.java`.
    pub fn build(line: &str) -> Option<Self> {
        let trimmed = line.trim_start_matches('#').trim();
        if let Some(rest) = trimmed.strip_prefix("highlight") {
            let path = rest.trim();
            if path.is_empty() {
                None
            } else {
                Some(Self {
                    path: path.to_string(),
                })
            }
        } else {
            None
        }
    }

    /// Checks if a line is a highlight definition.
    pub fn matches_definition(line: &str) -> bool {
        line.trim_start_matches('#').trim_start().to_lowercase().starts_with("highlight")
    }
}

/// Computed layout for a JSON node.
struct NodeLayout {
    /// Box x position (top-left).
    x: f64,
    /// Box y position (top-left).
    y: f64,
    /// Box width.
    width: f64,
    /// Box height.
    height: f64,
    /// Center x of this node (for curve connections).
    center_x: f64,
    /// Children layouts.
    children: Vec<Self>,
    /// Label text for this node.
    label: String,
    /// Whether this is a container (object/array) vs leaf.
    is_container: bool,
    /// Whether this node is highlighted.
    highlighted: bool,
}

/// Approximate text width for monospace font at `FONT_SIZE`.
fn text_width(text: &str) -> f64 {
    // Monospace: each char is ~0.6 * font_size wide.
    f64::from(FONT_SIZE) * 0.6 * text.chars().count() as f64
}

/// Approximate text height for `FONT_SIZE`.
const fn text_height() -> f64 {
    (FONT_SIZE as f64) * 1.2
}

/// Formats a JSON value for display in a box.
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => "{...}".to_string(),
        Value::Array(_) => "[...]".to_string(),
    }
}

/// Checks if a path matches a highlight path.
fn path_matches(highlight_path: &str, node_path: &str) -> bool {
    highlight_path == node_path
        || node_path.starts_with(&format!("{highlight_path}."))
        || node_path.starts_with(&format!("{highlight_path}["))
}

/// Recursively computes the layout of a JSON tree.
///
/// Returns the `NodeLayout` with absolute positions (relative to `origin_x`, `origin_y`).
fn compute_layout(
    value: &Value,
    key: Option<&str>,
    path: &str,
    highlights: &[Highlight],
    origin_x: f64,
    origin_y: f64,
) -> NodeLayout {
    let highlighted = highlights.iter().any(|h| path_matches(&h.path, path));

    match value {
        Value::Object(obj) => {
            let label = key
                .map(std::string::ToString::to_string)
                .unwrap_or_default();

            // Compute children layouts first to determine total width.
            let mut child_layouts = Vec::with_capacity(obj.len());
            let mut child_x = origin_x;
            let child_y = origin_y + text_height() + PADDING_V * 2.0 + LEVEL_SPACING;

            for (k, v) in obj {
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                let layout = compute_layout(v, Some(k), &child_path, highlights, child_x, child_y);
                child_x += layout.width + SIBLING_SPACING;
                child_layouts.push(layout);
            }

            // Remove trailing spacing.
            if !child_layouts.is_empty() {
                child_x -= SIBLING_SPACING;
            }

            // Container box width: max(label width, children total width).
            let label_w = text_width(&label) + PADDING_H * 2.0;
            let children_w = child_x - origin_x;
            let width = label_w.max(children_w).max(40.0);
            let height = text_height() + PADDING_V * 2.0;

            // Center children under the parent.
            let children_total_w = child_x - origin_x;
            let offset = (width - children_total_w) / 2.0;
            if offset > 0.0 && !child_layouts.is_empty() {
                for child in &mut child_layouts {
                    shift_layout(child, offset, 0.0);
                }
            }

            let center_x = origin_x + width / 2.0;

            NodeLayout {
                x: origin_x,
                y: origin_y,
                width,
                height,
                center_x,
                children: child_layouts,
                label,
                is_container: true,
                highlighted,
            }
        }
        Value::Array(arr) => {
            let label = key
                .map(std::string::ToString::to_string)
                .unwrap_or_default();

            let mut child_layouts = Vec::with_capacity(arr.len());
            let mut child_x = origin_x;
            let child_y = origin_y + text_height() + PADDING_V * 2.0 + LEVEL_SPACING;

            for (i, v) in arr.iter().enumerate() {
                let child_path = format!("{path}[{i}]");
                let layout = compute_layout(v, None, &child_path, highlights, child_x, child_y);
                child_x += layout.width + SIBLING_SPACING;
                child_layouts.push(layout);
            }

            if !child_layouts.is_empty() {
                child_x -= SIBLING_SPACING;
            }

            let label_w = text_width(&label) + PADDING_H * 2.0;
            let children_w = child_x - origin_x;
            let width = label_w.max(children_w).max(40.0);
            let height = text_height() + PADDING_V * 2.0;

            let children_total_w = child_x - origin_x;
            let offset = (width - children_total_w) / 2.0;
            if offset > 0.0 && !child_layouts.is_empty() {
                for child in &mut child_layouts {
                    shift_layout(child, offset, 0.0);
                }
            }

            let center_x = origin_x + width / 2.0;

            NodeLayout {
                x: origin_x,
                y: origin_y,
                width,
                height,
                center_x,
                children: child_layouts,
                label,
                is_container: true,
                highlighted,
            }
        }
        _ => {
            // Leaf node.
            let val_str = format_value(value);
            let label = if let Some(k) = key {
                format!("{k}: {val_str}")
            } else {
                val_str
            };
            let width = text_width(&label) + PADDING_H * 2.0;
            let height = text_height() + PADDING_V * 2.0;
            let center_x = origin_x + width / 2.0;

            NodeLayout {
                x: origin_x,
                y: origin_y,
                width,
                height,
                center_x,
                children: Vec::new(),
                label,
                is_container: false,
                highlighted,
            }
        }
    }
}

/// Recursively shifts a layout and all its children by (dx, dy).
fn shift_layout(layout: &mut NodeLayout, dx: f64, dy: f64) {
    layout.x += dx;
    layout.y += dy;
    layout.center_x += dx;
    for child in &mut layout.children {
        shift_layout(child, dx, dy);
    }
}

/// Computes the bounding box of a layout tree.
fn bounding_box(layout: &NodeLayout) -> (f64, f64) {
    let mut max_x = layout.x + layout.width;
    let mut max_y = layout.y + layout.height;
    for child in &layout.children {
        let (cw, ch) = bounding_box(child);
        max_x = max_x.max(cw);
        max_y = max_y.max(ch);
    }
    (max_x, max_y)
}

/// Renders a `NodeLayout` tree to `SvgGraphics`.
fn render_node(svg: &mut SvgGraphics, layout: &NodeLayout) {
    let fill = if layout.highlighted {
        FILL_HIGHLIGHT
    } else if layout.is_container {
        FILL_CONTAINER
    } else {
        FILL_LEAF
    };

    // Draw box.
    svg.set_fill_color(fill);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH, None);
    svg.svg_rectangle(
        layout.x,
        layout.y,
        layout.width,
        layout.height,
        0.0,
        0.0,
        0.0,
    );

    // Draw label text.
    let text_x = layout.x + PADDING_H;
    let text_y = layout.y + PADDING_V + f64::from(FONT_SIZE) * 0.85;
    let color = if layout.is_container && !layout.label.is_empty() {
        COLOR_KEY
    } else {
        COLOR_VALUE
    };
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), color.to_string());
    svg.text(
        &layout.label,
        text_x,
        text_y,
        Some(FONT_FAMILY),
        FONT_SIZE,
        Some("normal"),
        Some("normal"),
        None,
        text_width(&layout.label),
        &attrs,
        None,
    );

    for child in &layout.children {
        // Curve from bottom-center of parent to top-center of child.
        let x1 = layout.center_x;
        let y1 = layout.y + layout.height;
        let x2 = child.center_x;
        let y2 = child.y;
        // Simple cubic bezier curve.
        let cy1 = (y1 + y2) / 2.0;
        let cy2 = cy1;
        let d = format!(
            "M{x1:.1},{y1:.1} C{x1:.1},{cy1:.1} {x2:.1},{cy2:.1} {x2:.1},{y2:.1}"
        );
        svg.svg_path(&d, 0.0);

        // Recursively render child.
        render_node(svg, child);
    }
}

/// Renders a JSON value tree as an SVG string.
///
/// Ported from: `SmetanaForJson.drawMe()` + `JsonDiagram.drawU()`.
#[must_use]
pub fn render_json_svg(
    root: &Value,
    highlights: &[Highlight],
    diagram_type: &str,
) -> String {
    // Compute layout starting from page margin.
    let layout = compute_layout(root, None, "", highlights, PAGE_MARGIN, PAGE_MARGIN);

    // Compute total dimensions.
    let (max_x, max_y) = bounding_box(&layout);
    let total_w = max_x + PAGE_MARGIN;
    let total_h = max_y + PAGE_MARGIN;

    // Create SVG document.
    let mut option = SvgOption::basic();
    option.set_title(format!("({})", diagram_type.to_pascal_case()));
    option.set_desc(format!("Generated by plantuml.rs — {diagram_type} diagram"));

    let mut svg = SvgGraphics::new(0, option);

    // Set SVG dimensions.
    svg.set_root_attribute("width", &format_total(total_w));
    svg.set_root_attribute("height", &format_total(total_h));
    svg.set_root_attribute(
        "viewBox",
        &format!("0 0 {} {}", format_total(total_w), format_total(total_h)),
    );

    // Render the tree.
    render_node(&mut svg, &layout);

    svg.create_xml()
}

/// Converts a diagram type string to PascalCase for the title.
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

/// Formats a dimension value for SVG attributes.
fn format_total(val: f64) -> String {
    if (val - val.round()).abs() < 0.001 {
        format!("{}", val.round() as i64)
    } else {
        format!("{val:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_object() {
        let json: Value = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });
        let svg = render_json_svg(&json, &[], "json");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("name"));
        assert!(svg.contains("Alice"));
    }

    #[test]
    fn test_render_nested_object() {
        let json: Value = serde_json::json!({
            "person": {
                "name": "Bob",
                "address": {
                    "city": "NYC"
                }
            }
        });
        let svg = render_json_svg(&json, &[], "json");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("person"));
        assert!(svg.contains("address"));
    }

    #[test]
    fn test_render_array() {
        let json: Value = serde_json::json!({
            "items": ["apple", "banana", "cherry"]
        });
        let svg = render_json_svg(&json, &[], "json");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("items"));
        assert!(svg.contains("apple"));
    }

    #[test]
    fn test_render_empty_object() {
        let json: Value = serde_json::json!({});
        let svg = render_json_svg(&json, &[], "json");
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_render_null() {
        let json: Value = Value::Null;
        let svg = render_json_svg(&json, &[], "json");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("null"));
    }

    #[test]
    fn test_highlight_parsing() {
        assert!(Highlight::matches_definition("#highlight foo.bar"));
        assert!(Highlight::matches_definition("#highlight foo"));
        assert!(!Highlight::matches_definition("#some other directive"));
        let h = Highlight::build("#highlight foo.bar").unwrap();
        assert_eq!(h.path, "foo.bar");
    }

    #[test]
    fn test_render_with_highlight() {
        let json: Value = serde_json::json!({
            "name": "Alice",
            "age": 30
        });
        let highlights = vec![Highlight {
            path: "name".to_string(),
        }];
        let svg = render_json_svg(&json, &highlights, "json");
        assert!(svg.contains("<svg"));
        assert!(svg.contains(FILL_HIGHLIGHT));
    }
}

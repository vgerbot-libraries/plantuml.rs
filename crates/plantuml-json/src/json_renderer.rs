//! JSON/YAML SVG renderer.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/jsondiagram/SmetanaForJson.java`
//! - `net/sourceforge/plantuml/jsondiagram/TextBlockJson.java`
//! - `net/sourceforge/plantuml/jsondiagram/JsonCurve.java`
//!
//! The Java version builds an internal directed graph of `record` shaped nodes
//! and lays it out with Smetana (Graphviz), then rotates the TB result into a
//! left-to-right picture. Each container/array child value becomes an edge.
//!
//! Geometry is driven by the exact font 14 advance widths measured against the
//! reference JAR (bold for keys, plain for values). A record row pitch is
//! `23.068` (a `19.068` text block plus the record cell pad).

use serde_json::Value;

use plantuml_svg::{SvgGraphics, SvgOption};

// ── Style constants ──────────────────────────────────────────────────────

/// Record background fill (also the fill-rect stroke).
const NODE_FILL: &str = "#F1F1F1";
/// Record border / separator color.
const NODE_STROKE: &str = "#000000";
/// Text color.
const TEXT_COLOR: &str = "#000000";
/// Fill-rect and border stroke width.
const BORDER_WIDTH: f64 = 1.5;
/// Separator line stroke width.
const SEPARATOR_WIDTH: f64 = 1.0;
/// Rounded corner radius.
const CORNER_RADIUS: f64 = 5.0;
/// Font family for keys and values.
const FONT_FAMILY: &str = "sans-serif";
/// Font size.
const FONT_SIZE: i32 = 14;
/// Outer margin between the content and the canvas edges (each side).
const MARGIN: f64 = 10.0;
/// Canvas padding added past the furthest content edge.
const CANVAS_PAD: f64 = 11.0;

/// Effective vertical pitch of one record row: text-block row `19.068` plus
/// the Graphviz record cell pad (`4.0`).
pub(crate) const ROW_PITCH: f64 = 23.0679;
/// Baseline of the first row measured from the record top.
const FIRST_BASELINE: f64 = 16.9659;
/// Horizontal text inset inside a column (the text-block 5px margin).
const TEXT_INSET: f64 = 5.0;

// ── Font 14 advance widths (reference JAR, ASCII 0x20–0x7E) ─────────────

/// Bold 14 advance widths, indexed by byte `0x20..=0x7E`.
#[allow(clippy::too_many_lines)]
const BOLD14: [f64; 95] = [
    3.6399, 4.004, 6.608, 9.044, 8.008, 12.614, 10.5, 3.724, 4.746, 4.746, 7.63, 8.008, 3.99,
    4.508, 3.99, 5.782, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 3.99,
    3.99, 8.008, 8.008, 8.008, 6.678, 12.558, 9.66, 9.408, 8.918, 10.36, 7.84, 7.686, 10.136,
    10.71, 5.446, 4.634, 9.296, 7.91, 13.202, 11.382, 11.144, 8.792, 11.144, 9.24, 7.714, 8.106,
    10.584, 9.1, 13.5379, 9.338, 8.736, 8.106, 4.634, 5.782, 4.634, 8.008, 5.754, 5.068, 8.456,
    8.862, 7.196, 8.862, 8.274, 5.418, 8.862, 9.198, 4.27, 4.27, 8.68, 4.27, 13.7479, 9.198,
    8.666, 8.862, 8.862, 6.356, 6.958, 6.076, 9.198, 7.966, 11.9839, 8.092, 7.966, 6.832, 5.516,
    7.714, 5.516, 8.008,
];

/// Plain 14 advance widths, indexed by byte `0x20..=0x7E`.
#[allow(clippy::too_many_lines)]
const PLAIN14: [f64; 95] = [
    3.6399, 3.766, 5.712, 9.044, 8.008, 11.634, 10.248, 3.15, 4.2, 4.2, 7.714, 8.008, 3.752, 4.508,
    3.752, 5.208, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 8.008, 3.752, 3.752,
    8.008, 8.008, 8.008, 6.076, 12.586, 8.946, 9.1, 8.848, 10.22, 7.784, 7.266, 10.192, 10.374,
    4.746, 3.822, 8.666, 7.336, 12.698, 10.64, 10.934, 8.47, 10.934, 8.708, 7.686, 7.784, 10.234,
    8.4, 13.0199, 8.204, 7.924, 8.008, 4.606, 5.208, 4.606, 8.008, 6.216, 3.934, 7.854, 8.61, 6.72,
    8.61, 7.896, 4.816, 8.61, 8.652, 3.612, 3.612, 7.476, 3.612, 13.09, 8.652, 8.47, 8.61, 8.61,
    5.782, 6.706, 5.054, 8.652, 7.112, 11.004, 7.406, 7.14, 6.58, 5.32, 7.714, 5.32, 8.008,
];

/// Advance width of a non-ASCII glyph in the 14 SansSerif face.
///
/// Most symbols are absent from the ASCII tables; only the handful the
/// JSON/YAML short-string renderer emits need explicit advances. All other
/// characters fall back to the generic 8.0 cell width.
fn glyph14(ch: char) -> Option<f64> {
    match ch {
        // No-break space shares the ordinary space advance.
        '\u{00A0}' => Some(PLAIN14[0]),
        // Checked / unchecked ballot box.
        '\u{2611}' | '\u{2610}' => Some(12.5506),
        _ => None,
    }
}

/// Advance width of a string in the bold 14 face.
fn bold_width(text: &str) -> f64 {
    text.chars()
        .map(|ch| BOLD14.get((ch as usize).saturating_sub(0x20)).copied().unwrap_or(8.0))
        .sum()
}

/// Advance width of a string in the plain 14 face.
fn plain_width(text: &str) -> f64 {
    text.chars()
        .map(|ch| {
            PLAIN14
                .get((ch as usize).saturating_sub(0x20))
                .copied()
                .or_else(|| glyph14(ch))
                .unwrap_or(8.0)
        })
        .sum()
}

// ── Highlight ────────────────────────────────────────────────────────────

/// A highlight specification (key path).
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
        let path = trimmed.strip_prefix("highlight")?.trim();
        if path.is_empty() {
            None
        } else {
            Some(Self {
                path: path.to_string(),
            })
        }
    }

    /// Checks if a line is a highlight definition.
    pub fn matches_definition(line: &str) -> bool {
        line.trim_start_matches('#')
            .trim_start()
            .to_lowercase()
            .starts_with("highlight")
    }
}

// ── Record model ─────────────────────────────────────────────────────────

/// One row of a record: a bold key and (for objects) a plain value.
pub(crate) struct Row {
    /// Key text (bold).
    key: String,
    /// Key advance width.
    key_w: f64,
    /// Value text (plain), if present.
    value: Option<String>,
    /// Value advance width.
    value_w: f64,
}

/// A record node (one JSON object or array).
pub(crate) struct Record {
    /// Rows, top to bottom.
    pub(crate) rows: Vec<Row>,
    /// Whether this record renders an array (single column) vs a map.
    is_array: bool,
    /// Record width (`col_a + col_b` for maps).
    pub(crate) width: f64,
    /// Record height (`rows.len() * ROW_PITCH`).
    pub(crate) height: f64,
    /// Inner width of column A (max key width + margins).
    col_a: f64,

    /// Assigned top-left X.
    x: f64,
    /// Assigned top-left Y.
    y: f64,
    /// Child records for container values, `(row index, child index in edges)`.
    pub(crate) children: Vec<(usize, usize)>,
}

impl Record {
    /// Number of rows (record fields) in this node.
    pub(crate) fn row_count(&self) -> usize {
        self.rows.len()
    }
}
/// Value rendered in a container's own row in place of its children.
///
/// Java's record-label machinery emits the blank field as three
/// non-breaking spaces (U+00A0) so the whitespace survives serialization
/// (`SmetanaForJson.getDotLabelMap`/`getDotLabelArray`, rendered through
/// `SvgGraphics.text` as `&#160;&#160;&#160;`).
const CONTAINER_PLACEHOLDER: &str = "\u{00A0}\u{00A0}\u{00A0}";

/// Returns the display text for a scalar value.
///
/// Ported from: `TextBlockJson.getShortString()` (JSON). The YAML path uses
/// plain `true`/`false` rather than the ballot-box glyphs.
fn short_string(value: &Value, is_yaml: bool) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => '\u{2400}'.to_string(),
        Value::Bool(b) => {
            if is_yaml {
                if *b { "true".to_string() } else { "false".to_string() }
            } else if *b {
                "\u{2611} true".to_string()
            } else {
                "\u{2610} false".to_string()
            }
        }
        Value::Number(_) => value.to_string(),
        _ => CONTAINER_PLACEHOLDER.to_string(),
    }
}

/// Whether a JSON value is a container (object or array).
fn is_container(value: &Value) -> bool {
    value.is_object() || value.is_array()
}

/// Builds the record tree rooted at `value`.
///
/// Records are appended to `records` in depth-first order; edges reference
/// child records by index.
fn build_records(value: &Value, records: &mut Vec<Record>, is_yaml: bool) -> usize {
    let mut rows = Vec::new();
    let mut children = Vec::new();

    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let row_index = rows.len();
                if is_container(v) {
                    rows.push(Row {
                        key: k.clone(),
                        key_w: bold_width(k),
                        value: Some(CONTAINER_PLACEHOLDER.to_string()),
                        value_w: plain_width("   "),
                    });
                    let child_idx = build_records(v, records, is_yaml);
                    children.push((row_index, child_idx));
                } else {
                    let text = short_string(v, is_yaml);
                    let w = plain_width(&text);
                    rows.push(Row {
                        key: k.clone(),
                        key_w: bold_width(k),
                        value: Some(text),
                        value_w: w,
                    });
                }
            }
        }
        Value::Array(arr) => {
            for v in arr {
                let row_index = rows.len();
                if is_container(v) {
                    rows.push(Row {
                        key: String::new(),
                        key_w: 0.0,
                        value: Some(CONTAINER_PLACEHOLDER.to_string()),
                        value_w: plain_width("   "),
                    });
                    let child_idx = build_records(v, records, is_yaml);
                    children.push((row_index, child_idx));
                } else {
                    let text = short_string(v, is_yaml);
                    let w = plain_width(&text);
                    rows.push(Row {
                        key: String::new(),
                        key_w: 0.0,
                        value: Some(text),
                        value_w: w,
                    });
                }
            }
        }
        _ => {
            let text = short_string(value, is_yaml);
            let w = plain_width(&text);
            rows.push(Row {
                key: String::new(),
                key_w: 0.0,
                value: Some(text),
                value_w: w,
            });
        }
    }

    // Column inner widths: max advance width + 10 (5px margin each side).
    let col_a = rows.iter().map(|r| r.key_w).fold(0.0_f64, f64::max) + 10.0;
    let has_b = rows.iter().any(|r| r.value.is_some());
    let col_b = if has_b {
        rows.iter().map(|r| r.value_w).fold(0.0_f64, f64::max) + 10.0
    } else {
        0.0
    };

    // Array records use a single column: the values are the only column.
    let is_array = value.is_array();
    let (width, used_col_a) = if is_array {
        (col_b, 0.0)
    } else {
        (col_a + col_b, col_a)
    };

    let height = rows.len() as f64 * ROW_PITCH;

    let index = records.len();
    records.push(Record {
        rows,
        is_array,
        width,
        height,
        col_a: used_col_a,

        x: 0.0,
        y: 0.0,
        children,
    });
    index
}

// ── Rendering ────────────────────────────────────────────────────────────

/// Draws a single record (fill, rows, border) at its assigned position.
fn draw_record(svg: &mut SvgGraphics, record: &Record) {
    let x = record.x;
    let y = record.y;
    let w = record.width;
    let h = record.height;

    // Fill rect (fill == stroke).
    svg.set_fill_color(NODE_FILL);
    svg.set_stroke_color(Some(NODE_FILL));
    svg.set_stroke_width(BORDER_WIDTH, None);
    svg.svg_rectangle(x, y, w, h, CORNER_RADIUS, CORNER_RADIUS, 0.0);

    let col_a = if record.is_array { 0.0 } else { record.col_a };

    for (i, row) in record.rows.iter().enumerate() {
        let row_top = y + i as f64 * ROW_PITCH;
        let baseline = row_top + FIRST_BASELINE;
        let row_bottom = row_top + ROW_PITCH;

        // Key (bold), right-aligned text keeps left inset.
        if !record.is_array {
            let mut attrs = indexmap::IndexMap::new();
            attrs.insert("fill".to_string(), TEXT_COLOR.to_string());
            svg.text(
                &row.key,
                x + TEXT_INSET,
                baseline,
                Some(FONT_FAMILY),
                FONT_SIZE,
                Some("700"),
                None,
                None,
                row.key_w,
                &attrs,
                None,
            );
        }

        // Value (plain).
        if let Some(value) = &row.value {
            let value_x = if record.is_array {
                x + TEXT_INSET
            } else {
                x + col_a + TEXT_INSET
            };
            let mut attrs = indexmap::IndexMap::new();
            attrs.insert("fill".to_string(), TEXT_COLOR.to_string());
            svg.text(
                value,
                value_x,
                baseline,
                Some(FONT_FAMILY),
                FONT_SIZE,
                None,
                None,
                None,
                row.value_w,
                &attrs,
                None,
            );

            // Column separator for this row (maps only).
            if !record.is_array {
                svg.set_stroke_color(Some(NODE_STROKE));
                svg.set_stroke_width(SEPARATOR_WIDTH, None);
                svg.svg_line(x + col_a, row_top, x + col_a, row_bottom, 0.0);
            }
        }

        // Row separator below every row except the last.
        if i + 1 < record.rows.len() {
            svg.set_stroke_color(Some(NODE_STROKE));
            svg.set_stroke_width(SEPARATOR_WIDTH, None);
            svg.svg_line(x, row_bottom, x + w, row_bottom, 0.0);
        }
    }

    // Border rect, drawn last (fill none).
    svg.set_fill_color("none");
    svg.set_stroke_color(Some(NODE_STROKE));
    svg.set_stroke_width(BORDER_WIDTH, None);
    svg.svg_rectangle(x, y, w, h, CORNER_RADIUS, CORNER_RADIUS, 0.0);
}

/// Renders a JSON/YAML value tree as an SVG string.
///
/// Ported from: `SmetanaForJson.drawMe()` + `JsonDiagram.drawU()`.
#[must_use]
pub fn render_json_svg(root: &Value, _highlights: &[Highlight], diagram_type: &str) -> String {
    let is_yaml = diagram_type == "YAML";
    let mut records = Vec::new();
    let root_idx = build_records(root, &mut records, is_yaml);
    let layout = crate::layout::solve(&records, root_idx);

    // Assign card top-left positions from internal-frame node centers.
    // Internal box: vertical ht = ROUND(card width)+1, horizontal w = card
    // height. After the TB→LR rotation:
    //   final x = max - internal.y ; final y = internal.x.
    for (i, record) in records.iter_mut().enumerate() {
        let pos = &layout.nodes[i];
        let box_h = (record.width.round() as i64) as f64 + 1.0;
        record.x = layout.max - pos.y - box_h / 2.0 + MARGIN;
        record.y = pos.x - record.height / 2.0 + MARGIN;
    }

    // Canvas bounds from furthest content edges.
    let mut right = 0.0_f64;
    let mut bottom = 0.0_f64;
    for record in &records {
        right = right.max(record.x + record.width);
        bottom = bottom.max(record.y + record.height);
    }
    let total_w = (right.ceil() + CANVAS_PAD).max(MARGIN + CANVAS_PAD);
    let total_h = (bottom.ceil() + CANVAS_PAD).max(MARGIN + CANVAS_PAD);

    let mut option = SvgOption::basic();
    option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    option.set_root_attribute("data-diagram-type", diagram_type);

    let mut svg = SvgGraphics::new(0, option);

    // Cards are drawn root-first, then depth-first in child order (the
    // `manageOneNode` creation order), not by record Vec index: children
    // records are pushed before their parent.
    let mut order = Vec::new();
    fn push_preorder(idx: usize, records: &[Record], order: &mut Vec<usize>) {
        order.push(idx);
        for &(_, child) in &records[idx].children {
            push_preorder(child, records, order);
        }
    }
    push_preorder(root_idx, &records, &mut order);
    for &idx in &order {
        draw_record(&mut svg, &records[idx]);
    }
    for edge in &layout.edges {
        draw_edge(&mut svg, edge, layout.max);
    }

    // The canvas adds an external margin: `ceil(furthest content) + 11`.
    svg.ensure_visible(total_w, total_h);

    svg.create_xml()
}

// ── Edge drawing ─────────────────────────────────────────────────────────

/// Mirrors an internal-frame point into final SVG coordinates.
fn tf(p: crate::pathplan::Point, max: f64) -> (f64, f64) {
    (max - p.y + MARGIN, p.x + MARGIN)
}

/// Draws one routed edge: dashed curve, filled arrow head and tail spot.
///
/// Ported from `JsonCurve.drawCurve/drawSpot` + `Arrow.drawArrow`.
fn draw_edge(svg: &mut SvgGraphics, edge: &crate::layout::EdgeRoute, max: f64) {
    let pts: [(f64, f64); 4] = edge.points.map(|p| tf(p, max));

    // veryFirst = P0 backed 13 units toward the incoming direction.
    let (x0, y0) = pts[0];
    let (x1, y1) = pts[1];
    let full = (x1 - x0).hypot(y1 - y0);
    let very_first = if full > 1e-9 {
        (x0 + (x0 - x1) / full * 13.0, y0 + (y0 - y1) / full * 13.0)
    } else {
        (x0, y0)
    };

    // Dashed curve: M veryFirst L P0 C P1 P2 P3.
    let d = format!(
        "M{} L{} C{} {} {}",
        num(very_first.0) + "," + &num(very_first.1),
        num(x0) + "," + &num(y0),
        num(pts[1].0) + "," + &num(pts[1].1),
        num(pts[2].0) + "," + &num(pts[2].1),
        num(pts[3].0) + "," + &num(pts[3].1),
    );
    svg.set_fill_color("none");
    svg.set_stroke_color(Some(NODE_STROKE));
    svg.set_stroke_width(SEPARATOR_WIDTH, Some([3.0, 3.0]));
    svg.svg_path(&d, 0.0);

    // Filled arrow head (p1 = P3 base, p2 = ep tip).
    if let Some(ep) = edge.ep {
        let tip = tf(ep, max);
        draw_arrow(svg, pts[3], tip);
    }

    // Tail spot: filled r=3 circle at veryFirst.
    svg.set_fill_color(NODE_STROKE);
    svg.set_stroke_color(Some(NODE_STROKE));
    svg.set_stroke_width(SEPARATOR_WIDTH, None);
    svg.svg_ellipse(very_first.0, very_first.1, 3.0, 3.0, 0.0);
}

/// Draws the filled normal-arrow polygon. Ported from `Arrow.drawArrow`.
fn draw_arrow(svg: &mut SvgGraphics, p1: (f64, f64), p2: (f64, f64)) {
    let dist = (p2.0 - p1.0).hypot(p2.1 - p1.1);
    let alpha = (p2.0 - p1.0).atan2(p2.1 - p1.1);
    let point = |ang: f64, len: f64| -> (f64, f64) {
        (p1.0 + len * ang.sin(), p1.1 + len * ang.cos())
    };
    let p3 = point(alpha + std::f64::consts::FRAC_PI_2, dist * 0.4);
    let p4 = point(alpha - std::f64::consts::FRAC_PI_2, dist * 0.4);
    let p11 = point(alpha, dist * 0.3);

    let d = format!(
        "M{} L{} L{} L{} L{}",
        num(p4.0) + "," + &num(p4.1),
        num(p11.0) + "," + &num(p11.1),
        num(p3.0) + "," + &num(p3.1),
        num(p2.0) + "," + &num(p2.1),
        num(p4.0) + "," + &num(p4.1),
    );
    svg.set_fill_color(NODE_STROKE);
    svg.set_stroke_width(0.0, None);
    svg.svg_path(&d, 0.0);
}

/// Formats a coordinate like the reference path output: integer when whole,
/// otherwise up to four decimals with trailing zeros removed.
fn num(v: f64) -> String {
    let r = (v * 10000.0).round() / 10000.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        let s = format!("{r:.4}");
        s.trim_end_matches('0').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_tables_match_reference() {
        assert!((bold_width("key") - 24.9199).abs() < 0.001);
        assert!((bold_width("count") - 40.3338).abs() < 0.001);
        assert!((plain_width("value") - 35.1259).abs() < 0.001);
        assert!((plain_width("42") - 16.0159).abs() < 0.001);
    }

    #[test]
    fn renders_simple_object() {
        let json: Value = serde_json::json!({ "key": "value", "count": 42 });
        let svg = render_json_svg(&json, &[], "JSON");
        assert!(svg.contains("key"));
        assert!(svg.contains("value"));
    }

    #[test]
    fn highlight_parsing() {
        assert!(Highlight::matches_definition("#highlight foo.bar"));
        assert!(!Highlight::matches_definition("#some other directive"));
        let h = Highlight::build("#highlight foo.bar").unwrap();
        assert_eq!(h.path, "foo.bar");
    }
}

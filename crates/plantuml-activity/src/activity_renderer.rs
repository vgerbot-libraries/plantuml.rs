//! Activity diagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/activitydiagram/ActivityDiagram.drawU()`.
//!
//! Renders activity nodes as a vertical flowchart.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::activity_parser::{ActivityNode, ActivityNodeType, ActivitySource};

// ── Constants ────────────────────────────────────────────────────────────

const NODE_WIDTH: f64 = 140.0;
const NODE_HEIGHT: f64 = 35.0;
const V_GAP: f64 = 15.0;
const MARGIN: f64 = 20.0;
const STROKE: &str = "#181818";
const FILL_ACTION: &str = "#F2F2F2";
const FILL_START: &str = "#181818";
const FILL_DECISION: &str = "#FFFFFF";
const COLOR_TEXT: &str = "#000000";
const FONT_SIZE: i32 = 12;
const FONT_FAMILY: &str = "sans-serif";
const START_RADIUS: f64 = 8.0;

/// Renders an activity diagram as SVG.
#[must_use]
pub fn render_activity_svg(source: &ActivitySource, diagram_label: &str) -> String {
    let mut option = SvgOption::basic();
    option.set_title(diagram_label.to_string());

    let mut svg = SvgGraphics::new(0, option);

    if source.nodes.is_empty() {
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), "#0000FF".to_string());
        svg.text(
            "Empty activity diagram",
            10.0,
            20.0,
            Some("monospace"),
            14,
            Some("normal"),
            Some("normal"),
            None,
            200.0,
            &attrs,
            None,
        );
        return svg.create_xml();
    }

    let center_x = MARGIN + NODE_WIDTH / 2.0;
    let mut y = MARGIN;

    for (i, node) in source.nodes.iter().enumerate() {
        render_node(&mut svg, node, center_x, y);

        // Draw connector to next node.
        if i + 1 < source.nodes.len() {
            let connector_end = y + node_height(node) + V_GAP;
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.5, None);
            svg.svg_line(center_x, y + node_height(node), center_x, connector_end, 0.0);

            // Arrowhead.
            svg.svg_polygon(0.0, &[
                center_x, connector_end,
                center_x - 4.0, connector_end - 6.0,
                center_x + 4.0, connector_end - 6.0,
            ]);

            y = connector_end;
        }
    }

    svg.create_xml()
}

/// Returns the height of a node based on its type.
fn node_height(node: &ActivityNode) -> f64 {
    match node.node_type {
        ActivityNodeType::Start | ActivityNodeType::Stop => START_RADIUS * 2.0,
        ActivityNodeType::If | ActivityNodeType::While => NODE_HEIGHT + 10.0,
        _ => NODE_HEIGHT,
    }
}

/// Renders a single activity node.
fn render_node(svg: &mut SvgGraphics, node: &ActivityNode, center_x: f64, y: f64) {
    match node.node_type {
        ActivityNodeType::Start => {
            // Filled circle.
            svg.set_fill_color(FILL_START);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.5, None);
            svg.svg_ellipse(center_x, y + START_RADIUS, START_RADIUS, START_RADIUS, 0.0);
        }
        ActivityNodeType::Stop => {
            // Circle with filled inner circle.
            svg.set_fill_color("#FFFFFF");
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.5, None);
            svg.svg_ellipse(center_x, y + START_RADIUS, START_RADIUS, START_RADIUS, 0.0);
            svg.set_fill_color(FILL_START);
            svg.svg_ellipse(center_x, y + START_RADIUS, START_RADIUS * 0.5, START_RADIUS * 0.5, 0.0);
        }
        ActivityNodeType::Action => {
            // Rounded rectangle.
            let x = center_x - NODE_WIDTH / 2.0;
            svg.set_fill_color(FILL_ACTION);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(x, y, NODE_WIDTH, NODE_HEIGHT, 15.0, 15.0, 0.0);
            render_text(svg, &node.label, center_x, y + NODE_HEIGHT / 2.0, COLOR_TEXT, true);
        }
        ActivityNodeType::If | ActivityNodeType::While => {
            // Diamond.
            let h = node_height(node);
            let half_w = NODE_WIDTH / 2.0;
            let half_h = h / 2.0;
            svg.set_fill_color(FILL_DECISION);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_polygon(0.0, &[
                center_x, y,
                center_x + half_w, y + half_h,
                center_x, y + h,
                center_x - half_w, y + half_h,
            ]);
            render_text(svg, &node.label, center_x, y + half_h, COLOR_TEXT, true);
        }
        ActivityNodeType::Else => {
            // Just a label on the branch.
            render_text(svg, &node.label, center_x + 30.0, y + NODE_HEIGHT / 2.0, COLOR_TEXT, false);
        }
        ActivityNodeType::EndIf | ActivityNodeType::EndWhile | ActivityNodeType::EndFork => {
            // Merge point (small filled circle).
            svg.set_fill_color(FILL_START);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_ellipse(center_x, y + 5.0, 5.0, 5.0, 0.0);
        }
        ActivityNodeType::Fork | ActivityNodeType::ForkAgain => {
            // Fork bar (horizontal thick line).
            svg.set_fill_color(FILL_START);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(center_x - NODE_WIDTH / 2.0, y, NODE_WIDTH, 5.0, 0.0, 0.0, 0.0);
        }
        ActivityNodeType::Note => {
            // Note box (yellow with folded corner).
            let x = center_x + NODE_WIDTH / 2.0 + 10.0;
            svg.set_fill_color("#FBFB77");
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(x, y, 80.0, 25.0, 0.0, 0.0, 0.0);
            render_text(svg, &node.label, x + 40.0, y + 12.5, COLOR_TEXT, true);
        }
    }
}

/// Renders centered text.
fn render_text(svg: &mut SvgGraphics, text: &str, center_x: f64, center_y: f64, color: &str, center: bool) {
    let label_w = text_width(text, FONT_SIZE);
    let x = if center { center_x - label_w / 2.0 } else { center_x };
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), color.to_string());
    svg.text(
        text,
        x,
        f64::from(FONT_SIZE).mul_add(0.35, center_y),
        Some(FONT_FAMILY),
        FONT_SIZE,
        Some("normal"),
        Some("normal"),
        None,
        label_w,
        &attrs,
        None,
    );
}

fn text_width(text: &str, font_size: i32) -> f64 {
    f64::from(font_size) * 0.6 * text.chars().count() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_parser::parse_activity_source;

    #[test]
    fn test_render_activity_svg() {
        let lines = vec!["start", ":Do something;", "stop"];
        let source = parse_activity_source(&lines);
        let svg = render_activity_svg(&source, "(Activity)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Do something"));
    }

    #[test]
    fn test_render_empty_activity() {
        let source = ActivitySource::default();
        let svg = render_activity_svg(&source, "(Activity)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Empty"));
    }
}

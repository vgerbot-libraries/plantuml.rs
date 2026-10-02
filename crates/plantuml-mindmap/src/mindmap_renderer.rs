//! Mindmap SVG renderer.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/mindmap/MindMap.java` (two-side orchestration)
//! - `net/sourceforge/plantuml/mindmap/FingerImpl.java` (phalanx + link)
//!
//! Geometry comes from [`crate::layout`]; this module only walks the laid tree
//! and emits the rounded-rectangle nodes and cubic `FingerImpl` connectors in
//! the reference draw order: a node's own box, then each whole child subtree,
//! then the connector to that child.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::idea::Idea;
use crate::layout::{layout, LaidNode};
use crate::text_metrics::{baseline_y, BOX_PADDING};

/// Point size of every node label.
const FONT_SIZE: i32 = 14;
/// Node corner radius (`FtileBoxOld` mindmap style).
const CORNER_RADIUS: f64 = 12.5;
/// Box fill.
const FILL_COLOR: &str = "#F1F1F1";
/// Box/link stroke.
const STROKE_COLOR: &str = "#181818";
/// Box border width.
const BOX_STROKE: f64 = 1.5;
/// Link stroke width.
const LINK_STROKE: f64 = 1.0;
/// Text color.
const TEXT_COLOR: &str = "#000000";
/// Horizontal lead before the cubic, from a node edge.
const LEAD: f64 = 10.0;
/// Horizontal cubic control offset from a node edge.
const CONTROL: f64 = 25.0;

/// Draws one node box at its rectangle origin.
fn draw_box(svg: &mut SvgGraphics, node: &LaidNode, x: f64, y: f64) {
    svg.set_fill_color(FILL_COLOR);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(BOX_STROKE, None);
    svg.svg_rectangle(x, y, node.width, node.height, CORNER_RADIUS, CORNER_RADIUS, 0.0);

    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), TEXT_COLOR.to_string());
    svg.text(
        &node.label,
        x + BOX_PADDING,
        y + baseline_y(FONT_SIZE),
        Some("sans-serif"),
        FONT_SIZE,
        None,
        None,
        None,
        node.width - 2.0 * BOX_PADDING,
        &attrs,
        None,
    );
}

/// Builds the cubic `FingerImpl` path between a parent edge and a child edge.
///
/// `right` selects the facing direction. `px`/`py` are the parent anchor (the
/// box edge at parent centre); `cx`/`cy` are the child anchor (the facing box
/// edge at child centre).
fn link_path(right: bool, px: f64, py: f64, cx: f64, cy: f64) -> String {
    if right {
        // M parent L parent+lead C parent+control … child-control child-lead L child
        format!(
            "M{p},{py} L{pl},{py} C{pc},{py} {cc},{cy} {cl},{cy} L{c},{cy}",
            p = num(px),
            pl = num(px + LEAD),
            pc = num(px + CONTROL),
            cc = num(cx - CONTROL),
            cl = num(cx - LEAD),
            c = num(cx),
        )
    } else {
        // Mirrored: lead/control run inward from each edge.
        format!(
            "M{p},{py} L{pl},{py} C{pc},{py} {cc},{cy} {cl},{cy} L{c},{cy}",
            p = num(px),
            pl = num(px - LEAD),
            pc = num(px - CONTROL),
            cc = num(cx + CONTROL),
            cl = num(cx + LEAD),
            c = num(cx),
        )
    }
}

/// Formats an f64 the way the path data is written (trimmed, enough decimals).
fn num(v: f64) -> String {
    let s = format!("{v:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Recursively draws one side.
///
/// `ox`/`oy` are the page coordinates of this node's finger origin. A
/// right-facing node anchors its box left edge (left-facing: right edge) at
/// `ox`; the box is vertically centred on `oy`. Each child finger origin is
/// offset by the parent box width plus 50 horizontally and its packed `cy`.
fn draw_side(svg: &mut SvgGraphics, node: &LaidNode, ox: f64, oy: f64, right: bool) {
    // Own box: edge-anchored at ox, vertically centred on oy. The reverse
    // root of a two-sided map draws nothing (the regular side draws it).
    if node.draw_phalanx {
        let box_x = if right { ox } else { ox - node.phalanx_e };
        let box_y = oy - node.height / 2.0;
        draw_box(svg, node, box_x, box_y);
    }

    for child in &node.children {
        // Child finger origin: parent phalanx width + getX12 (50).
        let (child_ox, child_oy) = if right {
            (ox + node.phalanx_e + 50.0, oy + child.cy)
        } else {
            (ox - node.phalanx_e - 50.0, oy + child.cy)
        };

        // Whole child subtree first…
        draw_side(svg, child, child_ox, child_oy, right);

        // …then the connector, from the parent facing edge at parent centre
        // to the child finger origin at child centre.
        let parent_edge = if right {
            ox + node.phalanx_e
        } else {
            ox - node.phalanx_e
        };
        svg.set_stroke_color(Some(STROKE_COLOR));
        svg.set_stroke_width(LINK_STROKE, None);
        svg.set_fill_color("none");
        svg.svg_path(
            &link_path(right, parent_edge, oy, child_ox, child_oy),
            0.0,
        );
    }
}

/// Renders a mindmap `Idea` tree as an SVG string.
#[must_use]
pub fn render_mindmap_svg(root: &Idea) -> String {
    let laid = layout(root);

    let mut option = SvgOption::basic();
    option.set_root_attribute("data-diagram-type", "MINDMAP");
    option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    option.set_min_dim(laid.canvas_w.ceil(), laid.canvas_h.ceil());

    let mut svg = SvgGraphics::new(0, option);
    if let Some(node) = &laid.right.root {
        draw_side(&mut svg, node, laid.root_ox, laid.root_oy, true);
    }
    if let Some(node) = &laid.left.root {
        draw_side(&mut svg, node, laid.root_ox, laid.root_oy, false);
    }
    svg.create_xml()
}

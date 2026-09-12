//! Mindmap SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/mindmap/FingerImpl.java`,
//! `Tetris.java`, `SymetricalTee.java`.
//!
//! The Java version uses a complex FingerImpl/Tetris layout algorithm.
//! This Rust implementation uses a simpler recursive tree layout:
//! root at center, right children branch right, left children branch left,
//! with curved Bézier connectors.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::idea::{Idea, IdeaShape, MindMapDirection};

// ── Layout constants ─────────────────────────────────────────────────────

/// Horizontal padding inside each box (each side).
const PADDING_H: f64 = 8.0;
/// Vertical padding inside each box (top + bottom).
const PADDING_V: f64 = 4.0;
/// Horizontal gap between a parent and its children.
const BRANCH_GAP: f64 = 50.0;
/// Vertical spacing between sibling nodes.
const SIBLING_SPACING: f64 = 6.0;
/// Stroke color for boxes.
const STROKE_COLOR: &str = "#181818";
/// Fill color for root node.
const FILL_ROOT: &str = "#F2F2F2";
/// Fill color for regular nodes.
const FILL_NODE: &str = "#F2F2F2";
/// Fill color for boxless nodes (transparent).
const FILL_NONE: &str = "none";
/// Text color.
const COLOR_TEXT: &str = "#000000";
/// Stroke width for box borders.
const STROKE_WIDTH: f64 = 1.5;
/// Stroke width for curves.
const STROKE_WIDTH_CURVE: f64 = 1.5;
/// Font size.
const FONT_SIZE: i32 = 12;
/// Font family.
const FONT_FAMILY: &str = "sans-serif";
/// Page margin.
const PAGE_MARGIN: f64 = 10.0;

/// Computed layout for a mindmap node.
struct MindmapLayout {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    center_x: f64,
    center_y: f64,
    right_children: Vec<Self>,
    left_children: Vec<Self>,
    label: String,
    shape: IdeaShape,
    back_color: Option<String>,
    is_root: bool,
}

/// Approximate text width.
fn text_width(text: &str) -> f64 {
    f64::from(FONT_SIZE) * 0.6 * text.chars().count() as f64
}

/// Text height for the font size.
fn text_height() -> f64 {
    f64::from(FONT_SIZE) * 1.2
}

/// Computes the layout for a subtree, returning the layout and the total height.
fn compute_subtree(
    idea: &Idea,
    x: f64,
    y: f64,
    _direction: MindMapDirection,
) -> MindmapLayout {
    let label_w = text_width(&idea.label);
    let width = PADDING_H.mul_add(2.0, label_w);
    let height = PADDING_V.mul_add(2.0, text_height());

    // Separate children by direction.
    let (right_children, left_children): (Vec<_>, Vec<_>) = idea
        .children
        .iter()
        .partition(|c| c.direction == MindMapDirection::Right);

    // Compute right children layout.
    let mut right_layouts = Vec::new();
    let mut right_y = y;
    for child in &right_children {
        let child_x = x + width + BRANCH_GAP;
        let layout = compute_subtree(child, child_x, right_y, MindMapDirection::Right);
        right_y += layout_height(&layout) + SIBLING_SPACING;
        right_layouts.push(layout);
    }
    if !right_layouts.is_empty() {
        right_y -= SIBLING_SPACING;
    }

    // Compute left children layout.
    let mut left_layouts = Vec::new();
    let mut left_y = y;
    for child in &left_children {
        let child_w = PADDING_H.mul_add(2.0, text_width(&child.label));
        let child_x = x - child_w - BRANCH_GAP;
        let layout = compute_subtree(child, child_x, left_y, MindMapDirection::Left);
        left_y += layout_height(&layout) + SIBLING_SPACING;
        left_layouts.push(layout);
    }
    if !left_layouts.is_empty() {
        left_y -= SIBLING_SPACING;
    }

    // Total height = max(self height, right children height, left children height).
    let right_h = if right_layouts.is_empty() {
        0.0
    } else {
        right_y - y
    };
    let left_h = if left_layouts.is_empty() {
        0.0
    } else {
        left_y - y
    };
    let total_h = height.max(right_h).max(left_h);

    // Center this node vertically within the total height.
    let center_y = y + total_h / 2.0;
    let node_y = center_y - height / 2.0;
    let center_x = x + width / 2.0;

    MindmapLayout {
        x,
        y: node_y,
        width,
        height,
        center_x,
        center_y,
        right_children: right_layouts,
        left_children: left_layouts,
        label: idea.label.clone(),
        shape: idea.shape,
        back_color: idea.back_color.clone(),
        is_root: idea.is_root(),
    }
}

/// Returns the total height of a layout subtree.
fn layout_height(layout: &MindmapLayout) -> f64 {
    let right_h: f64 = layout
        .right_children
        .iter()
        .map(layout_height)
        .sum::<f64>()
        + if layout.right_children.is_empty() {
            0.0
        } else {
            SIBLING_SPACING * (layout.right_children.len() - 1) as f64
        };
    let left_h: f64 = layout
        .left_children
        .iter()
        .map(layout_height)
        .sum::<f64>()
        + if layout.left_children.is_empty() {
            0.0
        } else {
            SIBLING_SPACING * (layout.left_children.len() - 1) as f64
        };
    layout.height.max(right_h).max(left_h)
}

/// Computes the bounding box of a layout tree.
fn bounding_box(layout: &MindmapLayout) -> (f64, f64, f64, f64) {
    let mut min_x = layout.x;
    let mut min_y = layout.y;
    let mut max_x = layout.x + layout.width;
    let mut max_y = layout.y + layout.height;

    for child in &layout.right_children {
        let (cmin_x, cmin_y, cmax_x, cmax_y) = bounding_box(child);
        min_x = min_x.min(cmin_x);
        min_y = min_y.min(cmin_y);
        max_x = max_x.max(cmax_x);
        max_y = max_y.max(cmax_y);
    }
    for child in &layout.left_children {
        let (cmin_x, cmin_y, cmax_x, cmax_y) = bounding_box(child);
        min_x = min_x.min(cmin_x);
        min_y = min_y.min(cmin_y);
        max_x = max_x.max(cmax_x);
        max_y = max_y.max(cmax_y);
    }

    (min_x, min_y, max_x, max_y)
}

/// Renders a mindmap layout tree to SVG.
fn render_node(svg: &mut SvgGraphics, layout: &MindmapLayout, offset_x: f64, offset_y: f64) {
    let x = layout.x + offset_x;
    let y = layout.y + offset_y;
    let _ = layout.center_x + offset_x;
    let cy = layout.center_y + offset_y;

    let fill = if layout.shape == IdeaShape::None {
        FILL_NONE
    } else if let Some(ref bc) = layout.back_color {
        bc.as_str()
    } else if layout.is_root {
        FILL_ROOT
    } else {
        FILL_NODE
    };

    // Draw box (unless boxless).
    if layout.shape != IdeaShape::None {
        svg.set_fill_color(fill);
        svg.set_stroke_color(Some(STROKE_COLOR));
        svg.set_stroke_width(STROKE_WIDTH, None);
        svg.svg_rectangle(x, y, layout.width, layout.height, 5.0, 5.0, 0.0);
    }

    // Draw label text.
    let text_x = x + PADDING_H;
    let text_y = f64::from(FONT_SIZE).mul_add(0.85, y + PADDING_V);
    let tw = text_width(&layout.label);
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        &layout.label,
        text_x,
        text_y,
        Some(FONT_FAMILY),
        FONT_SIZE,
        Some("normal"),
        Some("normal"),
        None,
        tw,
        &attrs,
        None,
    );

    // Draw curves to right children.
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_CURVE, None);
    svg.set_fill_color("none");

    for child in &layout.right_children {
        let child_cy = child.center_y + offset_y;
        // Curve from right-center of parent to left-center of child.
        let x1 = x + layout.width;
        let y1 = cy;
        let x2 = child.x + offset_x;
        let y2 = child_cy;
        let cx1 = (x2 - x1).mul_add(0.5, x1);
        let cy1 = y1;
        let cx2 = (x2 - x1).mul_add(0.5, x1);
        let cy2 = y2;
        let d = format!(
            "M{x1:.1},{y1:.1} C{cx1:.1},{cy1:.1} {cx2:.1},{cy2:.1} {x2:.1},{y2:.1}"
        );
        svg.svg_path(&d, 0.0);
        render_node(svg, child, offset_x, offset_y);
    }

    // Draw curves to left children.
    for child in &layout.left_children {
        let child_cy = child.center_y + offset_y;
        // Curve from left-center of parent to right-center of child.
        let x1 = x;
        let y1 = cy;
        let x2 = child.x + offset_x + child.width;
        let y2 = child_cy;
        let cx1 = (x2 - x1).mul_add(0.5, x1);
        let cy1 = y1;
        let cx2 = (x2 - x1).mul_add(0.5, x1);
        let cy2 = y2;
        let d = format!(
            "M{x1:.1},{y1:.1} C{cx1:.1},{cy1:.1} {cx2:.1},{cy2:.1} {x2:.1},{y2:.1}"
        );
        svg.svg_path(&d, 0.0);
        render_node(svg, child, offset_x, offset_y);
    }
}

/// Renders a mindmap `Idea` tree as an SVG string.
#[must_use]
pub fn render_mindmap_svg(root: &Idea) -> String {
    // Compute layout starting from a nominal position.
    let layout = compute_subtree(root, PAGE_MARGIN, PAGE_MARGIN, MindMapDirection::Right);

    // Compute bounding box to determine SVG dimensions and offsets.
    let (min_x, min_y, max_x, max_y) = bounding_box(&layout);
    let offset_x = PAGE_MARGIN - min_x;
    let offset_y = PAGE_MARGIN - min_y;
    let _total_w = PAGE_MARGIN.mul_add(2.0, max_x - min_x);
    let _total_h = PAGE_MARGIN.mul_add(2.0, max_y - min_y);

    let mut option = SvgOption::basic();
    option.set_title("(Mindmap)");

    let mut svg = SvgGraphics::new(0, option);

    // Render the tree with offsets to ensure all content is visible.
    render_node(&mut svg, &layout, offset_x, offset_y);

    svg.create_xml()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idea::{parse_mindmap_orgmode, parse_mindmap_plus};

    #[test]
    fn test_render_simple_mindmap() {
        let lines = vec!["* root", "** a", "*** a1", "** b"];
        let tree = parse_mindmap_orgmode(&lines).unwrap();
        let svg = render_mindmap_svg(&tree);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("root"));
        assert!(svg.contains("a1"));
    }

    #[test]
    fn test_render_plus_syntax() {
        let lines = vec!["+ root", "++ right", "-- left"];
        let tree = parse_mindmap_plus(&lines).unwrap();
        let svg = render_mindmap_svg(&tree);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("root"));
        assert!(svg.contains("right"));
        assert!(svg.contains("left"));
    }

    #[test]
    fn test_render_boxless() {
        let lines = vec!["* root", "**_ boxless"];
        let tree = parse_mindmap_orgmode(&lines).unwrap();
        let svg = render_mindmap_svg(&tree);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("boxless"));
    }
}

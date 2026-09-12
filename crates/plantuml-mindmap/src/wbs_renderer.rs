//! WBS SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/wbs/Fork.java`,
//! `ITFComposed.java`, `ITFLeaf.java`, `WBSTextBlock.java`.
//!
//! WBS uses a top-down tree layout with orthogonal connectors:
//! vertical line from parent bottom to midpoint, horizontal line spanning
//! children, vertical lines down to each child.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::idea::IdeaShape;
use crate::wbs_element::WElement;

// ── Layout constants (from Java Fork/ITFComposed) ────────────────────────

/// Horizontal gap between sibling children.
const DELTA1X: f64 = 20.0;
/// Vertical gap from parent to children row.
const DELTAY: f64 = 40.0;
/// Horizontal padding inside each box.
const PADDING_H: f64 = 8.0;
/// Vertical padding inside each box.
const PADDING_V: f64 = 4.0;
/// Stroke color for boxes.
const STROKE_COLOR: &str = "#181818";
/// Fill color for nodes.
const FILL_NODE: &str = "#F2F2F2";
/// Text color.
const COLOR_TEXT: &str = "#000000";
/// Stroke width.
const STROKE_WIDTH: f64 = 1.5;
/// Font size.
const FONT_SIZE: i32 = 12;
/// Font family.
const FONT_FAMILY: &str = "sans-serif";
/// Page margin.
const PAGE_MARGIN: f64 = 10.0;

/// Computed layout for a WBS node.
struct WbsLayout {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    center_x: f64,
    children: Vec<Self>,
    label: String,
    shape: IdeaShape,
    back_color: Option<String>,
}

fn text_width(text: &str) -> f64 {
    f64::from(FONT_SIZE) * 0.6 * text.chars().count() as f64
}

fn text_height() -> f64 {
    f64::from(FONT_SIZE) * 1.2
}

/// Computes the layout for a WBS subtree.
///
/// Top-down layout: parent at top, children below.
fn compute_wbs_layout(element: &WElement, x: f64, y: f64) -> WbsLayout {
    let label_w = text_width(&element.label);
    let width = PADDING_H.mul_add(2.0, label_w);
    let height = PADDING_V.mul_add(2.0, text_height());

    if element.children.is_empty() {
        return WbsLayout {
            x,
            y,
            width,
            height,
            center_x: x + width / 2.0,
            children: Vec::new(),
            label: element.label.clone(),
            shape: element.shape,
            back_color: element.back_color.clone(),
        };
    }

    // Compute children layouts.
    let child_y = y + height + DELTAY;
    let mut child_layouts = Vec::new();
    let mut child_x = x;

    for child in &element.children {
        let layout = compute_wbs_layout(child, child_x, child_y);
        child_x += layout.width + DELTA1X;
        child_layouts.push(layout);
    }

    // Remove trailing gap.
    if !child_layouts.is_empty() {
        child_x -= DELTA1X;
    }

    let children_total_w = child_x - x;

    // Center parent over children.
    let parent_width = width.max(children_total_w);
    let parent_x = x + (children_total_w - width) / 2.0;
    let center_x = x + parent_width / 2.0;

    // Adjust children to be centered under parent.
    let children_offset = (parent_width - children_total_w) / 2.0;
    if children_offset > 0.0 && !child_layouts.is_empty() {
        for child in &mut child_layouts {
            child.x += children_offset;
            child.center_x += children_offset;
            shift_children(child, children_offset);
        }
    }

    WbsLayout {
        x: parent_x,
        y,
        width,
        height,
        center_x,
        children: child_layouts,
        label: element.label.clone(),
        shape: element.shape,
        back_color: element.back_color.clone(),
    }
}

/// Recursively shifts children by dx.
fn shift_children(layout: &mut WbsLayout, dx: f64) {
    for child in &mut layout.children {
        child.x += dx;
        child.center_x += dx;
        shift_children(child, dx);
    }
}

/// Computes the bounding box.
fn bounding_box(layout: &WbsLayout) -> (f64, f64, f64, f64) {
    let mut min_x = layout.x;
    let mut min_y = layout.y;
    let mut max_x = layout.x + layout.width;
    let mut max_y = layout.y + layout.height;

    for child in &layout.children {
        let (cmin_x, cmin_y, cmax_x, cmax_y) = bounding_box(child);
        min_x = min_x.min(cmin_x);
        min_y = min_y.min(cmin_y);
        max_x = max_x.max(cmax_x);
        max_y = max_y.max(cmax_y);
    }

    (min_x, min_y, max_x, max_y)
}

/// Renders a WBS layout tree to SVG.
fn render_wbs_node(svg: &mut SvgGraphics, layout: &WbsLayout, offset_x: f64, offset_y: f64) {
    let x = layout.x + offset_x;
    let y = layout.y + offset_y;
    let cx = layout.center_x + offset_x;

    let fill = if layout.shape == IdeaShape::None {
        "none"
    } else if let Some(ref bc) = layout.back_color {
        bc.as_str()
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

    // Draw label.
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

    // Draw orthogonal connectors to children.
    if !layout.children.is_empty() {
        svg.set_stroke_color(Some(STROKE_COLOR));
        svg.set_stroke_width(STROKE_WIDTH, None);
        svg.set_fill_color("none");

        let parent_bottom = y + layout.height;
        let midpoint_y = parent_bottom + DELTAY / 2.0;

        // Vertical line from parent bottom to midpoint.
        svg.svg_line(cx, parent_bottom, cx, midpoint_y, 0.0);

        // Horizontal line spanning all children centers.
        let first_child_cx = layout.children.first().unwrap().center_x + offset_x;
        let last_child_cx = layout.children.last().unwrap().center_x + offset_x;
        svg.svg_line(first_child_cx, midpoint_y, last_child_cx, midpoint_y, 0.0);

        // Vertical lines from midpoint down to each child top.
        for child in &layout.children {
            let child_cx = child.center_x + offset_x;
            let child_top = child.y + offset_y;
            svg.svg_line(child_cx, midpoint_y, child_cx, child_top, 0.0);
        }

        // Recursively render children.
        for child in &layout.children {
            render_wbs_node(svg, child, offset_x, offset_y);
        }
    }
}

/// Renders a WBS `WElement` tree as an SVG string.
#[must_use]
pub fn render_wbs_svg(root: &WElement) -> String {
    let layout = compute_wbs_layout(root, PAGE_MARGIN, PAGE_MARGIN);

    let (min_x, min_y, max_x, max_y) = bounding_box(&layout);
    let offset_x = PAGE_MARGIN - min_x;
    let offset_y = PAGE_MARGIN - min_y;
    let _total_w = PAGE_MARGIN.mul_add(2.0, max_x - min_x);
    let _total_h = PAGE_MARGIN.mul_add(2.0, max_y - min_y);

    let mut option = SvgOption::basic();
    option.set_title("(WBS)");

    let mut svg = SvgGraphics::new(0, option);

    render_wbs_node(&mut svg, &layout, offset_x, offset_y);

    svg.create_xml()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wbs_element::parse_wbs_tree;

    #[test]
    fn test_render_simple_wbs() {
        let lines = vec!["* Project", "** Planning", "*** Define scope", "** Implementation"];
        let tree = parse_wbs_tree(&lines).unwrap();
        let svg = render_wbs_svg(&tree);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Project"));
        assert!(svg.contains("Define scope"));
    }

    #[test]
    fn test_render_wbs_with_boxless() {
        let lines = vec!["* root", "**_ boxless", "** normal"];
        let tree = parse_wbs_tree(&lines).unwrap();
        let svg = render_wbs_svg(&tree);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("boxless"));
        assert!(svg.contains("normal"));
    }
}

//! WBS SVG renderer.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/wbs/Fork.java` (root, `delta1x = 20`)
//! - `net/sourceforge/plantuml/wbs/ITFComposed.java` (`delta1x = 10`)
//! - `net/sourceforge/plantuml/wbs/ITFLeaf.java`
//! - `net/sourceforge/plantuml/wbs/WBSDiagram.java` (margin/delta handling)
//!
//! Coordinate origins (outermost first):
//! 1. exporter page margin — 10 pixels on every side (`TitledDiagram` defaults);
//! 2. `WBSDiagram.drawU` — an explicit `UTranslate(10, 10)`;
//! 3. the `Fork` local coordinate system.
//!
//! The canvas is sized from the final dimension (`Fork` + 20 for the diagram
//! margin + 20 for the page margin), which the SVG `minDim` enforces.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::text_metrics::{baseline_y, box_size, BOX_PADDING};
use crate::wbs_element::WElement;

/// Fork sibling gap (`Fork.delta1x`).
const FORK_DELTA_X: f64 = 20.0;
/// Fork parent-to-child vertical gap (`Fork.deltay`).
const FORK_DELTA_Y: f64 = 40.0;
/// Composed-node sibling/anchor gap (`ITFComposed.delta1x`).
const COMPOSED_DELTA_X: f64 = 10.0;
/// Vertical margin above a composed node's child row (node style margin bottom).
const MARGIN_BOTTOM: f64 = 15.0;

const STROKE_COLOR: &str = "#181818";
const FILL_COLOR: &str = "#F1F1F1";
const TEXT_COLOR: &str = "#000000";
const STROKE_WIDTH: f64 = 1.5;
const FONT_SIZE: i32 = 12;

/// Page (exporter) margin plus the explicit `WBSDiagram` (10, 10) translate.
const ORIGIN: f64 = 20.0;
/// Total canvas enlargement over the raw `Fork` dimension (20 + 20).
const CANVAS_DELTA: f64 = 40.0;

/// 2D dimension plus the anchor points Java's `ITF` exposes.
#[derive(Clone, Copy)]
struct Metrics {
    w: f64,
    h: f64,
    /// `getT1` — top anchor.
    t1x: f64,
    /// `getF1` — left-middle anchor.
    f1x: f64,
    /// `getF2` — right-middle anchor.
    f2x: f64,
    /// y component shared by F1/F2.
    f_y: f64,
}

/// A laid-out WBS subtree in its own local coordinates.
struct Node {
    label: String,
    main_w: f64,
    main_h: f64,
    /// Left and right child subtrees (only `right` is used with `*` syntax).
    left: Vec<Self>,
    right: Vec<Self>,
}

impl Node {
    fn build(el: &WElement) -> Self {
        let (main_w, main_h) = box_size(&el.label, FONT_SIZE);
        let left = Vec::new();
        let mut right = Vec::new();
        for child in &el.children {
            // Star-syntax children are always "right" (Direction.RIGHT default).
            right.push(Self::build(child));
        }
        // WElement parsing here does not split by direction (all right); keep
        // `left` available to mirror ITFComposed if direction tags appear.
        debug_assert!(left.is_empty());
        Self { label: el.label.clone(), main_w, main_h, left, right }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_empty() && self.right.is_empty()
    }

    /// `ITFLeaf` metrics: the box itself, anchors at the edges.
    fn leaf_metrics(&self) -> Metrics {
        Metrics {
            w: self.main_w,
            h: self.main_h,
            t1x: self.main_w / 2.0,
            f1x: 0.0,
            f2x: self.main_w,
            f_y: self.main_h / 2.0,
        }
    }

    fn coll_width(list: &[Self]) -> f64 {
        list.iter().map(Node::metrics).fold(0.0, |acc, m| acc.max(m.w))
    }

    fn coll_height(list: &[Self]) -> f64 {
        list.iter()
            .map(|c| MARGIN_BOTTOM + c.metrics().h)
            .sum()
    }

    /// `ITFComposed`/`ITFLeaf` metrics depending on the node kind.
    fn metrics(&self) -> Metrics {
        if self.is_leaf() {
            return self.leaf_metrics();
        }
        let w1 = (self.main_w / 2.0).max(COMPOSED_DELTA_X + Node::coll_width(&self.left));
        let w_right =
            (self.main_w / 2.0).max(COMPOSED_DELTA_X + Node::coll_width(&self.right));
        let h = self.main_h
            + Node::coll_height(&self.left)
                .max(Node::coll_height(&self.right));
        Metrics {
            w: w1 + w_right,
            h,
            t1x: w1,
            f1x: w1 - self.main_w / 2.0,
            f2x: w1 + self.main_w / 2.0,
            f_y: self.main_h / 2.0,
        }
    }

    /// Draws the node box at the given origin.
    fn draw_box(&self, svg: &mut SvgGraphics, x: f64, y: f64) {
        svg.set_fill_color(FILL_COLOR);
        svg.set_stroke_color(Some(STROKE_COLOR));
        svg.set_stroke_width(STROKE_WIDTH, None);
        svg.svg_rectangle(x, y, self.main_w, self.main_h, 0.0, 0.0, 0.0);

        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), TEXT_COLOR.to_string());
        svg.text(
            &self.label,
            x + BOX_PADDING,
            y + baseline_y(FONT_SIZE),
            Some("sans-serif"),
            FONT_SIZE,
            None,
            None,
            None,
            self.main_w - 2.0 * BOX_PADDING,
            &attrs,
            None,
        );
    }

    /// `ITFComposed.drawU` / `ITFLeaf.drawU` at translate `(ox, oy)`.
    fn draw(&self, svg: &mut SvgGraphics, ox: f64, oy: f64) {
        if self.is_leaf() {
            self.draw_box(svg, ox, oy);
            return;
        }
        let m = self.metrics();
        let w1 = m.t1x;

        // Main box, offset so its centre sits on the anchor line.
        self.draw_box(svg, ox + w1 - self.main_w / 2.0, oy);

        svg.set_stroke_color(Some(STROKE_COLOR));
        svg.set_stroke_width(STROKE_WIDTH, None);
        svg.set_fill_color("none");

        // Right children.
        let mut y = self.main_h;
        let mut last_y = y;
        for child in &self.right {
            y += MARGIN_BOTTOM;
            let cm = child.metrics();
            last_y = y + cm.f_y;
            svg.svg_line(
                ox + w1,
                oy + last_y,
                ox + w1 + COMPOSED_DELTA_X + cm.f1x,
                oy + last_y,
                0.0,
            );
            child.draw(svg, ox + w1 + COMPOSED_DELTA_X, oy + y);
            y += cm.h;
        }
        let last_right = last_y;

        // Left children (mirrored).
        let mut y = self.main_h;
        let mut last_y = y;
        for child in &self.left {
            y += MARGIN_BOTTOM;
            let cm = child.metrics();
            last_y = y + cm.f_y;
            svg.svg_line(
                ox + w1,
                oy + last_y,
                ox + w1 - COMPOSED_DELTA_X - cm.f2x,
                oy + last_y,
                0.0,
            );
            child.draw(svg, ox + w1 - COMPOSED_DELTA_X - cm.w, oy + y);
            y += cm.h;
        }
        let last_left = last_y;

        // Vertical spine from the box bottom to the deepest connector.
        svg.svg_line(
            ox + w1,
            oy + self.main_h,
            ox + w1,
            oy + last_left.max(last_right),
            0.0,
        );
    }
}

/// Raw `Fork.calculateDimension` (excluding the diagram/page margins).
fn fork_dimension(root: &Node) -> (f64, f64) {
    if root.right.is_empty() {
        return (root.main_w, root.main_h + FORK_DELTA_Y);
    }
    let mut width = 0.0_f64;
    let mut max_h = 0.0_f64;
    for child in &root.right {
        let cm = child.metrics();
        width += cm.w;
        max_h = max_h.max(cm.h);
    }
    width += FORK_DELTA_X * (root.right.len() - 1) as f64;
    let height = root.main_h + FORK_DELTA_Y + max_h;
    (width.max(root.main_w), height)
}

/// `Fork.drawU` at translate `(dx, dy)` in the parent canvas.
fn draw_fork(svg: &mut SvgGraphics, root: &Node, dx: f64, dy: f64) {
    let y0 = root.main_h;
    let y1 = dy + y0 + FORK_DELTA_Y / 2.0;

    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH, None);
    svg.set_fill_color("none");

    if root.right.is_empty() {
        root.draw_box(svg, dx, dy);
        svg.svg_line(dx + root.main_w / 2.0, dy + y0, dx + root.main_w / 2.0, y1, 0.0);
        return;
    }

    let y2 = dy + y0 + FORK_DELTA_Y;
    let mut x = dx;
    let mut first_x = 0.0;
    let mut last_x = 0.0;
    for (i, child) in root.right.iter().enumerate() {
        let cm = child.metrics();
        last_x = x + cm.t1x;
        if i == 0 {
            first_x = last_x;
        }
        svg.svg_line(last_x, y1, last_x, y2, 0.0);
        child.draw(svg, x, y2);
        x += cm.w + FORK_DELTA_X;
    }

    let pos_main;
    if last_x > first_x {
        svg.svg_line(first_x, y1, last_x, y1, 0.0);
        pos_main = first_x + (last_x - first_x - root.main_w) / 2.0;
    } else {
        let (fork_w, _) = fork_dimension(root);
        pos_main = dx + (fork_w - root.main_w) / 2.0;
        svg.svg_line(first_x, y1, pos_main + root.main_w / 2.0, y1, 0.0);
    }
    root.draw_box(svg, pos_main, dy);
    svg.svg_line(pos_main + root.main_w / 2.0, dy + y0, pos_main + root.main_w / 2.0, y1, 0.0);
}

/// Renders a WBS `WElement` tree as an SVG string.
#[must_use]
pub fn render_wbs_svg(root_el: &WElement) -> String {
    let root = Node::build(root_el);
    let (fork_w, fork_h) = fork_dimension(&root);

    let mut option = SvgOption::basic();
    option.set_root_attribute("data-diagram-type", "WBS");
    option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    option.set_min_dim(fork_w + CANVAS_DELTA, fork_h + CANVAS_DELTA);

    let mut svg = SvgGraphics::new(0, option);
    draw_fork(&mut svg, &root, ORIGIN, ORIGIN);
    svg.create_xml()
}

//! Activity diagram SVG renderer.
//!
//! Ported from the ftile rendering pipeline of
//! `net/sourceforge/plantuml/activitydiagram3/ActivityDiagram3.getTextBlock`:
//! the `Swimlanes` ftile tree (start/stop circles, action boxes,
//! `FtileIfWithLinks` conditionals, `FtileWhile` loops) followed by the
//! `CompressionXorY` ON_X/ON_Y compression and the `Recentred` translation.
//!
//! Shapes are emitted first; connector snakes (lines + arrowheads) are
//! buffered and flushed last, mirroring `UGraphicForSnake`.

use indexmap::IndexMap;
use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;
use plantuml_klimt::string_bounder_svg::StringBounderSvg;
use plantuml_svg::{SvgGraphics, SvgOption};

use crate::activity_parser::{ActivityBlock, ActivitySource};

// ── Colors ────────────────────────────────────────────────────────────────

const CIRCLE: &str = "#222222";
const STROKE: &str = "#181818";
const FILL_BOX: &str = "#F1F1F1";
const TEXT_COLOR: &str = "#000000";

// ── Font metrics (AWT SansSerif, measured from the jar) ──────────────────

/// AWT line height factor (`size * 1.362`).
const LINE_FACTOR: f64 = 1.362;
/// AWT ascender factor (`size * 1.069`).
const ASCENT_FACTOR: f64 = 1.069;
/// Action / start-stop body font size.
const SIZE_ACTION: i32 = 12;
/// Decision / branch-label font size.
const SIZE_LABEL: i32 = 11;

/// Padding around an action's text (10 on every side).
const BOX_PADDING: f64 = 10.0;
/// Rounded-corner radius of an action box (`roundCorner 25 / 2`).
const BOX_CORNER: f64 = 12.5;

/// Half-width of a decision hexagon's pointed end.
const HEX_HALF: f64 = 12.0;
/// Minimum width of a branch tile (`FtileMinWidthCentered 30`).
const BRANCH_MIN_WIDTH: f64 = 30.0;
/// Horizontal margin added around a branch tile.
const BRANCH_MARGIN: f64 = 10.0;
/// Vertical gap between the decision diamond and a branch.
const GAP_DIAMOND_BRANCH: f64 = 10.0;
/// Vertical gap between a branch and the merge diamond (two branches).
const GAP_BRANCH_MERGE: f64 = 6.0;
/// Empty space added between two assembled tiles (`FtileFactoryDelegatorAssembly`).
const ASSEMBLY_SPACE: f64 = 35.0;

/// Extra width reserved inside an if around the decision diamond
/// (`SUPP_WIDTH = 20`).
const IF_SUPP_WIDTH: f64 = 20.0;

/// Extra vertical space inside a while tile (`FtileWhile`: `+48`).
const WHILE_EXTRA_BOTTOM: f64 = 48.0;
/// Extra horizontal space on the right of a while tile (`24 + 12`).
const WHILE_EXTRA_RIGHT: f64 = 36.0;

// ── Geometry ──────────────────────────────────────────────────────────────

/// Connection-point geometry of a laid-out tile.
///
/// Ported from `FtileGeometry`. `out_y == None` means the tile has no outgoing
/// connection point.
#[derive(Debug, Clone)]
struct Geo {
    width: f64,
    height: f64,
    left: f64,
    in_y: f64,
    out_y: Option<f64>,
}

impl Geo {
    /// Horizontal extent to the right of the `left` connection column.
    fn right(&self) -> f64 {
        self.width - self.left
    }
}

// ── Shapes & connectors ───────────────────────────────────────────────────

/// A drawable node shape in swimlanes coordinates.
#[derive(Debug, Clone)]
enum Shape {
    /// Ellipse given by top-left corner and radii.
    Ellipse {
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        fill: &'static str,
        stroke_width: f64,
    },
    /// Rounded rectangle.
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rx: f64,
    },
    /// Polygon given as flat `x,y` pairs.
    Polygon {
        points: Vec<f64>,
        fill: &'static str,
        stroke_width: f64,
    },
    /// Single-line text.
    Text {
        text: String,
        x: f64,
        y: f64,
        size: i32,
        text_length: f64,
    },
    /// Invisible marker that still registers for layout/compression
    /// (`UEmpty`).
    Empty { x: f64, y: f64, w: f64, h: f64 },
}

/// Direction an arrowhead points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// A connector: an ordered polyline (`points`) drawn in order, with an optional
/// arrowhead at each end and an optional emphasized arrow at the midpoint of
/// the first segment facing a given direction.
#[derive(Debug, Clone)]
struct Connector {
    points: Vec<(f64, f64)>,
    start_arrow: Option<Dir>,
    end_arrow: Option<Dir>,
    emphasize: Option<Dir>,
}

impl Connector {
    /// Returns every arrowhead polygon (as flat coordinate pairs) that this
    /// connector draws.
    fn arrowheads(&self) -> Vec<Vec<f64>> {
        let mut result = Vec::new();
        if let Some(d) = self.start_arrow {
            result.push(translate_poly(arrow_poly(d), self.points[0]));
        }
        if let Some(d) = self.end_arrow {
            result.push(translate_poly(arrow_poly(d), *self.points.last().unwrap()));
        }
        if let Some(d) = self.emphasize {
            // First segment whose direction matches carries a centered arrow.
            for pair in self.points.windows(2) {
                let (x1, y1) = pair[0];
                let (x2, y2) = pair[1];
                if segment_dir(x1, y1, x2, y2) == Some(d) {
                    result.push(translate_poly(
                        arrow_poly(d),
                        ((x1 + x2) / 2.0, (y1 + y2) / 2.0),
                    ));
                    break;
                }
            }
        }
        result
    }

    /// Returns only the start/end arrowhead polygons (no emphasized one).
    fn terminal_arrowheads(&self) -> Vec<Vec<f64>> {
        let mut result = Vec::new();
        if let Some(d) = self.start_arrow {
 result.push(translate_poly(arrow_poly(d), self.points[0]));
        }
        if let Some(d) = self.end_arrow {
            result.push(translate_poly(arrow_poly(d), *self.points.last().unwrap()));
        }
        result
    }
}

/// A laid-out diagram: shapes first, connectors flushed last.
#[derive(Debug, Default)]
struct Canvas {
    shapes: Vec<Shape>,
    connectors: Vec<Connector>,
}

// ── Arrowhead geometry (ArrowsRegular) ──────────────────────────────────

/// Returns the local polygon (tip at the origin) for an arrow pointing in `d`.
fn arrow_poly(d: Dir) -> Vec<f64> {
    match d {
        Dir::Down => vec![-4.0, -10.0, 0.0, 0.0, 4.0, -10.0, 0.0, -6.0],
        Dir::Up => vec![-4.0, 10.0, 0.0, 0.0, 4.0, 10.0, 0.0, 6.0],
        Dir::Right => vec![-10.0, -4.0, 0.0, 0.0, -10.0, 4.0, -6.0, 0.0],
        Dir::Left => vec![10.0, -4.0, 0.0, 0.0, 10.0, 4.0, 6.0, 0.0],
    }
}

/// Translates a flat polygon so its origin lands at `(dx,dy)`.
fn translate_poly(poly: Vec<f64>, (dx, dy): (f64, f64)) -> Vec<f64> {
    poly.chunks(2)
        .flat_map(|p| [p[0] + dx, p[1] + dy])
        .collect()
}

/// Axis tolerance: two coordinates on the same grid line differ by no more
/// than this after layout arithmetic.
const AXIS_EPS: f64 = 1e-9;

/// True when two coordinates denote the same axis line.
fn same_axis(a: f64, b: f64) -> bool {
    (a - b).abs() < AXIS_EPS
}

/// Direction of a non-degenerate segment.
fn segment_dir(x1: f64, y1: f64, x2: f64, y2: f64) -> Option<Dir> {
    if same_axis(x1, x2) && y1 < y2 {
        Some(Dir::Down)
    } else if same_axis(x1, x2) && y1 > y2 {
        Some(Dir::Up)
    } else if same_axis(y1, y2) && x1 < x2 {
        Some(Dir::Right)
    } else if same_axis(y1, y2) && x1 > x2 {
        Some(Dir::Left)
    } else {
        None
    }
}

// ── Layout ────────────────────────────────────────────────────────────────

/// Context for measuring text.
struct Layout<'a> {
    bounder: &'a StringBounderSvg,
}

impl Layout<'_> {
    /// Width of `text` at the given font size.
    fn text_width(&self, size: i32, text: &str) -> f64 {
        self.bounder
            .calculate_dimension(&UFont::sans_serif(size), text)
            .width()
    }
}

/// Lays out a full diagram into a swimlanes-coordinate canvas.
fn layout_diagram(source: &ActivitySource, bounder: &StringBounderSvg) -> Canvas {
    let mut ctx = Layout { bounder };
    let mut canvas = Canvas::default();

    // A top-level Stop immediately following a While is absorbed as the
    // while's special output (ActivityDiagram3.manageSpecialStopEndAfterEndWhile).
    let mut blocks: Vec<&ActivityBlock> = Vec::new();
    let mut i = 0;
    while i < source.blocks.len() {
        if source.blocks[i] == ActivityBlock::Stop {
            if let Some(ActivityBlock::While { .. }) = blocks.last().copied() {
                i += 1;
                continue;
            }
        }
        blocks.push(&source.blocks[i]);
        i += 1;
    }

    // Assemble top-level blocks vertically.
    let mut left = 0.0_f64;
    let mut geos: Vec<Geo> = Vec::new();
    for block in &blocks {
        let geo = block_geo(&mut ctx, block);
        left = left.max(geo.left);
        geos.push(geo);
    }

    let mut y = 0.0_f64;
    let mut prev_out: Option<(f64, f64)> = None;
    for (idx, block) in blocks.iter().enumerate() {
        let dx = left - geos[idx].left;
        // Draws the block's shapes and its internal connectors (these flush
        // before the assembly connector entering it).
        layout_block(&mut ctx, &mut canvas, block, dx, y);

        // Assembly connector from the previous tile's out to this tile's in.
        if idx > 0 {
            if let Some(p1) = prev_out {
                // This tile's pointIn translates to the global left column.
                let p2 = (left, y);
                canvas.connectors.push(Connector {
                    points: vec![p1, p2],
                    start_arrow: None,
                    end_arrow: Some(Dir::Down),
                    emphasize: None,
                });
            }
        }

        if idx + 1 < blocks.len() {
            prev_out = geos[idx]
                .out_y
                .map(|oy| (dx + geos[idx].left, y + oy));
            y += geos[idx].height + ASSEMBLY_SPACE;
        }
    }

    canvas
}

/// Lays out one block at offset `(dx,dy)` into `canvas`.
fn layout_block(ctx: &mut Layout, canvas: &mut Canvas, block: &ActivityBlock, dx: f64, dy: f64) {
    match block {
        ActivityBlock::Start => {
            canvas.shapes.push(Shape::Ellipse {
                x: dx,
                y: dy,
                rx: 10.0,
                ry: 10.0,
                fill: CIRCLE,
                stroke_width: 1.0,
            });
        }
        ActivityBlock::Stop => {
            // Outer circle, then filled inner circle.
            canvas.shapes.push(Shape::Ellipse {
                x: dx,
                y: dy,
                rx: 11.0,
                ry: 11.0,
                fill: "none",
                stroke_width: 1.0,
            });
            canvas.shapes.push(Shape::Ellipse {
                x: dx + 5.0,
                y: dy + 5.0,
                rx: 6.0,
                ry: 6.0,
                fill: CIRCLE,
                stroke_width: 1.0,
            });
        }
        ActivityBlock::Action(label) => {
            let geo = action_geo(ctx, label);
            canvas.shapes.push(Shape::Rect {
                x: dx,
                y: dy,
                w: geo.width,
                h: geo.height,
                rx: BOX_CORNER,
            });
            let tw = ctx.text_width(SIZE_ACTION, label);
            canvas.shapes.push(Shape::Text {
                text: label.clone(),
                x: dx + BOX_PADDING,
                y: dy + BOX_PADDING + asc(SIZE_ACTION),
                size: SIZE_ACTION,
                text_length: tw,
            });
        }
        ActivityBlock::If {
            condition,
            then_label,
            then_block,
            else_label,
            else_block,
        } => layout_if(
            ctx,
            canvas,
            condition,
            then_label.as_deref(),
            then_block,
            else_label.as_deref(),
            else_block.as_deref(),
            dx,
            dy,
        ),
        ActivityBlock::While {
            condition,
            yes_label,
            body,
            out_label,
        } => layout_while(
            ctx,
            canvas,
            condition,
            yes_label.as_deref(),
            body,
            out_label.as_deref(),
            dx,
            dy,
        ),
    }
}

/// Ascender height for a font size.
fn asc(size: i32) -> f64 {
    f64::from(size) * ASCENT_FACTOR
}

/// Line height for a font size.
#[allow(dead_code)]
fn line_h(size: i32) -> f64 {
    f64::from(size) * LINE_FACTOR
}

/// Geometry of an action box.
fn action_geo(ctx: &Layout, label: &str) -> Geo {
    let tw = ctx.text_width(SIZE_ACTION, label);
    let w = tw + 2.0 * BOX_PADDING;
    let h = line_h(SIZE_ACTION) + 2.0 * BOX_PADDING;
    Geo {
        width: w,
        height: h,
        left: w / 2.0,
        in_y: 0.0,
        out_y: Some(h),
    }
}

// ── Sequence (branch/body) layout ────────────────────────────────────────

/// A laid-out sub-sequence together with its geometry.
struct SubLayout {
    geo: Geo,
    shapes: Vec<(Shape, f64, f64)>,
}

/// Lays out a list of blocks as a vertically assembled sub-sequence, with
/// coordinates relative to its own origin.
fn layout_sequence(ctx: &mut Layout, blocks: &[ActivityBlock]) -> SubLayout {
    let mut canvas = Canvas::default();
    let mut geos: Vec<Geo> = Vec::new();
    let mut left = 0.0_f64;
    for b in blocks {
        let g = block_geo(ctx, b);
        left = left.max(g.left);
        geos.push(g);
    }
    let mut y = 0.0_f64;
    for (idx, b) in blocks.iter().enumerate() {
        let dx = left - geos[idx].left;
        layout_block(ctx, &mut canvas, b, dx, y);
        let bottom = y + geos[idx].height;
        if idx + 1 < blocks.len() {
            let next = &geos[idx + 1];
            let next_in_x = left + (next.left - geos[idx + 1].left);
            if let Some(out_y) = geos[idx].out_y {
                canvas.connectors.push(Connector {
                    points: vec![
                        (dx + geos[idx].left, y + out_y),
                        (next_in_x, bottom + ASSEMBLY_SPACE),
                    ],
                    start_arrow: None,
                    end_arrow: Some(Dir::Down),
                    emphasize: None,
                });
            }
            y = bottom + ASSEMBLY_SPACE;
        }
    }
    let total_h = if blocks.is_empty() {
        0.0
    } else {
        y + geos.last().unwrap().height
    };
    let total_w = geos.iter().fold(0.0_f64, |acc, g| {
        acc.max(g.width + (left - g.left))
    });
    let geo = Geo {
        width: total_w,
        height: total_h,
        left,
        in_y: geos.first().map_or(0.0, |g| g.in_y),
        out_y: geos.last().and_then(|g| {
            g.out_y.map(|oy| {
                let last_y = total_h - geos.last().unwrap().height;
                last_y + oy
            })
        }),
    };
    let shapes = canvas
        .shapes
        .into_iter()
        .map(|s| (s, 0.0, 0.0))
        .collect();
    SubLayout {
        geo,
        shapes,
    }
}

/// Geometry of any single block.
fn block_geo(ctx: &mut Layout, block: &ActivityBlock) -> Geo {
    match block {
        ActivityBlock::Start => Geo {
            width: 20.0,
            height: 20.0,
            left: 10.0,
            in_y: 0.0,
            out_y: Some(20.0),
        },
        ActivityBlock::Stop => Geo {
            width: 22.0,
            height: 22.0,
            left: 11.0,
            in_y: 0.0,
            out_y: None,
        },
        ActivityBlock::Action(label) => action_geo(ctx, label),
        ActivityBlock::If {
            condition,
            then_label,
            then_block,
            else_label,
            else_block,
        } => if_geo(
            ctx,
            condition,
            then_label.as_deref(),
            then_block,
            else_label.as_deref(),
            else_block.as_deref(),
        ),
        ActivityBlock::While {
            condition,
            yes_label,
            body,
            out_label,
        } => while_geo(
            ctx,
            condition,
            yes_label.as_deref(),
            body,
            out_label.as_deref(),
        ),
    }
}

// ── Branch wrapper ───────────────────────────────────────────────────────

/// A branch tile after `FtileMinWidthCentered(30)` and horizontal margin 10.
struct Branch {
    geo: Geo,
    /// Offset of the wrapped tile's origin relative to the branch origin.
    inner_dx: f64,
}

/// Wraps a sub-sequence as a centered min-width branch with a 10-unit margin.
fn wrap_branch(sub: &SubLayout) -> Branch {
    let raw = &sub.geo;
    let (w, inner_dx, left) = if raw.width < BRANCH_MIN_WIDTH {
        let dx = (BRANCH_MIN_WIDTH - raw.width) / 2.0;
        (BRANCH_MIN_WIDTH, dx, raw.left + dx)
    } else {
        (raw.width, 0.0, raw.left)
    };
    // Add 10-unit horizontal margin on each side.
    Branch {
        geo: Geo {
            width: w + 2.0 * BRANCH_MARGIN,
            height: raw.height,
            left: left + BRANCH_MARGIN,
            in_y: raw.in_y,
            out_y: raw.out_y,
        },
        inner_dx: inner_dx + BRANCH_MARGIN,
    }
}

// ── IF ───────────────────────────────────────────────────────────────────

/// Computes the decision-diamond (diamond1) size for a condition.
fn decision_size(ctx: &Layout, condition: &str) -> (f64, f64) {
    let tw = ctx.text_width(SIZE_LABEL, condition);
    let th = line_h(SIZE_LABEL);
    let w = tw.max(24.0) + 24.0;
    let h = th.max(24.0);
    (w, h)
}

/// Geometry of an if construct.
#[allow(clippy::too_many_arguments)]
fn if_geo(
    ctx: &mut Layout,
    condition: &str,
    _then_label: Option<&str>,
    then_block: &[ActivityBlock],
    _else_label: Option<&str>,
    else_block: Option<&[ActivityBlock]>,
) -> Geo {
    let then_sub = layout_sequence(ctx, then_block);
    let then_branch = wrap_branch(&then_sub);
    let (dw, dh) = decision_size(ctx, condition);
    let else_branch = else_block.map(|b| wrap_branch(&layout_sequence(ctx, b)));
    let (width, height, left) = if_dims(&then_branch, else_branch.as_ref(), dw, dh);
    Geo {
        width,
        height,
        left,
        in_y: 0.0,
        out_y: Some(height),
    }
}

/// Local IF geometry `(width, height, left)` for the wrapped branches.
fn if_dims(then_branch: &Branch, else_branch: Option<&Branch>, dw: f64, dh: f64) -> (f64, f64, f64) {
    // Without an else branch, the second branch is the then branch itself and
    // no merge-diamond gap is added.
    let b1 = then_branch;
    let b2 = else_branch.unwrap_or(then_branch);
    let inner = (dw + IF_SUPP_WIDTH).max(b1.geo.right() + b2.geo.left);
    let w = b1.geo.left + inner + b2.geo.right();
    let left = b1.geo.left + inner / 2.0;
    let branch_h = b1.geo.height.max(b2.geo.height);
    let tail = if else_branch.is_some() {
        GAP_BRANCH_MERGE + 24.0
    } else {
        0.0
    };
    let h = dh + GAP_DIAMOND_BRANCH + branch_h + tail;
    (w, h, left)
}

/// Lays out an if construct at `(dx,dy)`.
#[allow(clippy::too_many_arguments)]
fn layout_if(
    ctx: &mut Layout,
    canvas: &mut Canvas,
    condition: &str,
    then_label: Option<&str>,
    then_block: &[ActivityBlock],
    else_label: Option<&str>,
    else_block: Option<&[ActivityBlock]>,
    dx: f64,
    dy: f64,
) {
    let then_sub = layout_sequence(ctx, then_block);
    let then_branch = wrap_branch(&then_sub);
    let else_sub_opt: Option<(SubLayout, Branch)> = else_block.map(|eb| {
        let sub = layout_sequence(ctx, eb);
        let branch = wrap_branch(&sub);
        (sub, branch)
    });

    let (dw, dh) = decision_size(ctx, condition);
    // Local IF geometry relative to the tile origin.
    let (w, h, left) = if_dims(
        &then_branch,
        else_sub_opt.as_ref().map(|(_, b)| b),
        dw,
        dh,
    );

    // Decision diamond: translate = (left - dw/2, 0).
    let dia_x = dx + left - dw / 2.0;
    push_hexagon(canvas, dia_x, dy, dw, dh);
    push_inner_label(canvas, condition, dia_x, dy, dw, dh, ctx);

    // West/east branch labels.
    let yes = then_label.unwrap_or("yes");
    push_label(canvas, yes, dia_x, dy, dw, dh, Dir::Left, ctx);
    let no = else_label.unwrap_or("no");
    push_label(canvas, no, dia_x, dy, dw, dh, Dir::Right, ctx);

    // Branch rows.
    let by = dh + GAP_DIAMOND_BRANCH;
    let c1 = then_branch.geo.left;
    push_sub(canvas, &then_sub, dx + then_branch.inner_dx, dy + by);

    if let Some((else_sub, else_branch)) = &else_sub_opt {
        let b2w = else_branch.geo.width;
        let branch2_ox = w - b2w;
        let c2 = branch2_ox + else_branch.geo.left;
        push_sub(
            canvas,
            else_sub,
            dx + branch2_ox + else_branch.inner_dx,
            dy + by,
        );

        // Merge diamond (24x24) at (left-12, h-24).
        let merge_x = dx + left - 12.0;
        let merge_y = dy + h - 24.0;
        canvas.shapes.push(Shape::Polygon {
            points: diamond_poly(merge_x, merge_y, 24.0, 24.0),
            fill: FILL_BOX,
            stroke_width: 0.5,
        });

        let dia_mid_y = dy + dh / 2.0;
        let branch_out_y = dy + by + then_branch.geo.height;
        let merge_mid_y = dy + h - 12.0;

        // Diamond west → branch1.
        canvas.connectors.push(Connector {
            points: vec![
                (dia_x, dia_mid_y),
                (dx + c1, dia_mid_y),
                (dx + c1, dy + by),
            ],
            start_arrow: None,
            end_arrow: Some(Dir::Down),
            emphasize: None,
        });
        // Diamond east → branch2.
        canvas.connectors.push(Connector {
            points: vec![
                (dia_x + dw, dia_mid_y),
                (dx + c2, dia_mid_y),
                (dx + c2, dy + by),
            ],
            start_arrow: None,
            end_arrow: Some(Dir::Down),
            emphasize: None,
        });
        // Branch1 out → merge west.
        canvas.connectors.push(Connector {
            points: vec![
                (dx + c1, branch_out_y),
                (dx + c1, merge_mid_y),
                (merge_x, merge_mid_y),
            ],
            start_arrow: None,
            end_arrow: Some(Dir::Right),
            emphasize: None,
        });
        // Branch2 out → merge east.
        canvas.connectors.push(Connector {
            points: vec![
                (dx + c2, branch_out_y),
                (dx + c2, merge_mid_y),
                (merge_x + 24.0, merge_mid_y),
            ],
            start_arrow: None,
            end_arrow: Some(Dir::Left),
            emphasize: None,
        });
    }
}

/// Pushes the decision hexagon polygon (`FtileDiamondInside` background).
fn push_hexagon(canvas: &mut Canvas, x: f64, y: f64, w: f64, h: f64) {
    canvas.shapes.push(Shape::Polygon {
        points: hex_poly(x, y, w, h),
        fill: FILL_BOX,
        stroke_width: 0.5,
    });
}

/// Pushes a decision's centered inner label.
fn push_inner_label(
    canvas: &mut Canvas,
    condition: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    ctx: &Layout,
) {
    let tw = ctx.text_width(SIZE_LABEL, condition);
    let tx = x + (w - tw) / 2.0;
    let ty = y + (h - line_h(SIZE_LABEL)) / 2.0 + asc(SIZE_LABEL);
    canvas.shapes.push(Shape::Text {
        text: condition.to_string(),
        x: tx,
        y: ty,
        size: SIZE_LABEL,
        text_length: tw,
    });
}

/// Pushes a west/east branch label next to a decision diamond.
#[allow(clippy::too_many_arguments)]
fn push_label(
    canvas: &mut Canvas,
    label: &str,
    dia_x: f64,
    dia_y: f64,
    dia_w: f64,
    dia_h: f64,
    side: Dir,
    ctx: &Layout,
) {
    let lw = ctx.text_width(SIZE_LABEL, label);
    let lh = line_h(SIZE_LABEL);
    let origin_y = dia_y - lh + dia_h / 2.0;
    // West label ends at the diamond's left edge; east label starts at the
    // diamond's right edge.
    let x = if side == Dir::Left {
        dia_x - lw
    } else {
        dia_x + dia_w
    };
    canvas.shapes.push(Shape::Text {
        text: label.to_string(),
        x,
        y: origin_y + asc(SIZE_LABEL),
        size: SIZE_LABEL,
        text_length: lw,
    });
}


/// Pushes a sub-sequence's shapes at an offset.
fn push_sub(canvas: &mut Canvas, sub: &SubLayout, dx: f64, dy: f64) {
    for (shape, ox, oy) in &sub.shapes {
        canvas.shapes.push(shift_shape(shape, dx + ox, dy + oy));
    }
}

/// Clones a shape shifted by `(dx,dy)`.
fn shift_shape(shape: &Shape, dx: f64, dy: f64) -> Shape {
    let mut s = shape.clone();
    match &mut s {
        Shape::Polygon { points, .. } => {
            for c in points.chunks_exact_mut(2) {
                c[0] += dx;
                c[1] += dy;
            }
        }
        Shape::Ellipse { x, y, .. }
        | Shape::Rect { x, y, .. }
        | Shape::Text { x, y, .. }
        | Shape::Empty { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
    }
    s
}

/// Builds the decision hexagon polygon (long hexagon via `Hexagon.asPolygon`).
fn hex_poly(x: f64, y: f64, w: f64, h: f64) -> Vec<f64> {
    vec![
        x + HEX_HALF,
        y,
        x + w - HEX_HALF,
        y,
        x + w,
        y + h / 2.0,
        x + w - HEX_HALF,
        y + h,
        x + HEX_HALF,
        y + h,
        x,
        y + h / 2.0,
        x + HEX_HALF,
        y,
    ]
}

/// Builds a four-point diamond polygon.
fn diamond_poly(x: f64, y: f64, w: f64, h: f64) -> Vec<f64> {
    vec![
        x + w / 2.0,
        y,
        x + w,
        y + h / 2.0,
        x + w / 2.0,
        y + h,
        x,
        y + h / 2.0,
        x + w / 2.0,
        y,
    ]
}

// ── WHILE ───────────────────────────────────────────────────────────────

/// Geometry of a while construct (with its special output absorbed).
#[allow(clippy::too_many_arguments)]
fn while_geo(
    ctx: &mut Layout,
    condition: &str,
    yes_label: Option<&str>,
    body: &[ActivityBlock],
    _out_label: Option<&str>,
) -> Geo {
    let body_sub = layout_sequence(ctx, body);
    let (mw, mh, mleft) = while_inner(ctx, condition, yes_label, &body_sub.geo);
    Geo {
        // FtileWhile: special-out width (22) + merged width + 24 + 12.
        width: SPECIAL_WIDTH + mw + WHILE_EXTRA_RIGHT,
        height: mh + WHILE_EXTRA_BOTTOM,
        left: SPECIAL_WIDTH + mleft + 24.0,
        in_y: 0.0,
        out_y: None,
    }
}

/// Width of the special stop tile (`CircleEnd`, 22).
const SPECIAL_WIDTH: f64 = 22.0;

/// Computes the decision ftile for a while and merges it above the body,
/// returning the merged `(width, height, left)`.
fn while_inner(
    ctx: &mut Layout,
    condition: &str,
    yes_label: Option<&str>,
    body_geo: &Geo,
) -> (f64, f64, f64) {
    let yes = yes_label.unwrap_or("yes");
    // Decision polygon: inner condition dim at least 24, plus 24 in width.
    let tw = ctx.text_width(SIZE_LABEL, condition);
    let th = line_h(SIZE_LABEL);
    let diamond_w = tw.max(24.0) + 24.0;
    let polygon_h = th.max(24.0);
    // North ("yes") label appends its height under the polygon.
    let north_h = line_h(SIZE_LABEL);
    let _ = ctx.text_width(SIZE_LABEL, yes);
    let diamond_h = polygon_h + north_h;
    let diamond_left = diamond_w / 2.0;

    // appendBottom: align the left columns, then merge the extents.
    let left = diamond_left.max(body_geo.left);
    let width = (diamond_w + (left - diamond_left))
        .max(body_geo.width + (left - body_geo.left));
    let height = diamond_h + body_geo.height;
    (width, height, left)
}

/// Lays out a while construct at global offset `(dx,dy)`.
#[allow(clippy::too_many_arguments, clippy::similar_names)]
fn layout_while(
    ctx: &mut Layout,
    canvas: &mut Canvas,
    condition: &str,
    yes_label: Option<&str>,
    body: &[ActivityBlock],
    out_label: Option<&str>,
    dx: f64,
    dy: f64,
) {
    let body_sub = layout_sequence(ctx, body);
    let yes = yes_label.unwrap_or("yes");

    // Decision polygon/ftile metrics.
    let tw = ctx.text_width(SIZE_LABEL, condition);
    let th = line_h(SIZE_LABEL);
    let diamond_w = tw.max(24.0) + 24.0;
    let polygon_h = th.max(24.0);
    let north_h = line_h(SIZE_LABEL);
    let diamond_h = polygon_h + north_h;
    let diamond_left = diamond_w / 2.0;

    let body_geo = &body_sub.geo;
    let left = diamond_left.max(body_geo.left);

    // Final ftile geometry.
    let total_left = dx + SPECIAL_WIDTH + left + 24.0;
    let total_w = dx
        + SPECIAL_WIDTH
        + (diamond_w + (left - diamond_left))
            .max(body_geo.width + (left - body_geo.left))
        + WHILE_EXTRA_RIGHT;

    // Child translates (FtileWhile getTranslateFor*).
    let body_tx = total_left - body_geo.left;
    let body_ty = dy + diamond_h + 24.0;
    let diamond_tx = total_left - diamond_left;

    // ── Shapes, in drawU order: body, diamond, special. ──
    push_sub(canvas, &body_sub, body_tx, body_ty);

    // Decision hexagon.
    push_hexagon(canvas, diamond_tx, dy, diamond_w, polygon_h);
    // North "yes" label (left-aligned at 4 + diamond_w/2).
    let yw = ctx.text_width(SIZE_LABEL, yes);
    canvas.shapes.push(Shape::Text {
        text: yes.to_string(),
        x: diamond_tx + 4.0 + diamond_w / 2.0,
        y: dy + polygon_h + asc(SIZE_LABEL),
        size: SIZE_LABEL,
        text_length: yw,
    });
    push_inner_label(canvas, condition, diamond_tx, dy, diamond_w, polygon_h, ctx);
    let no = out_label.unwrap_or("no");
    let nw = ctx.text_width(SIZE_LABEL, no);
    let nh = line_h(SIZE_LABEL);
    canvas.shapes.push(Shape::Text {
        text: no.to_string(),
        x: diamond_tx - nw,
        y: dy - nh + polygon_h / 2.0 + asc(SIZE_LABEL),
        size: SIZE_LABEL,
        text_length: nw,
    });

    // Special stop translate: (min(body_tx-12, diamond_tx) - 22, dy+48).
    let special_x = (body_tx - 12.0).min(diamond_tx) - SPECIAL_WIDTH;
    let special_y = dy + WHILE_EXTRA_BOTTOM;
    layout_block(ctx, canvas, &ActivityBlock::Stop, special_x, special_y);

    // ── Connectors, flush order: In, BackSimple, OutSpecial. ──
    let center_x = total_left;

    // ConnectionIn: diamond south → body north.
    let in_p1 = (center_x, dy + polygon_h);
    let in_p2 = (center_x, body_ty);
    canvas.connectors.push(Connector {
        points: vec![in_p1, in_p2],
        start_arrow: None,
        end_arrow: Some(Dir::Down),
        emphasize: None,
    });

    // ConnectionBackSimple: body south → down 12 → right to total width →
    // up → left into the diamond east.
    if let Some(body_out_y) = body_geo.out_y {
        let p1 = (body_tx + body_geo.left, body_ty + body_out_y);
        let y_bis = p1.1 + 12.0;
        let y2 = dy + polygon_h / 2.0;
        let x2 = diamond_tx + diamond_w;
        canvas.connectors.push(Connector {
            points: vec![
                p1,
                (p1.0, y_bis),
                (total_w, y_bis),
                (total_w, y2),
                (x2, y2),
            ],
            start_arrow: None,
            end_arrow: Some(Dir::Left),
            emphasize: Some(Dir::Up),
        });
        // UEmpty(5,12) at the loop-back corner registers the bottom slot.
        canvas.shapes.push(Shape::Empty {
            x: p1.0,
            y: y_bis,
            w: 5.0,
            h: 12.0,
        });
    }

    // ConnectionOutSpecial: diamond west → horizontal → down into stop.
    let special_in = (special_x + 11.0, special_y);
    let diamond_west = (diamond_tx, dy + polygon_h / 2.0);
    canvas.connectors.push(Connector {
        points: vec![
            diamond_west,
            (special_in.0, diamond_west.1),
            special_in,
        ],
        start_arrow: None,
        end_arrow: Some(Dir::Down),
        emphasize: None,
    });
}

// ── Compression (SlotFinder / SlotSet) ───────────────────────────────────

/// Returns the ON_Y occupied range a shape contributes to `SlotFinder`.
fn shape_slot_y(shape: &Shape) -> Option<(f64, f64)> {
    match shape {
        Shape::Ellipse { y, ry, .. } => Some((*y, y + 2.0 * ry)),
        Shape::Rect { y, h, .. } | Shape::Empty { y, h, .. } => Some((*y, y + h)),
        Shape::Polygon { points, .. } => {
            let (min_y, max_y) = min_max_y(points)?;
            Some((min_y, max_y))
        }
        Shape::Text { y, size, .. } => {
            let h = line_h(*size);
            Some((y - h + 1.5, y + 1.5))
        }
    }
}

/// Returns min/max y of a flat polygon.
fn min_max_y(points: &[f64]) -> Option<(f64, f64)> {
    let ys: Vec<f64> = points.chunks_exact(2).map(|p| p[1]).collect();
    Some((*ys.first()?, ys.iter().copied().fold(f64::MIN, f64::max)))
}

/// Collects the unioned occupied y ranges (SlotSet) of a canvas, including
/// connector arrowheads but excluding plain connector lines.
fn collect_slots(canvas: &Canvas) -> Vec<(f64, f64)> {
    let mut ranges: Vec<(f64, f64)> = Vec::new();
    for shape in &canvas.shapes {
        if let Some(r) = shape_slot_y(shape) {
            ranges.push(r);
        }
    }
    for connector in &canvas.connectors {
        for poly in connector.arrowheads() {
            if let Some(r) = min_max_y(&poly) {
                ranges.push(r);
            }
        }
    }
    union_ranges(&mut ranges)
}

/// Sorts and unions overlapping ranges (SlotSet.addSlot semantics).
fn union_ranges(ranges: &mut Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut result: Vec<(f64, f64)> = Vec::new();
    for (s, e) in ranges.drain(..) {
        if let Some(last) = result.last_mut() {
            if s <= last.1 {
                if e > last.1 {
                    last.1 = e;
                }
                continue;
            }
        }
        result.push((s, e));
    }
    result
}

/// Builds the compression slots: occupied ranges reversed (the gaps), then
/// restricted to gaps larger than `2 * 5` with a 5-unit margin each side
/// (`SlotSet.reverse().smaller(5)`).
fn compression_slots(occupied: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut gaps: Vec<(f64, f64)> = Vec::new();
    for pair in occupied.windows(2) {
        gaps.push((pair[0].1, pair[1].0));
    }
    let mut slots: Vec<(f64, f64)> = Vec::new();
    for (s, e) in gaps {
        if e - s <= 10.0 {
            continue;
        }
        slots.push((s + 5.0, e - 5.0));
    }
    union_ranges(&mut slots)
}

/// Applies a `CompressionTransform` to a value.
fn compress_y(slots: &[(f64, f64)], v: f64) -> f64 {
    let mut delta = 0.0_f64;
    for (s, e) in slots {
        if *s > v {
            continue;
        }
        if v > *e {
            delta += e - s;
        } else {
            delta += v - s;
        }
    }
    v - delta
}

// ── LimitFinder (minmax) ─────────────────────────────────────────────────

/// Accumulates the `LimitFinder` minmax over shapes and connectors.
fn limit_minmax(canvas: &Canvas, slots: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let mut lo_x = f64::MAX;
    let mut lo_y = f64::MAX;
    let mut hi_x = f64::MIN;
    let mut hi_y = f64::MIN;

    let mut add = |x: f64, y: f64, lo_x: &mut f64, lo_y: &mut f64,
        hi_x: &mut f64, hi_y: &mut f64| {
        *lo_x = lo_x.min(x);
        *lo_y = lo_y.min(y);
        *hi_x = hi_x.max(x);
        *hi_y = hi_y.max(y);
    };

    for shape in &canvas.shapes {
        limit_shape(shape, slots, &mut add, &mut lo_x, &mut lo_y, &mut hi_x, &mut hi_y);
    }
    for connector in &canvas.connectors {
        // Plain line segments.
        for pair in connector.points.windows(2) {
            let (x1, y1) = pair[0];
            let (x2, y2) = pair[1];
            add(x1, compress_y(slots, y1), &mut lo_x, &mut lo_y, &mut hi_x, &mut hi_y);
            add(x2, compress_y(slots, y2), &mut lo_x, &mut lo_y, &mut hi_x, &mut hi_y);
        }
        // Arrowhead polygons: LimitFinder adds a ±10 x margin to UPolygons.
        for poly in connector.arrowheads() {
            if let Some((p_min_y, p_max_y)) = min_max_y(&poly) {
                let xs: Vec<f64> = poly.chunks_exact(2).map(|p| p[0]).collect();
                let min_xp = xs.iter().copied().fold(f64::MAX, f64::min);
                let max_xp = xs.iter().copied().fold(f64::MIN, f64::max);
                add(
                    min_xp - 10.0,
                    compress_y(slots, p_min_y),
                    &mut lo_x,
                    &mut lo_y,
                    &mut hi_x,
                    &mut hi_y,
                );
                add(
                    max_xp + 10.0,
                    compress_y(slots, p_max_y),
                    &mut lo_x,
                    &mut lo_y,
                    &mut hi_x,
                    &mut hi_y,
                );
            }
        }
    }

    (lo_x, lo_y, hi_x, hi_y)
}

/// Adds a shape's LimitFinder points.
#[allow(clippy::too_many_arguments)]
fn limit_shape(
    shape: &Shape,
    slots: &[(f64, f64)],
    add: &mut impl FnMut(f64, f64, &mut f64, &mut f64, &mut f64, &mut f64),
    lo_x: &mut f64,
    lo_y: &mut f64,
    hi_x: &mut f64,
    hi_y: &mut f64,
) {
    match shape {
        Shape::Ellipse { x, y, rx, ry, .. } => {
            let y = compress_y(slots, *y);
            add(*x, y, lo_x, lo_y, hi_x, hi_y);
            add(x + 2.0 * rx - 1.0, y + 2.0 * ry - 1.0, lo_x, lo_y, hi_x, hi_y);
        }
        Shape::Rect { x, y, w, h, .. } => {
            let y = compress_y(slots, *y);
            add(x - 1.0, y - 1.0, lo_x, lo_y, hi_x, hi_y);
            add(x + w - 1.0, y + h - 1.0, lo_x, lo_y, hi_x, hi_y);
        }
        Shape::Polygon { points, .. } => {
            let Some((pmin_y, pmax_y)) = min_max_y(points) else {
                return;
            };
            let xs: Vec<f64> = points.chunks_exact(2).map(|p| p[0]).collect();
            let pmin_x = xs.iter().copied().fold(f64::MAX, f64::min);
            let pmax_x = xs.iter().copied().fold(f64::MIN, f64::max);
            add(pmin_x - 10.0, compress_y(slots, pmin_y), lo_x, lo_y, hi_x, hi_y);
            add(pmax_x + 10.0, compress_y(slots, pmax_y), lo_x, lo_y, hi_x, hi_y);
        }
        Shape::Text {
            x,
            y,
            size,
            text_length,
            ..
        } => {
            let h = line_h(*size);
            // LimitFinder text: y reduced by h-1.5, with the four corners over
            // [x, x + dim.width].
            let top = compress_y(slots, *y) - h + 1.5;
            add(*x, top, lo_x, lo_y, hi_x, hi_y);
            add(*x, top + h, lo_x, lo_y, hi_x, hi_y);
            add(
                *x + text_length,
                top,
                lo_x,
                lo_y,
                hi_x,
                hi_y,
            );
            add(
                *x + text_length,
                top + h,
                lo_x,
                lo_y,
                hi_x,
                hi_y,
            );
        }
        Shape::Empty { x, y, w, h } => {
            let y = compress_y(slots, *y);
            add(*x, y, lo_x, lo_y, hi_x, hi_y);
            add(x + w, y + h, lo_x, lo_y, hi_x, hi_y);
        }
    }
}

/// Renders an activity diagram as SVG.
///
/// Ported from the ftile export of
/// `net/sourceforge/plantuml/activitydiagram3/ActivityDiagram3.getTextBlock`.
#[must_use]
pub fn render_activity_svg(source: &ActivitySource, _diagram_label: &str) -> String {
    let bounder = StringBounderSvg::new(FileFormat::Svg);
    let canvas = layout_diagram(source, &bounder);

    let occupied = collect_slots(&canvas);
    let slots = compression_slots(&occupied);

    let (min_x, min_y, max_x, max_y) = limit_minmax(&canvas, &slots);

    // Final Recentred + export margin (TitledDiagram same(10)):
    //   draw offset = -min + 5 (recentred) + 10 (margin) = -min + 15
    //   canvas dim   = (max - min) + 15 (recentred enlarge) + 20 (margin)
    let ox = -min_x + 15.0;
    let oy = -min_y + 15.0;
    let dim_w = max_x - min_x + 35.0;
    let dim_h = max_y - min_y + 35.0;

    let mut option = SvgOption::basic();
    option.set_root_attribute("data-diagram-type", "ACTIVITY");
    option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    option.set_min_dim(dim_w, dim_h);

    let mut svg = SvgGraphics::new(0, option);

    // Shapes first.
    for shape in &canvas.shapes {
        emit_shape(&mut svg, shape, &slots, ox, oy);
    }
    // Connectors (lines + arrowheads) flushed last.
    for connector in &canvas.connectors {
        emit_connector(&mut svg, connector, &slots, ox, oy);
    }

    svg.create_xml()
}

/// Emits one shape after compression and Recentred translation.
#[allow(clippy::too_many_lines)]
fn emit_shape(svg: &mut SvgGraphics, shape: &Shape, slots: &[(f64, f64)], ox: f64, oy: f64) {
    match shape {
        Shape::Ellipse {
            x,
            y,
            rx,
            ry,
            fill,
            stroke_width,
        } => {
            svg.set_fill_color(fill);
            svg.set_stroke_color(Some(CIRCLE));
            svg.set_stroke_width(*stroke_width, None);
            svg.svg_ellipse(x + rx + ox, compress_y(slots, *y) + ry + oy, *rx, *ry, 0.0);
        }
        Shape::Rect { x, y, w, h, rx } => {
            svg.set_fill_color(FILL_BOX);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(0.5, None);
            svg.svg_rectangle(x + ox, compress_y(slots, *y) + oy, *w, *h, *rx, *rx, 0.0);
        }
        Shape::Polygon { points, fill, stroke_width } => {
            svg.set_fill_color(fill);
            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(*stroke_width, None);
            let compressed: Vec<f64> = points
                .chunks_exact(2)
                .flat_map(|p| [p[0], compress_y(slots, p[1])])
                .collect();
            let shifted: Vec<f64> = compressed
                .chunks_exact(2)
                .flat_map(|p| [p[0] + ox, p[1] + oy])
                .collect();
            svg.svg_polygon(0.0, &shifted);
        }
        Shape::Text {
            text,
            x,
            y,
            size,
            text_length,
        } => {
            svg.set_fill_color(TEXT_COLOR);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            svg.text(
                text,
                x + ox,
                compress_y(slots, *y) + oy,
                Some("sans-serif"),
                *size,
                None,
                None,
                None,
                *text_length,
                &IndexMap::new(),
                None,
            );
        }
        Shape::Empty { .. } => { /* UEmpty draws nothing. */ }
    }
}

/// Emits a connector: line segments then arrowhead polygons.
fn emit_connector(
    svg: &mut SvgGraphics,
    connector: &Connector,
    slots: &[(f64, f64)],
    ox: f64,
    oy: f64,
) {
    svg.set_fill_color(STROKE);
    svg.set_stroke_color(Some(STROKE));
    svg.set_stroke_width(1.0, None);
    let mut emphasized = false;
    for pair in connector.points.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];
        // Worm.drawLine: the first segment matching the emphasized direction
        // carries a midpoint arrow, drawn before the segment's line.
        if !emphasized {
            if let Some(d) = connector.emphasize {
                if segment_dir(x1, y1, x2, y2) == Some(d) {
                    let poly = translate_poly(
                        arrow_poly(d),
                        ((x1 + x2) / 2.0, (y1 + y2) / 2.0),
                    );
                    let shifted: Vec<f64> = poly
                        .chunks_exact(2)
                        .flat_map(|p| [p[0] + ox, compress_y(slots, p[1]) + oy])
                        .collect();
                    svg.svg_polygon(0.0, &shifted);
                    emphasized = true;
                }
            }
        }
        // UGraphicCompressOnXorY.drawLine: after compression the endpoints
        // are normalized so the smaller y is first.
        let mut ly1 = compress_y(slots, y1) + oy;
        let mut ly2 = compress_y(slots, y2) + oy;
        if ly1 > ly2 {
            std::mem::swap(&mut ly1, &mut ly2);
        }
        svg.svg_line(x1 + ox, ly1, x2 + ox, ly2, 0.0);
    }
    // Terminal arrowheads (start/end), after every line.
    svg.set_fill_color(STROKE);
    svg.set_stroke_color(Some(STROKE));
    svg.set_stroke_width(1.0, None);
    for poly in connector.terminal_arrowheads() {
        let shifted: Vec<f64> = poly
            .chunks_exact(2)
            .flat_map(|p| [p[0] + ox, compress_y(slots, p[1]) + oy])
            .collect();
        svg.svg_polygon(0.0, &shifted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_flow() {
        let source = crate::parse_activity_source(&["start", ":Do something;", "stop"]);
        let svg = render_activity_svg(&source, "(Activity)");
        assert!(svg.contains("Do something"));
    }

    #[test]
    fn test_compress_identity_below_first_slot() {
        let slots = vec![(25.0, 40.0)];
        assert!((compress_y(&slots, 10.0) - 10.0).abs() < 1e-9);
        assert!((compress_y(&slots, 50.0) - 35.0).abs() < 1e-9);
    }
}

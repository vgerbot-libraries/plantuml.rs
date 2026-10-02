//! Box-list to boundary polygon, de Casteljau split and shape clipping.
//!
//! Ported from graphviz 14.1.2:
//! - `lib/common/routespl.c` (`routesplines_`, polygon assembly)
//! - `lib/common/geom.h` (`boxf`)
//! - `lib/common/utils.c` (`Bezier`)
//! - `lib/common/splines.c` (`bezier_clip`)
//! - `lib/common/shapes.c` (`poly_inside`)

#![allow(clippy::similar_names)]

use super::geom::Point;

/// Axis-aligned rectangle. Ported from: lib/common/geom.h (`boxf`).
#[derive(Clone, Copy, Debug)]
pub struct Box {
    pub ll: Point,
    pub ur: Point,
}

/// Convert an ordered, vertically contiguous box list into the boundary
/// polygon used by the shortest-path router.
///
/// Ported from: lib/common/routespl.c (`routesplines_`, polygon assembly).
pub fn boxes_to_polygon(boxes_in: &[Box]) -> Vec<Point> {
    let mut boxes: Vec<Box> = boxes_in.to_vec();
    let boxn = boxes.len();

    // Flip so box0 LL.y is not above box1.
    let flip = boxn > 1 && boxes[0].ll.y > boxes[1].ll.y;
    if flip {
        for b in &mut boxes {
            let v = b.ur.y;
            b.ur.y = -b.ll.y;
            b.ll.y = -v;
        }
    }
    let mut pp: Vec<Point> = Vec::new();

    // Forward pass over boxes.
    for bi in 0..boxn {
        let prev = if bi > 0 {
            if boxes[bi].ll.y > boxes[bi - 1].ll.y {
                -1
            } else {
                1
            }
        } else {
            0
        };
        let next = if bi + 1 < boxn {
            if boxes[bi + 1].ll.y > boxes[bi].ll.y {
                1
            } else {
                -1
            }
        } else {
            0
        };
        if prev != next {
            if next == -1 || prev == 1 {
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
            } else {
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
            }
        } else if prev == 0 {
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
        }
    }

    // Reverse pass over boxes.
    let mut bi = boxn;
    while bi > 0 {
        bi -= 1;
        let prev = if bi + 1 < boxn {
            if boxes[bi].ll.y > boxes[bi + 1].ll.y {
                -1
            } else {
                1
            }
        } else {
            0
        };
        let next = if bi > 0 {
            if boxes[bi - 1].ll.y > boxes[bi].ll.y {
                1
            } else {
                -1
            }
        } else {
            0
        };
        if prev != next {
            if next == -1 || prev == 1 {
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
            } else {
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
            }
        } else if prev == 0 {
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
        } else if prev == -1 && next == -1 {
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
        }
    }

    if flip {
        for p in &mut pp {
            p.y *= -1.0;
        }
    }
    pp
}

// ── de Casteljau split and shape clipping ──────────────────────────────────

/// Split a cubic Bezier at parameter `t`.
/// Ported from: lib/common/utils.c (`Bezier`).
fn bezier_split(v: &[Point; 4], t: f64) -> ([Point; 4], [Point; 4]) {
    let mut vt = [[Point::new(0.0, 0.0); 4]; 4];
    vt[0] = *v;
    for i in 1..=3 {
        for j in 0..=3 - i {
            vt[i][j] = Point::new(
                (1.0 - t) * vt[i - 1][j].x + t * vt[i - 1][j + 1].x,
                (1.0 - t) * vt[i - 1][j].y + t * vt[i - 1][j + 1].y,
            );
        }
    }
    let left = [vt[0][0], vt[1][0], vt[2][0], vt[3][0]];
    let right = [vt[3][0], vt[2][1], vt[1][2], vt[0][3]];
    (left, right)
}

/// Node shape for endpoint clipping, with half extents about the center.
#[derive(Clone, Copy, Debug)]
pub enum NodeShape {
    Rect { half_w: f64, half_h: f64 },
    Ellipse { half_w: f64, half_h: f64 },
}

impl NodeShape {
    /// Ported from the normalized inside test of lib/common/shapes.c
    /// (`poly_inside`). Coordinates are relative to the node center.
    fn inside(&self, p: Point) -> bool {
        match *self {
            Self::Rect { half_w, half_h } => p.x.abs() <= half_w && p.y.abs() <= half_h,
            Self::Ellipse { half_w, half_h } => {
                (p.x / half_w).hypot(p.y / half_h) < 1.0
            }
        }
    }
}

/// Endpoint of a routed edge: its center and shape.
#[derive(Clone, Copy, Debug)]
pub struct Endpoint {
    pub center: Point,
    pub shape: NodeShape,
}

/// Clip a cubic in place to the node boundary at one end.
///
/// Ported from: lib/common/splines.c (`bezier_clip`); `left_inside` means
/// curve[0] is inside the node, else curve[3] is.
pub fn bezier_clip(curve: &mut [Point; 4], end: Endpoint, left_inside: bool) {
    let rel = |p: Point| Point::new(p.x - end.center.x, p.y - end.center.y);
    let is_inside = |p: Point| end.shape.inside(rel(p));

    let mut low = 0.0f64;
    let mut high = 1.0f64;
    let mut pt = if left_inside { curve[0] } else { curve[3] };
    let mut best = *curve;
    let mut found = false;
    loop {
        let opt = pt;
        let t = (high + low) / 2.0;
        let (left, right) = bezier_split(curve, t);
        let seg = if left_inside { &right } else { &left };
        pt = if left_inside { right[0] } else { left[3] };
        if is_inside(pt) {
            if left_inside {
                low = t;
            } else {
                high = t;
            }
            best = *seg;
            found = true;
        } else if left_inside {
            high = t;
        } else {
            low = t;
        }
        if (opt.x - pt.x).abs() <= 0.5 && (opt.y - pt.y).abs() <= 0.5 {
            break;
        }
    }
    *curve = if found {
        best
    } else {
        bezier_split(curve, (high + low) / 2.0).1
    };
}

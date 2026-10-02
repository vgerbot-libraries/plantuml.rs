//! Pure-Rust internal-frame layout solver for JSON/YAML record trees.
//!
//! Reproduces the subset of graphviz dot used by
//! `net/sourceforge/plantuml/jsondiagram/SmetanaForJson`:
//! longest-path ranking, `set_ycoords` (rank gap 36), bottom-up x placement
//! with record port offsets, then the box-corridor spline routing with a
//! node-boundary clip followed by the normal-arrow length clip.
//!
//! Geometry is computed in the dot internal frame (y grows upward). The
//! caller mirrors it into final SVG coordinates.

#![allow(clippy::similar_names)]

use super::json_renderer::Record;
use crate::pathplan::{
    bezier_clip, boxes_to_polygon, route_spline, shortest_path, Box as GvBox, Edge as GvEdge,
    Endpoint, NodeShape, Point,
};

/// Horizontal node separation (dot default `nodesep`, 0.25").
const NODESEP: f64 = 18.0;
/// Vertical rank separation (dot default `ranksep`, 0.5").
const RANKSEP: f64 = 36.0;
/// Extra bounding-box slack (dotsplines.c `FUDGE`).
const FUDGE: f64 = 4.0;
/// Minimum corridor width added on each side (dotsplines.c `MINW`).
const MINW: f64 = 16.0;

/// Length of the normal arrow head (`10 * lenfact(1) * arrowsize(.75)`).
const ARROW_LEN: f64 = 7.5;

/// Graphviz `ROUND(f)`: nearest integer, ties away from zero.
fn gv_round(f: f64) -> i64 {
    if f >= 0.0 {
        (f + 0.5) as i64
    } else {
        (f - 0.5) as i64
    }
}

/// Internal-frame position of a record.
pub struct NodePos {
    pub x: f64,
    pub y: f64,
}

/// A fully routed edge in the internal frame.
pub struct EdgeRoute {
    /// Cubic control points P0..=P3 (P3 already arrow-clipped).
    pub points: [Point; 4],
    /// Arrow tip on the head boundary; `None` if no arrow.
    pub ep: Option<Point>,
}

/// A laid-out graph: node centers and edge routes in the internal frame.
pub struct Layout {
    pub nodes: Vec<NodePos>,
    pub edges: Vec<EdgeRoute>,
    /// Value of dot's `max`, used by the final mirror.
    pub max: f64,
}

/// Internal solver node.
struct Node {
    center: Point,
    half_w: f64,
    half_h: f64,
    rank: usize,
    order: usize,
}

/// One real edge: (tail, head, tail row index).
struct RealEdge {
    tail: usize,
    head: usize,
    row: usize,
}

/// X offset of a record's tail port for `row`, `ROUND((row-(n-1)/2)*pitch)`.
fn port_x(row: usize, rows: usize) -> f64 {
    let center = (rows as f64 - 1.0) / 2.0;
    gv_round((row as f64 - center) * super::json_renderer::ROW_PITCH) as f64
}

/// Depth-first post-order edge walk: each child subtree precedes the edge
/// joining it to `tail`, matching Java's recurse-then-`createEdge` order.
fn collect_edges(tail: usize, records: &[Record], out: &mut Vec<RealEdge>) {
    for &(row, child) in &records[tail].children {
        collect_edges(child, records, out);
        out.push(RealEdge { tail, head: child, row });
    }
}
/// Run the internal-frame solver over `records`, rooted at `root_idx`.
pub(crate) fn solve(records: &[Record], root_idx: usize) -> Layout {
    let n = records.len();

    // Internal box dims: box width = drawn record height; box height =
    // ROUND(drawn record width) + 1.
    let box_dims: Vec<(f64, f64)> = records
        .iter()
        .map(|r| (r.height, gv_round(r.width) as f64 + 1.0))
        .collect();

    // Real edges collected in post-order: the Java `manageOneNode` adds an
    // edge only AFTER recursively creating the child node, so deep edges are
    // emitted before their parent's edge.
    let mut real_edges: Vec<RealEdge> = Vec::new();
    collect_edges(root_idx, records, &mut real_edges);

    let mut nodes: Vec<Node> = box_dims
        .iter()
        .map(|&(w, h)| Node {
            center: Point::new(0.0, 0.0),
            half_w: w / 2.0,
            half_h: h / 2.0,
            rank: 0,
            order: 0,
        })
        .collect();

    // ── Ranking: tree depth via a traversal from the root ────────────────
    // Relaxing ranks over post-order edges is unsound: a grandchild edge is
    // visited before its parent node received its own rank. Assign ranks by
    // a pre-order walk so depth is independent of edge emission order.
    let mut child_of: Vec<Vec<usize>> = vec![Vec::new(); n];
    for e in &real_edges {
        child_of[e.tail].push(e.head);
    }
    let mut stack = vec![root_idx];
    while let Some(t) = stack.pop() {
        for &c in &child_of[t] {
            nodes[c].rank = nodes[t].rank + 1;
            stack.push(c);
        }
    }
    let max_rank = nodes.iter().map(|x| x.rank).max().unwrap_or(0);

    let mut members: Vec<Vec<usize>> = vec![Vec::new(); max_rank + 1];
    for (i, node) in nodes.iter().enumerate() {
        members[node.rank].push(i);
    }
    for r in &members {
        for (i, &idx) in r.iter().enumerate() {
            nodes[idx].order = i;
        }
    }

    // Per-rank half heights.
    let rank_ht: Vec<f64> = members
        .iter()
        .map(|r| r.iter().map(|&i| nodes[i].half_h).fold(0.0_f64, f64::max))
        .collect();

    // ── y coordinates (set_ycoords) ─────────────────────────────────────
    let mut rank_y = vec![0.0f64; max_rank + 1];
    rank_y[max_rank] = rank_ht[max_rank];
    for r in (0..max_rank).rev() {
        rank_y[r] = rank_y[r + 1] + rank_ht[r + 1] + rank_ht[r] + RANKSEP;
    }
    for node in &mut nodes {
        node.center.y = rank_y[node.rank];
    }

    // ── x coordinates (bottom-up barycenter over tail port offsets) ─────
    let mut children: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for e in &real_edges {
        children[e.tail].push((e.head, e.row));
    }

    let mut center_x = vec![0.0f64; n];
    for r in (0..=max_rank).rev() {
        let mut prev: Option<usize> = None;
        for &idx in &members[r] {
            let mut c = if children[idx].is_empty() {
                nodes[idx].half_w
            } else {
                let rows = records[idx].row_count();
                let sum: f64 = children[idx]
                    .iter()
                    .map(|&(child, row)| center_x[child] - port_x(row, rows))
                    .sum();
                sum / children[idx].len() as f64
            };
            if let Some(p) = prev {
                let minlen =
                    (nodes[p].half_w + NODESEP + nodes[idx].half_w).round();
                c = c.max(center_x[p] + minlen);
            }
            center_x[idx] = c;
            prev = Some(idx);
        }
    }
    let min_left = (0..n)
        .map(|i| center_x[i] - nodes[i].half_w)
        .fold(f64::INFINITY, f64::min);
    for (i, node) in nodes.iter_mut().enumerate() {
        node.center.x = center_x[i] - min_left;
    }

    // ── Routing bounds ──────────────────────────────────────────────────
    let mut left_bound = f64::INFINITY;
    let mut right_bound = f64::INFINITY;
    for r in &members {
        let first = &nodes[r[0]];
        let last = &nodes[r[r.len() - 1]];
        left_bound = left_bound.min(first.center.x - first.half_w);
        right_bound = right_bound.max(last.center.x + last.half_w);
    }
    if n == 0 {
        left_bound = 0.0;
        right_bound = 0.0;
    }
    left_bound -= MINW;
    right_bound += MINW;

    // dot's `max`: largest node top in the internal frame.
    let max = (0..n)
        .map(|i| nodes[i].center.y + nodes[i].half_h)
        .fold(0.0_f64, f64::max);

    // ── Route edges ─────────────────────────────────────────────────────
    let mut edges = Vec::with_capacity(real_edges.len());
    for e in &real_edges {
        edges.push(route_edge(
            e,
            &nodes,
            &members,
            &rank_ht,
            records,
            left_bound,
            right_bound,
        ));
    }

    Layout {
        nodes: nodes.into_iter().map(|x| NodePos { x: x.center.x, y: x.center.y }).collect(),
        edges,
        max,
    }
}

/// maximal_bbox x-range for a node, bounded by its same-rank neighbours.
fn node_xspan(
    idx: usize,
    nodes: &[Node],
    members: &[Vec<usize>],
    left_bound: f64,
    right_bound: f64,
) -> (f64, f64) {
    let node = &nodes[idx];
    let order = node.order;
    let r = &members[node.rank];

    let mut b = node.center.x - node.half_w - FUDGE;
    let llx = if order > 0 {
        let left = &nodes[r[order - 1]];
        b = b.min(left.center.x + left.half_w + NODESEP / 2.0);
        b.round()
    } else {
        b.round().min(left_bound)
    };

    let mut b = node.center.x + node.half_w + FUDGE;
    let urx = if order + 1 < r.len() {
        let right = &nodes[r[order + 1]];
        b = b.max(right.center.x - right.half_w - NODESEP / 2.0);
        b.round()
    } else {
        b.round().max(right_bound)
    };
    (llx, urx)
}

/// Route one edge through the corridor and apply the head clips.
#[allow(clippy::too_many_arguments)]
fn route_edge(
    e: &RealEdge,
    nodes: &[Node],
    members: &[Vec<usize>],
    rank_ht: &[f64],
    records: &[Record],
    left_bound: f64,
    right_bound: f64,
) -> EdgeRoute {
    let tail = &nodes[e.tail];
    let head = &nodes[e.head];
    let rt = tail.rank;
    let rh = head.rank;

    let (t_ll, t_ur) = node_xspan(e.tail, nodes, members, left_bound, right_bound);
    let (h_ll, h_ur) = node_xspan(e.head, nodes, members, left_bound, right_bound);

    let rows = records[e.tail].row_count();
    let tport_x = port_x(e.row, rows);

    // Tail corridor: bottom half of the tail box.
    let tail_box = GvBox {
        ll: Point::new(t_ll, tail.center.y - rank_ht[rt]),
        ur: Point::new(t_ur, tail.center.y),
    };
    // Start at the tail port anchor on the node boundary.
    let start = Point::new(tail.center.x + tport_x, tail.center.y - tail.half_h);

    // Inter-rank corridor.
    let inter_box = GvBox {
        ll: Point::new(left_bound, head.center.y + rank_ht[rh]),
        ur: Point::new(right_bound, tail.center.y - rank_ht[rt]),
    };

    // Head corridor: top half, down to the head center line.
    let head_box = GvBox {
        ll: Point::new(h_ll, head.center.y),
        ur: Point::new(h_ur, head.center.y + rank_ht[rh]),
    };
    // +1 unit beyond the head center toward the tail: the standard inside
    // epsilon for the default port (matches dot's start.p offsets).
    let away = if tail.center.y > head.center.y { 1.0 } else { -1.0 };
    let end = Point::new(head.center.x, head.center.y + away);

    let boxes: Vec<GvBox> = [tail_box, inter_box, head_box]
        .into_iter()
        .filter(|b| b.ll.x < b.ur.x && b.ll.y < b.ur.y)
        .collect();
    let polygon = boxes_to_polygon(&boxes);
    let pl = shortest_path(&polygon, start, end);
    let barriers: Vec<GvEdge> = polygon
        .iter()
        .enumerate()
        .map(|(k, &a)| GvEdge { a, b: polygon[(k + 1) % polygon.len()] })
        .collect();

    // The tail port is always tangent to the rank direction: a vertical
    // `(0,-1)` in the internal frame, regardless of where the head sits. The
    // lateral displacement is carried by the shortest-path waypoints, not the
    // initial tangent.
    let ev0 = Point::new(0.0, -1.0);
    let ev1 = Point::new(0.0, 0.0);
    let fitted = route_spline(&barriers, &pl, ev0, ev1);

    // Reduce to the final 4 control points (first, second, penultimate, last).
    let mut curve = if fitted.len() >= 4 {
        let m = fitted.len();
        [fitted[0], fitted[1], fitted[m - 2], fitted[m - 1]]
    } else {
        // Degenerate fit: synthesize a straight segment.
        let p0 = start;
        let p3 = end;
        let mid = Point::new((p0.x + p3.x) / 2.0, (p0.y + p3.y) / 2.0);
        [p0, mid, mid, p3]
    };
    // Head shape clip: the default Center port clips to the parsed record
    // OUTER FIELD box `fld0.b` (shapes__c.record_inside), not the node box:

    // its half-height is 0.5 inward from the node half-height, half-width
    // equal to the node half-width. The binary-search stop tolerance leaves
    // the tip ~0.37 beyond the geometric boundary.
    let head_ep = Endpoint {
        center: head.center,
        shape: NodeShape::Rect {
            half_w: head.half_w,
            half_h: head.half_h - 0.5,
        },
    };

    bezier_clip(&mut curve, head_ep, false);
    let ep_point = curve[3];

    // arrowEndClip (arrows.c): build a reversed endpoint cubic, clip it to a
    // circle of radius ARROW_LEN centered on the contact point with the far
    // end inside, then reverse back.
    let mut rev = [ep_point, curve[2], curve[1], curve[0]];
    let arrow_ep = Endpoint {
        center: ep_point,
        shape: NodeShape::Ellipse {
            half_w: ARROW_LEN,
            half_h: ARROW_LEN,
        },
    };
    bezier_clip(&mut rev, arrow_ep, true);
    curve = [rev[3], rev[2], rev[1], rev[0]];

    EdgeRoute { points: curve, ep: Some(ep_point) }
}

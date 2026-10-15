//! Pure-Rust edge routing for diagrams that carry an edge label.
//!
//! When any edge has a label graphviz (`lib/dotgen/rank.c::edgelabel_ranks`)
//! doubles every edge's `minlen`, inserts a virtual node per edge in the new
//! intermediate rank, and halves the integer rank separation. The virtual
//! nodes are laid out left-to-right on odd ranks (`lib/dotgen/position.c
//! ::make_LR_constraints`) with a reduced 5px separation, then each edge is
//! routed independently through the notched box corridor (`lib/dotgen
//! /dotsplines.c`, `lib/common/routespl.c`).
//!
//! Geometry is computed in the dot internal frame: the node column is x = 0
//! and y grows upward. [`solve_labeled`] returns the same
//! [`DotSvgResult`] shape the external-`dot` parser produces, so the consumer
//! applies one uniform move-delta translation.

use std::collections::HashMap;

use crate::cuca_layout::{DotEdgePath, DotNodePos, DotSvgResult, EntityData};
use crate::entity_link_parser::ParsedLink;
use crate::native_layout::{acyclic_edges, dot_roundtrip_px};
use crate::pathplan::{
    Box as GvBox, Edge as GvEdge, Endpoint, NodeShape, Point, bezier_clip, boxes_to_polygon,
    route_spline, shortest_path,
};

/// Extra bounding-box slack (`dotsplines.c::FUDGE`).
const FUDGE: f64 = 4.0;
/// Minimum corridor width added on the bounds (`dotsplines.c::MINW`).
const MINW: f64 = 16.0;
/// Half the default node penwidth: the inside-region is the node box expanded
/// by this much (`lib/common/shapes.c`).
const PEN_HALF: f64 = 0.5;
/// Fixed half-height of an edge-label table (13px label + margins).
const LABEL_HALF_H: f64 = 9.5;
/// Half-width of the plain virtual node inserted for an unlabeled edge whose
/// doubled length spans two ranks: `1 + nodesep/2` (computed at call).
/// Half-height of that plain virtual node.
const PLAIN_HALF: f64 = 0.5;
/// Font size of an edge label.
const LABEL_FONT_SIZE: i32 = 13;
/// Width added each side of the measured label text inside the label table.
const LABEL_PAD: f64 = 1.0;
/// A placed node in the internal frame.
struct Node {
    center: Point,
    half_w: f64,
    half_h: f64,
    is_ellipse: bool,
    /// Longest-path rank over the acyclic forward graph.
    orig_rank: usize,
    /// Doubled rank.
    rank: usize,
}

/// A virtual node on an odd (label) rank.
struct LabelVnode {
    /// Nominal x after LR centering.
    x: f64,
    /// Left width (always `nodesep`).
    lw: f64,
    /// Right width = label table width.
    rw: f64,
}

/// An edge between two known entities.
struct EdgeLink {
    tail: usize,
    head: usize,
    label: Option<String>,
}

/// Label table width exactly as `SvekEdge.appendTable` receives it: the 13px
/// text width plus a 1px pad each side, truncated to an integer.
fn label_table_width(text: &str) -> f64 {
    (crate::cuca_layout::measure_text_size_pub(text, LABEL_FONT_SIZE) + 2.0 * LABEL_PAD).floor()
}

/// Clip endpoint for `node`: its ellipse or rectangle outline expanded by
/// half a pen.
fn node_endpoint(node: &Node) -> Endpoint {
    Endpoint {
        center: node.center,
        shape: if node.is_ellipse {
            NodeShape::Ellipse {
                half_w: node.half_w + PEN_HALF,
                half_h: node.half_h + PEN_HALF,
            }
        } else {
            NodeShape::Rect {
                half_w: node.half_w + PEN_HALF,
                half_h: node.half_h + PEN_HALF,
            }
        },
    }
}

/// x-extent of the label-vnode corridor box, mirroring
/// `dotsplines.c::maximal_bbox` for a VIRTUAL node carrying a label.
/// Returns `(LL.x, UR.x)` before the post-route `limitBoxes` shrinkage.
fn maximal_label_box(
    vnodes: &[&LabelVnode],
    i: usize,
    left_bound: f64,
    right_bound: f64,
    splinesep: f64,
) -> (f64, f64) {
    let vn = vnodes[i];

    let b = vn.x - vn.lw - FUDGE;
    let llx = if i > 0 {
        let left = vnodes[i - 1];
        b.min(left.x + left.rw + splinesep).round()
    } else {
        b.round().min(left_bound)
    };

    // A virtual node with a label starts its own right reach at coord + 10.
    let mut b = vn.x + 10.0;
    let mut urx = if i + 1 < vnodes.len() {
        let right = vnodes[i + 1];
        b = b.max(right.x - right.lw - splinesep);
        b.round()
    } else {
        b.round().max(right_bound)
    };
    // Leave room for the node's own label: pull the right edge in by rw.
    urx -= vn.rw;
    if urx < llx {
        urx = vn.x;
    }
    (llx, urx)
}

/// Max x reached by a spline inside a y-band.
///
/// Graphviz positions the label virtual node through the network-simplex x
/// constraints, then routes; on the singleton-rank chains this solver handles,
/// the resulting vnode coordinate equals the routed spline's geometric x
/// extent across the label band. Recover it directly from the fitted cubic
/// with a fine de Casteljau subdivision. A coarse fixed grid (the
/// `routesplines.c::limitBoxes` 10*boxn sampling) undershoots the true
/// extremum by ~0.005 where the spline runs tangent to the band edge, which
/// quantizes the anchor one cent wrong (back edge "switch": 1.045 at the
/// extremum → 1.05, vs 1.044997 on the coarse grid → 1.04).
fn spline_max_x_in_band(points: &[Point], y_lo: f64, y_hi: f64) -> f64 {
    const F: f64 = 0.0001;
    const SUBDIV: usize = 2000;
    let mut max_x = f64::NEG_INFINITY;

    for seg in points.windows(4).step_by(3) {
        let (p0, p1, p2, p3) = (seg[0], seg[1], seg[2], seg[3]);
        for si in 0..=SUBDIV {
            let t = si as f64 / SUBDIV as f64;
            // de Casteljau down to one sampled point.
            let s0 = p0.add(p1.sub(p0).scale(t));
            let s1 = p1.add(p2.sub(p1).scale(t));
            let s2 = p2.add(p3.sub(p2).scale(t));
            let q0 = s0.add(s1.sub(s0).scale(t));
            let q1 = s1.add(s2.sub(s1).scale(t));
            let q = q0.add(q1.sub(q0).scale(t));
            if q.y <= y_hi + F && q.y >= y_lo - F {
                max_x = max_x.max(q.x);
            }
        }
    }
    max_x
}

/// Left edge of an edge-label table, exactly as graphviz 14 derives it.
///
/// Ported from `lib/common/routespl.c::limitBoxes` followed by
/// `lib/dotgen/dotsplines.c::recover_slack`. After routing, graphviz shrinks
/// the label rank's corridor box by coarse de Casteljau sampling of the
/// *straight* centre-to-centre cubic `[P0, P0, P3, P3]` with
/// `num_div = 10 * boxn` subdivisions and recording the greatest sample `x`
/// that falls in the box's y-band; `recover_slack` then sets the labelled
/// virtual node's x to that right edge. For a `minlen = 1` (doubled-rank)
/// chain there are five rank boxes, hence `num_div = 50`.
///
/// The routed/clipped spline is deliberately *not* used: graphviz samples the
/// straight cubic here, which is what keeps a diagonal label anchored on the
/// column side rather than at the curved edge's furthest excursion.
///
/// `start` and `end` are the one-pixel-inset path terminals in the internal
/// y-up frame (`tail.y - 1` and `head.y + 1`); `vn_y` is the label virtual
/// node's rank coordinate.
pub fn label_box_left_straight(start: Point, end: Point, vn_y: f64) -> f64 {
    const NUM_DIV: usize = 50;
    const F: f64 = 0.0001;

    let y_lo = vn_y - LABEL_HALF_H;
    let y_hi = vn_y + LABEL_HALF_H;

    // Control polygon [P0, P0, P3, P3]. The in-place forward substitution
    // mirrors `limitBoxes` exactly so rounding is bit-for-bit identical.
    let (x0, y0) = (start.x, start.y);
    let (x1, y1) = (start.x, start.y);
    let (x2, y2) = (end.x, end.y);
    let (x3, y3) = (end.x, end.y);

    let mut right = f64::NEG_INFINITY;
    for si in 0..=NUM_DIV {
        let t = si as f64 / NUM_DIV as f64;
        let (mut ax, mut ay) = (x0, y0);
        let (mut bx, mut by) = (x1, y1);
        let (mut cx, mut cy) = (x2, y2);
        ax += t * (bx - ax);
        ay += t * (by - ay);
        bx += t * (cx - bx);
        by += t * (cy - by);
        cx += t * (x3 - cx);
        cy += t * (y3 - cy);
        ax += t * (bx - ax);
        ay += t * (by - ay);
        bx += t * (cx - bx);
        by += t * (cy - by);
        ax += t * (bx - ax);
        ay += t * (by - ay);
        if ay <= y_hi + F && ay >= y_lo - F {
            right = right.max(ax);
        }
    }
    right
}

/// Runs the labeled-edge layout solver.
///
/// `sorted` is source-line ordered `(name, svek extents)` of leaves only.
/// `links` is the full parsed link list. Geometry is returned in the internal
/// frame (node column x = 0, y upward); edges carry every fitted point so the
/// consumer emits multi-segment cubics.
pub fn solve_labeled(
    sorted: &[(&String, &EntityData)],
    links: &[ParsedLink],
    nodesep: f64,
    base_ranksep: f64,
) -> DotSvgResult {
    // Graphviz derives the inter-node routing geometry from the integer
    // `nodesep`: spline clearance = nodesep/4, a label vnode's left width is
    // the whole nodesep, and a plain vnode half-width is 1 + nodesep/2.
    let splinesep = (nodesep / 4.0).floor();
    let plain_lw = 1.0 + (nodesep / 2.0).floor();
    let names: Vec<&String> = sorted.iter().map(|(n, _)| *n).collect();
    let index: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // ── Node dimensions ──────────────────────────────────────────────────
    let mut nodes: Vec<Node> = sorted
        .iter()
        .map(|(_, ed)| {
            let is_ellipse =
                (2.0 * ed.rx - ed.svek_w).abs() < 0.5 && (2.0 * ed.ry - ed.svek_h).abs() < 0.5;
            Node {
                center: Point::new(0.0, 0.0),
                half_w: dot_roundtrip_px(ed.svek_w) / 2.0,
                half_h: dot_roundtrip_px(ed.svek_h) / 2.0,
                is_ellipse,
                orig_rank: 0,
                rank: 0,
            }
        })
        .collect();

    // ── Edges between known entities, in link order ──────────────────────
    let edges: Vec<EdgeLink> = links
        .iter()
        .filter_map(|l| {
            let tail = index.get(l.from.as_str()).copied()?;
            let head = index.get(l.to.as_str()).copied()?;
            Some(EdgeLink {
                tail,
                head,
                label: l.label.clone(),
            })
        })
        .collect();

    // ── Longest-path ranks over the acyclic forward graph ────────────────
    // Reuse the three-color feedback-edge removal native_layout uses for the
    // unlabeled solver so back edges never contribute a rank.
    let edge_tuples: Vec<(&String, &String)> = edges
        .iter()
        .map(|e| (names[e.tail], names[e.head]))
        .collect();
    let kept_tuples = acyclic_edges(&names, &edge_tuples);
    let kept: std::collections::HashSet<(usize, usize)> = kept_tuples
        .iter()
        .map(|(a, b)| (index[a.as_str()], index[b.as_str()]))
        .collect();

    let mut changed = true;
    while changed {
        changed = false;
        for (ei, e) in edges.iter().enumerate() {
            if kept.contains(&(e.tail, e.head)) {
                let nr = nodes[e.tail].orig_rank + 1;
                if nr > nodes[e.head].orig_rank {
                    nodes[e.head].orig_rank = nr;
                    changed = true;
                }
            }
            let _ = ei;
        }
    }
    for n in &mut nodes {
        n.rank = n.orig_rank * 2;
    }
    let max_doubled = nodes.iter().map(|n| n.rank).max().unwrap_or(0);

    // ── One label vnode per labeled edge, grouped by doubled odd rank ─────
    // vnodes follow link (source) order within a rank.
    let mut vnodes_by_rank: HashMap<usize, Vec<LabelVnode>> = HashMap::new();
    let mut edge_vnode: HashMap<usize, usize> = HashMap::new();
    for (ei, e) in edges.iter().enumerate() {
        let Some(text) = e.label.as_ref() else { continue };
        let rank = nodes[e.tail].rank.max(nodes[e.head].rank) - 1;
        let entry = vnodes_by_rank.entry(rank).or_default();
        edge_vnode.insert(ei, entry.len());
        entry.push(LabelVnode {
            x: 0.0,
            lw: nodesep,
            rw: label_table_width(text),
        });
    }

    // Center each label rank across the node column: consecutive vnodes are
    // `rw(prev) + lw(next) + 5` apart.
    for vnodes in vnodes_by_rank.values_mut() {
        let gaps: Vec<f64> = vnodes
            .windows(2)
            .map(|w| w[0].rw + w[1].lw + 5.0)
            .collect();
        let total: f64 = gaps.iter().sum();
        let mut cursor = -total / 2.0;
        for (i, vn) in vnodes.iter_mut().enumerate() {
            vn.x = cursor;
            if i < gaps.len() {
                cursor += gaps[i];
            }
        }
    }

    // ── Half-height of each doubled rank ─────────────────────────────────
    let mut half = vec![0.0f64; max_doubled + 1];
    for n in &nodes {
        half[n.rank] = half[n.rank].max(n.half_h);
    }
    for &r in vnodes_by_rank.keys() {
        half[r] = half[r].max(LABEL_HALF_H);
    }
    for r in (0..max_doubled).step_by(2) {
        let odd = r + 1;
        if odd <= max_doubled && !vnodes_by_rank.contains_key(&odd) {
            half[odd] = half[odd].max(PLAIN_HALF);
        }
    }

    // ── y coordinates ────────────────────────────────────────────────────
    let mut rank_y = vec![0.0f64; max_doubled + 1];
    rank_y[max_doubled] = half[max_doubled];
    for r in (0..max_doubled).rev() {
        rank_y[r] = rank_y[r + 1] + half[r] + half[r + 1] + base_ranksep / 2.0;
    }
    for n in &mut nodes {
        n.center.y = rank_y[n.rank];
    }

    // ── Routing bounds (MINW accumulates per rank) ───────────────────────
    let mut left_bound = 0.0f64;
    let mut right_bound = 0.0f64;
    for r in 0..=max_doubled {
        let (rl, rr) = if let Some(vnodes) = vnodes_by_rank.get(&r) {
            (
                vnodes[0].x - vnodes[0].lw,
                vnodes[vnodes.len() - 1].x + vnodes[vnodes.len() - 1].rw,
            )
        } else if r % 2 == 1 {
            (-plain_lw, plain_lw)
        } else {
            let hw = nodes
                .iter()
                .filter(|n| n.rank == r)
                .map(|n| n.half_w)
                .fold(0.0_f64, f64::max);
            (-hw, hw)
        };
        left_bound = left_bound.min(rl);
        right_bound = right_bound.max(rr);
        left_bound -= MINW;
        right_bound += MINW;
    }

    let q2 = |v: f64| (v * 100.0).round() / 100.0;

    // ── Node boxes ───────────────────────────────────────────────────────
    // Graphviz centre-aligns every node of a TB column on one axis placed at
    // the widest node's half-width. It rounds each corner in graph
    // coordinates against that shared centre, then translates by the rounded
    // centre; a narrower node therefore quantizes to a value an independent
    // per-node rounding would miss (Paused: 3.67 − 37.77 = −34.10).
    let column_center = nodes.iter().map(|n| n.half_w).fold(0.0_f64, f64::max);
    let mut out_nodes = HashMap::new();
    for (idx, node) in nodes.iter().enumerate() {
        let min_x = q2(column_center - node.half_w) - q2(column_center);
        out_nodes.insert(
            (idx + 6) as i32,
            DotNodePos {
                min_x,
                min_y: q2(-(node.center.y + node.half_h)),
            },
        );
    }

    // ── Route each edge, recover its label anchor ────────────────────────
    let mut out_edges = HashMap::new();
    let mut label_vnode_y = HashMap::new();
    let mut label_box_left = HashMap::new();
    let first_color = (nodes.len() + 6) as i32;
    // Route order mirrors graphviz's forward-converted edge sort: treat a back
    // edge as running from its geometrically upper node, order by that upper
    // doubled rank ascending, the forward edge before its feedback twin.
    let mut route_order: Vec<usize> = (0..edges.len()).collect();
    route_order.sort_by(|&a, &b| {
        let key = |ei: usize| -> (usize, u8) {
            let e = &edges[ei];
            let rt = nodes[e.tail].rank;
            let rh = nodes[e.head].rank;
            (rt.max(rh), u8::from(rt > rh))
        };
        key(a).cmp(&key(b))
    });
    // Edge colors in that order for the consumer's draw-order channel.
    let draw_order: Vec<i32> = route_order
        .iter()
        .map(|&ei| first_color + ei as i32)
        .collect();

    for &ei in &route_order {
        let e = &edges[ei];
        let is_back = nodes[e.tail].rank > nodes[e.head].rank;
        let top = nodes[e.tail].rank.max(nodes[e.head].rank);
        let mid_rank = top - 1;
        let (upper, lower) = if is_back {
            (&nodes[e.head], &nodes[e.tail])
        } else {
            (&nodes[e.tail], &nodes[e.head])
        };

        let upper_box = GvBox {
            ll: Point::new(left_bound, upper.center.y - upper.half_h),
            ur: Point::new(right_bound, upper.center.y),
        };
        let upper_inter = GvBox {
            ll: Point::new(left_bound, rank_y[mid_rank] + half[mid_rank]),
            ur: Point::new(right_bound, upper.center.y - upper.half_h),
        };
        let mid_box = if let Some(&vi) = edge_vnode.get(&ei) {
            let rank_vnodes = vnodes_by_rank.get(&mid_rank).expect("label rank");
            let refs: Vec<&LabelVnode> = rank_vnodes.iter().collect();
            let (llx, urx) = maximal_label_box(&refs, vi, left_bound, right_bound, splinesep);
            GvBox {
                ll: Point::new(llx, rank_y[mid_rank] - LABEL_HALF_H),
                ur: Point::new(urx, rank_y[mid_rank] + LABEL_HALF_H),
            }
        } else {
            GvBox {
                ll: Point::new(left_bound, rank_y[mid_rank] - PLAIN_HALF),
                ur: Point::new(right_bound, rank_y[mid_rank] + PLAIN_HALF),
            }
        };
        let lower_inter = GvBox {
            ll: Point::new(left_bound, lower.center.y + lower.half_h),
            ur: Point::new(right_bound, rank_y[mid_rank] - half[mid_rank]),
        };
        let lower_box = GvBox {
            ll: Point::new(left_bound, lower.center.y),
            ur: Point::new(right_bound, lower.center.y + lower.half_h),
        };

        let boxes: Vec<GvBox> = [upper_box, upper_inter, mid_box, lower_inter, lower_box]
            .into_iter()
            .filter(|b| b.ll.x < b.ur.x && b.ll.y < b.ur.y)
            .collect();
        let polygon = boxes_to_polygon(&boxes);
        let start = Point::new(0.0, upper.center.y - 1.0);
        let end = Point::new(0.0, lower.center.y + 1.0);
        let pl = shortest_path(&polygon, start, end);
        let barriers: Vec<GvEdge> = polygon
            .iter()
            .enumerate()
            .map(|(k, &a)| GvEdge {
                a,
                b: polygon[(k + 1) % polygon.len()],
            })
            .collect();

        let free = Point::new(0.0, 0.0);
        let mut fitted = route_spline(&barriers, &pl, free, free);
        if e.label.is_some() {
            // The label box's left edge is the `UR.x` of the tight path box:
            // graphviz `limitBoxes` samples the fitted spline on a fixed grid
            // and tightens each corridor box to the sampled points. Reading
            // raw path vertices or evaluating on a different grid misses the
            // ~0.045 overshoot, so mirror the sampler exactly.
            let raw_anchor = spline_max_x_in_band(
                &fitted,
                rank_y[mid_rank] - LABEL_HALF_H,
                rank_y[mid_rank] + LABEL_HALF_H,
            );
            // `limitBoxes` tightens the corridor box in graph coordinates,
            // then the output is read back relative to the rounded column
            // axis; round the sampled extent through that frame so a sub-0.01
            // overshoot quantizes correctly (resume: 47.22 − 37.77 = 9.45).
            let anchor = q2(column_center + raw_anchor) - q2(column_center);
            label_box_left.insert(first_color + ei as i32, anchor);
            label_vnode_y.insert(first_color + ei as i32, rank_y[mid_rank]);
        }

        // Clip the terminal cubic segments at each node. The fitted spline
        // begins and ends inside the node boxes.
        if fitted.len() >= 4 {
            let n = fitted.len();
            let mut head_seg = [fitted[0], fitted[1], fitted[2], fitted[3]];
            bezier_clip(&mut head_seg, node_endpoint(upper), true);
            fitted[0] = head_seg[0];
            fitted[1] = head_seg[1];
            fitted[2] = head_seg[2];

            let mut tail_seg = [fitted[n - 4], fitted[n - 3], fitted[n - 2], fitted[n - 1]];
            bezier_clip(&mut tail_seg, node_endpoint(lower), false);
            fitted[n - 3] = tail_seg[1];
            fitted[n - 2] = tail_seg[2];
            fitted[n - 1] = tail_seg[3];
        }

        // Stored tail->head; for a back edge the spline was routed
        // upper->lower, so reverse into parsed-tail order.
        // Graphviz quantizes each edge point's x in graph coordinates
        // against the shared column axis and translates back, exactly like
        // node corners, so route the x component through that frame rather
        // than rounding the raw value (end x: 16.85 − 37.77 = −20.92).
        let qx = |v: f64| q2(column_center + v) - q2(column_center);
        let points: Vec<(f64, f64)> = if is_back {
            fitted
                .iter()
                .rev()
                .map(|p| (qx(p.x), q2(-p.y)))
                .collect()
        } else {
            fitted.iter().map(|p| (qx(p.x), q2(-p.y))).collect()
        };
        out_edges.insert(first_color + ei as i32, DotEdgePath { points });
    }

    let raw_yup: HashMap<String, (f64, f64)> = names
        .iter()
        .zip(nodes.iter())
        .map(|(name, n)| ((**name).clone(), (n.center.y, n.half_h)))
        .collect();

    DotSvgResult {
        nodes: out_nodes,
        edges: out_edges,
        label_vnode_y,
        raw_yup,
        label_box_left,
        draw_order,
    }
}

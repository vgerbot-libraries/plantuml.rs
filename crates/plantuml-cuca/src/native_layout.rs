//! Pure-Rust layout solver replacing the external `dot` invocation.
//!
//! Reproduces the subset of graphviz dot used by CucaDiagram edge routing:
//! longest-path ranking, bottom-up x placement (lib/dotgen/position.c),
//! y-coordinate assignment (lib/dotgen/position.c `set_ycoords`), and the
//! box-corridor spline routing (lib/dotgen/dotsplines.c + lib/pathplan).
//!
//! Geometry is computed in the dot internal frame: y grows upward, x in
//! pixels. It is emitted as [`DotSvgResult`] with `full_height = 0`, where
//! the SVG frame has `y = -y_up`, exactly the frame produced by parsing dot
//! SVG output before the global move-delta translation.

use std::collections::{HashMap, HashSet};

use crate::cuca_layout::{DotEdgePath, DotNodePos, DotSvgResult, EntityData};
use crate::entity_link_parser::ParsedLink;
use crate::pathplan::{
    Box as GvBox, Edge as GvEdge, Endpoint, NodeShape, Point, bezier_clip, boxes_to_polygon,
    route_spline, shortest_path,
};

/// Horizontal node separation (matches dot `nodesep`).
const NODESEP: f64 = 35.0;
/// Vertical rank separation (matches dot `ranksep`).
const RANKSEP: f64 = 60.0;
/// Cluster box margin (lib/common/const.h `CL_OFFSET`).
const CL_OFFSET: f64 = 8.0;
/// Extra bounding-box slack (dotsplines.c `FUDGE`).
const FUDGE: f64 = 4.0;
/// Minimum corridor width added on each side (dotsplines.c `MINW`).
const MINW: f64 = 16.0;

/// Half the default node penwidth: graphviz's inside-region is the node box
/// expanded outward by this much (`lib/common/shapes.c`, outline periphery).
const PEN_HALF: f64 = 0.5;

/// Reconstructs a node dimension exactly as graphviz receives it.
///
/// `DotStringFactory` emits node width/height in inches rounded to six
/// decimals after the Svek pixel size was rounded to four decimals; dot
/// parses that string back and multiplies by 72. The native solver must use
/// the same round-tripped value (e.g. `78.4798 → 1.089997 in → 78.479784`)
/// or boundary rounding flips (`102.995 → 103.00` vs `102.99499 → 102.99`).
fn dot_roundtrip_px(px: f64) -> f64 {
    let rounded4 = (px * 10000.0).round() / 10000.0;
    let inches = (rounded4 / 72.0 * 1e6).round() / 1e6;
    inches * 72.0
}

/// A directed edge with its dot `minlen` constraint.
#[derive(Clone, Copy)]
struct REdge<'a> {
    /// Tail name.
    a: &'a String,
    /// Head name.
    b: &'a String,
    /// Dot minlen: 0 for a single-dash `->` (rank=same), 1 otherwise.
    minlen: usize,
    /// Whether the edge carries a label (enlarges the rank gap by 19).
    has_label: bool,
}

/// Returns dot `minlen` for an arrow string.
///
/// A single dash `->` contracts to rank=same; double dash `-->`, dotted
/// `..>` and plain `--` keep one rank apart.
fn edge_minlen(arrow: &str) -> usize {
    if arrow.trim() == "->" {
        0
    } else {
        1
    }
}
struct Node {
    center: Point,
    half_w: f64,
    half_h: f64,
    is_ellipse: bool,
    /// When set, edges clip to this r circle at the node centre instead of
    /// the placement box (component-diagram lollipop port).
    port_radius: Option<f64>,
    rank: usize,
    order: usize,
}

/// Returns the edges of a directed graph made acyclic, in input order.
///
/// Three-color DFS: an edge whose head is on the current DFS stack is a back
/// edge closing a cycle and is dropped. This mirrors graphviz
/// `lib/dotgen/acyclic.c`, which removes a feedback edge set before the
/// network-simplex ranking — the longest-path ranking in [`solve`] only
/// converges on a DAG. Dropped edges are still routed by the caller.
fn acyclic_edges<'a>(
    names: &[&'a String],
    edges: &[(&'a String, &'a String)],
) -> Vec<(&'a String, &'a String)> {
    let index: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); names.len()];
    for (a, b) in edges {
        if let (Some(&i), Some(&j)) = (index.get(a.as_str()), index.get(b.as_str())) {
            adj[i].push(j);
        }
    }

    // 0 = unvisited, 1 = on current DFS stack, 2 = finished
    let mut color = vec![0u8; names.len()];
    let mut excluded: HashSet<(usize, usize)> = HashSet::new();


    for u in 0..names.len() {
        if color[u] == 0 {
            dfs(u, &adj, &mut color, &mut excluded);
        }
    }

    edges
        .iter()
        .filter(|(a, b)| match (index.get(a.as_str()), index.get(b.as_str())) {
            (Some(&i), Some(&j)) => !excluded.contains(&(i, j)),
            _ => true,
        })
        .copied()
        .collect()
}

/// Inserts `n` into the same-rank order after all its precedence
/// predecessors, recursively placing those first. Already-placed nodes are
/// skipped; source order is preserved for independent nodes.
fn place_in_rank<'a>(
    n: &'a String,
    prec: &HashMap<&'a String, Vec<&'a String>>,
    placed: &mut HashSet<&'a String>,
    ordered: &mut Vec<&'a String>,
) {
    if placed.contains(n) {
        return;
    }
    if let Some(deps) = prec.get(n) {
        for &d in deps {
            place_in_rank(d, prec, placed, ordered);
        }
    }
    placed.insert(n);
    ordered.push(n);
}
/// Three-color DFS visitor; records back edges into `excluded`.
fn dfs(
    u: usize,
    adj: &[Vec<usize>],
    color: &mut [u8],
    excluded: &mut HashSet<(usize, usize)>,
) {
    color[u] = 1;
    for &v in &adj[u] {
        match color[v] {
            0 => dfs(v, adj, color, excluded),
            1 => {
                excluded.insert((u, v));
            }
            _ => {}
        }
    }
    color[u] = 2;
}

/// Run the native solver.
///
/// `sorted` is source-line ordered `(name, svek half extents)`. Only links
/// whose both endpoints are entities are routed; all edges use minlen 1.
pub fn solve(
    sorted: &[(&String, &EntityData)],
    links: &[ParsedLink],
) -> DotSvgResult {
    let names: Vec<&String> = sorted.iter().map(|(n, _)| *n).collect();

    // Node half extents from the precomputed SvekNode dimensions.
    let mut raw: HashMap<&String, Node> = HashMap::new();
    for (name, ed) in sorted {
        // A use case's Svek box is exactly its ellipse box; an actor carries
        // head-circle radii (rx = ry = 8) but its placement box is the tall
        // rect below, so it must be treated as a rectangle.
        let is_ellipse =
            (2.0 * ed.rx - ed.svek_w).abs() < 0.5 && (2.0 * ed.ry - ed.svek_h).abs() < 0.5;
        let (half_w, half_h) = (
            dot_roundtrip_px(ed.svek_w) / 2.0,
            dot_roundtrip_px(ed.svek_h) / 2.0,
        );
        raw.insert(
            *name,
            Node {
                center: Point::new(0.0, 0.0),
                half_w,
                half_h,
                is_ellipse,
                port_radius: ed.port_circle,
                rank: 0,
                order: 0,
            },
        );
    }

    // Real edges between known entities, in source order, carrying minlen.
    let real_redges: Vec<REdge> = links
        .iter()
        .filter(|l| raw.contains_key(&l.from) && raw.contains_key(&l.to))
        .map(|l| REdge {
            a: &l.from,
            b: &l.to,
            minlen: edge_minlen(&l.arrow),
            has_label: l.label.is_some(),
        })
        .collect();

    // Synthetic invisible edges for entities with no real edges, mirroring
    // generate_dot_string: split the unconnected set in half and chain the
    // first half to the second. These affect ranks/placement but are routed
    // by no real link.
    let mut with_edge: std::collections::HashSet<&String> = links
        .iter()
        .flat_map(|l| [&l.from, &l.to])
        .collect();
    let unconnected: Vec<&String> = names
        .iter()
        .copied()
        .filter(|n| !with_edge.remove(n))
        .collect();
    let mut all_redges: Vec<REdge> = real_redges.clone();
    if unconnected.len() > 2 {
        let split = unconnected.len().div_ceil(2);
        for i in 0..split.min(unconnected.len() - split) {
            all_redges.push(REdge {
                a: unconnected[i],
                b: unconnected[split + i],
                minlen: 1,
                has_label: false,
            });
        }
    }

    // Drop cycle-closing edges before ranking. Graphviz's network simplex
    // (`lib/dotgen/acyclic.c`) removes the minimum feedback edge set first.
    // Run the DFS over tuple views, then reattach each kept edge's minlen.
    let tuple_views: Vec<(&String, &String)> =
        all_redges.iter().map(|e| (e.a, e.b)).collect();
    let kept_pairs = acyclic_edges(&names, &tuple_views);
    let minlen_of: HashMap<(&String, &String), usize> = all_redges
        .iter()
        .map(|e| ((e.a, e.b), e.minlen))
        .collect();
    let label_of: HashMap<(&String, &String), bool> = all_redges
        .iter()
        .map(|e| ((e.a, e.b), e.has_label))
        .collect();
    let dag_edges: Vec<REdge> = kept_pairs
        .iter()
        .map(|&(a, b)| REdge {
            a,
            b,
            minlen: *minlen_of.get(&(a, b)).unwrap_or(&1),
            has_label: *label_of.get(&(a, b)).unwrap_or(&false),
        })
        .collect();

    // ── Ranking: longest path with per-edge minlen ───────────────────────
    let mut rank: HashMap<&String, usize> = names.iter().map(|n| (*n, 0usize)).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for e in &dag_edges {
            let nr = rank[e.a] + e.minlen;
            if nr > rank[e.b] {
                rank.insert(e.b, nr);
                changed = true;
            }
        }
    }
    let max_rank = rank.values().copied().max().unwrap_or(0);

    // Rank members in source order.
    let mut members: Vec<Vec<&String>> = vec![Vec::new(); max_rank + 1];
    for name in &names {
        members[rank[name]].push(*name);
    }

    // A minlen-0 edge places its tail left of its head within the same rank.
    // Reorder each rank to honor those precedence constraints while keeping
    // the source order otherwise.
    for r in &mut members {
        let member_set: std::collections::HashSet<&String> = r.iter().copied().collect();
        let mut prec: HashMap<&String, Vec<&String>> = HashMap::new();
        for e in &dag_edges {
            if e.minlen == 0 && member_set.contains(e.a) && member_set.contains(e.b) {
                prec.entry(e.b).or_default().push(e.a);
            }
        }
        if prec.values().any(|v| !v.is_empty()) {
            let mut ordered: Vec<&String> = Vec::new();
            let mut placed: std::collections::HashSet<&String> = HashSet::new();
            for &n in r.iter() {
                place_in_rank(n, &prec, &mut placed, &mut ordered);
            }
            *r = ordered;
        }
    }

    for name in &names {
        let r = rank[name];
        raw.get_mut(name).expect("node present").rank = r;
    }
    for r in &members {
        for (i, name) in r.iter().enumerate() {
            raw.get_mut(name).expect("node present").order = i;
        }
    }

    // Per-rank half heights (symmetric).
    let rank_ht: Vec<f64> = members
        .iter()
        .map(|r| r.iter().map(|n| raw[n].half_h).fold(0.0_f64, f64::max))
        .collect();

    // A labeled edge reserves a 19px FIXEDSIZE table in the rank gap.
    let mut label_gap = vec![0.0f64; max_rank + 1];
    for e in &dag_edges {
        if e.has_label && e.minlen > 0 {
            let r = rank[e.a];
            label_gap[r] = label_gap[r].max(19.0);
        }
    }

    // ── y coordinates (set_ycoords) ──────────────────────────────────────
    let mut rank_y = vec![0.0f64; max_rank + 1];
    rank_y[max_rank] = rank_ht[max_rank];
    if max_rank > 0 {
        for r in (0..max_rank).rev() {
            let d0 = rank_ht[r + 1] + rank_ht[r] + RANKSEP + label_gap[r];
            let d1 = rank_ht[r + 1] + rank_ht[r] + CL_OFFSET;
            rank_y[r] = rank_y[r + 1] + d0.max(d1);
        }
    }
    for name in &names {
        let r = rank[name];
        raw.get_mut(name).expect("node present").center.y = rank_y[r];
    }

    // ── x coordinates (position.c) ───────────────────────────────────────
    // Child lists over the edge graph (tail -> heads).
    let mut children: HashMap<&String, Vec<&String>> = names.iter().map(|n| (*n, Vec::new())).collect();
    for e in &dag_edges {
        // Same-rank (minlen 0) edges only order nodes within the rank; they
        // are not parent links in the bottom-up x placement.
        if e.minlen == 0 {
            continue;
        }
        children.get_mut(e.a).expect("tail present").push(e.b);
    }

    let mut center_x: HashMap<&String, f64> = HashMap::new();
    // `ideal_x`: position from the child mean alone, before within-rank
    // packing. The delta (packed - ideal) is propagated down unique-child
    // subtrees so a flat edge that pushes a rank member sideways carries its
    // descendants with it (network simplex tree alignment).
    let mut ideal_x: HashMap<&String, f64> = HashMap::new();
    for r in (0..=max_rank).rev() {
        let mut prev: Option<&String> = None;
        for name in &members[r] {
            let node = &raw[name];
            let ideal = if children[name].is_empty() {
                node.half_w
            } else {
                let sum: f64 = children[name].iter().map(|c| center_x[c]).sum();
                sum / children[name].len() as f64
            };
            let mut c = ideal;
            if let Some(p) = prev {
                let minlen = (raw[p].half_w + NODESEP + node.half_w).round();
                c = c.max(center_x[p] + minlen);
            }
            ideal_x.insert(*name, ideal);
            center_x.insert(*name, c);
            prev = Some(name);
        }
    }
    // Parent links (minlen > 0), used to carry a packing delta top-down.
    let mut parents_of: HashMap<&String, Vec<&String>> =
        names.iter().map(|n| (*n, Vec::new())).collect();
    for e in &dag_edges {
        if e.minlen > 0 {
            parents_of.get_mut(e.b).expect("child").push(e.a);
        }
    }
    // Carry each node's packing delta into its descendants, top-down. A node
    // keeps its own packed x; its children inherit the accumulated ancestor
    // shift so the whole subtree moves with the displaced member.
    let mut carry: HashMap<&String, f64> = HashMap::new();
    for r in 0..=max_rank {
        for name in &members[r] {
            let inherited = parents_of[name]
                .iter()
                .map(|p| carry.get(*p).copied().unwrap_or(0.0))
                .sum::<f64>()
                / parents_of[name].len().max(1) as f64;
            let own = center_x[name] - ideal_x[name];
            carry.insert(*name, inherited + own);
            if inherited != 0.0 {
                center_x.insert(*name, center_x[name] + inherited);
            }
        }
    }
    let min_left = names
        .iter()
        .map(|n| center_x[n] - raw[n].half_w)
        .fold(f64::INFINITY, f64::min);
    for name in &names {
        let cx = center_x[name] - min_left;
        raw.get_mut(name).expect("node present").center.x = cx;
    }

    // ── Routing bounds (spline_info) ─────────────────────────────────────
    // LeftBound/RightBound accumulate MINW per rank.
    let mut left_bound = 0.0f64;
    let mut right_bound = 0.0f64;
    for r in &members {
        let first = &raw[&r[0]];
        let last = &raw[&r[r.len() - 1]];
        left_bound = left_bound.min(first.center.x - first.half_w);
        right_bound = right_bound.max(last.center.x + last.half_w);
        left_bound -= MINW;
        right_bound += MINW;
    }

    // Quantize to dot SVG text precision (2 decimals). Dot prints node
    // centers and radii independently, and the old parser reconstructed the
    // box from those rounded values, so mirror that exactly.
    let q2 = |v: f64| (v * 100.0).round() / 100.0;
    let mut out_nodes = HashMap::new();
    for (idx, name) in names.iter().enumerate() {
        let node = &raw[name];
        let color = (idx + 6) as i32;
        if node.is_ellipse {
            // Ellipse: dot prints center and radii independently; the parser
            // reconstructs the box from those rounded values.
            let cx = q2(node.center.x);
            let cy = q2(node.center.y);
            let hw = q2(node.half_w);
            let hh = q2(node.half_h);
            out_nodes.insert(
                color,
                DotNodePos {
                    min_x: cx - hw,
                    min_y: -(cy + hh),
                },
            );
        } else {
            // Rect: dot prints the polygon vertices (the actual corners),
            // rounded to 2 decimals, and the parser takes them verbatim.
            let min_x = q2(node.center.x - node.half_w);
            let max_y = q2(node.center.y + node.half_h);
            // SVG frame y = -y_up.
            out_nodes.insert(
                color,
                DotNodePos {
                    min_x,
                    min_y: -max_y,
                },
            );
        }
    }

    // A bidirectional pair (u->v and v->u both present) splits its ports:
    // the rank-increasing forward edge routes on the LEFT, the feedback edge
    // stays centered. Build the set of forward edges that have a reverse.
    let edge_pairs: std::collections::HashSet<(&String, &String)> = real_redges
        .iter()
        .map(|e| (e.a, e.b))
        .collect();
    let forward_pairs: std::collections::HashSet<(&String, &String)> = real_redges
        .iter()
        .filter(|e| edge_pairs.contains(&(e.b, e.a)))
        .filter(|e| rank[e.a] < rank[e.b])
        .map(|e| (e.a, e.b))
        .collect();

    // ── Route edges ──────────────────────────────────────────────────────
    let mut out_edges = HashMap::new();
    let first_color = (names.len() + 6) as i32;
    for (i, re) in real_redges.iter().enumerate() {
        let tail_name = re.a;
        let head_name = re.b;
        let same_rank = re.minlen == 0;
        let tail = &raw[tail_name];
        let head = &raw[head_name];
        let rt = tail.rank;
        let rh = head.rank;

        // maximal_bbox for a node, bounded by adjacent same-rank neighbors.
        let maximal_bbox = |node_name: &String| -> (f64, f64) {
            let n = &raw[node_name];
            let order = n.order;
            let r = &members[n.rank];

            let mut b = n.center.x - n.half_w - FUDGE;
            let llx = if order > 0 {
                let left = &raw[&r[order - 1]];
                let nb = left.center.x + left.half_w + NODESEP / 2.0;
                b = b.min(nb);
                b.round()
            } else {
                b.round().min(left_bound)
            };

            let mut b = n.center.x + n.half_w + FUDGE;
            let urx = if order + 1 < r.len() {
                let right = &raw[&r[order + 1]];
                let nb = right.center.x - right.half_w - NODESEP / 2.0;
                b = b.max(nb);
                b.round()
            } else {
                b.round().max(right_bound)
            };
            (llx, urx)
        };

        let (t_ll, t_ur) = maximal_bbox(tail_name);
        let (h_ll, h_ur) = maximal_bbox(head_name);

/// Builds the clip endpoint for `node`: a centre 18x18 port cell when present
/// (an unstroked rectangle, clipped at half 9 exactly), otherwise the node's
/// ellipse/rectangle outline expanded by half a pen.
fn node_endpoint(node: &Node) -> Endpoint {
    if let Some(half) = node.port_radius {
        return Endpoint {
            center: node.center,
            shape: NodeShape::Rect {
                half_w: half,
                half_h: half,
            },
        };
    }
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

        if same_rank {
            // A rank=same edge is a flat horizontal cubic in the internal
            // frame: from the tail's right edge to the head's left edge, then
            // clipped exactly as a regular edge.
            let y = tail.center.y;
            // makeSimpleFlat: centers with controls at 1/3 and 2/3 of the span.
            let p0 = Point::new(tail.center.x, y);
            let p3 = Point::new(head.center.x, y);
            let p1 = Point::new((2.0 * p0.x + p3.x) / 3.0, y);
            let p2 = Point::new((p0.x + 2.0 * p3.x) / 3.0, y);
            let mut curve = [p0, p1, p2, p3];
            let tail_ep = node_endpoint(tail);
            let head_ep = node_endpoint(head);
            bezier_clip(&mut curve, tail_ep, true);
            bezier_clip(&mut curve, head_ep, false);
            let pts = curve.map(|p| (q2(p.x), q2(-p.y)));
            out_edges.insert(first_color + i as i32, DotEdgePath { points: pts });
            continue;
        }

        let is_forward_pair = forward_pairs.contains(&(tail_name, head_name));
        // Forward edge of a bidirectional pair exits/enters the node's LEFT
        // edge; other edges use the node center.
        let port_x = |node: &Node| {
            if is_forward_pair {
                node.center.x - node.half_w
            } else {
                node.center.x
            }
        };
        let tail_box = GvBox {
            ll: Point::new(t_ll, tail.center.y - rank_ht[rt]),
            ur: Point::new(t_ur, tail.center.y),
        };
        let start = Point::new(port_x(tail), tail.center.y - 1.0);

        // rank_box: full-width inter-rank corridor.
        let inter_box = GvBox {
            ll: Point::new(left_bound, head.center.y + rank_ht[rh]),
            ur: Point::new(right_bound, tail.center.y - rank_ht[rt]),
        };

        // endpath generic REGULAREDGE: top half of the head corridor.
        let head_box = GvBox {
            ll: Point::new(h_ll, head.center.y),
            ur: Point::new(h_ur, head.center.y + rank_ht[rh]),
        };
        let end = Point::new(port_x(head), head.center.y + 1.0);

        let boxes: Vec<GvBox> = [tail_box, inter_box, head_box]
            .into_iter()
            .filter(|b| b.ll.x < b.ur.x && b.ll.y < b.ur.y)
            .collect();
        let polygon = boxes_to_polygon(&boxes);
        let pl = shortest_path(&polygon, start, end);
        let barriers: Vec<GvEdge> = polygon
            .iter()
            .enumerate()
            .map(|(k, &a)| GvEdge {
                a,
                b: polygon[(k + 1) % polygon.len()],
            })
            .collect();

        // Default ports are unconstrained: pass zero tangent vectors and let
        // the least-squares fit derive the endpoint slopes, as dot does.
        let free = Point::new(0.0, 0.0);
        let mut fitted = route_spline(&barriers, &pl, free, free);
        if fitted.len() >= 4 {
            let n = fitted.len();
            let mut curve = [fitted[0], fitted[1], fitted[n - 2], fitted[n - 1]];
            let tail_ep = node_endpoint(tail);
            let head_ep = node_endpoint(head);
            bezier_clip(&mut curve, tail_ep, true);
            bezier_clip(&mut curve, head_ep, false);
            fitted = curve.to_vec();
        }

        // Emit in SVG frame (y = -y_up), quantized to dotsvg precision.
        let pts: [(f64, f64); 4] = {
            let n = fitted.len();
            let take = [0, 1, n.saturating_sub(2), n - 1];
            take.map(|k| (q2(fitted[k].x), q2(-fitted[k].y)))
        };
        out_edges.insert(first_color + i as i32, DotEdgePath { points: pts });
    }

    DotSvgResult { nodes: out_nodes, edges: out_edges }
}


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
pub const NODESEP: f64 = 35.0;
/// Vertical rank separation (matches dot `ranksep`).
const RANKSEP: f64 = 60.0;
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
/// decimals, converted directly from the Svek pixel size (`px / 72`); dot
/// parses that string back and multiplies by 72. The native solver must use
/// the same round-tripped value or a boundary rounding flips (e.g. Service
/// width 86.703812 px -> `1.204220` in -> 86.70384 px, not 86.70377).
pub fn dot_roundtrip_px(px: f64) -> f64 {
    let inches = (px / 72.0 * 1e6).round() / 1e6;
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
    usize::from(arrow.trim() != "->")
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
pub fn acyclic_edges<'a>(
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

/// Detects the topology on which graphviz's doubled-rank label solver is
/// bit-exact: at least one labeled interrank edge and exactly one real node
/// on every longest-path rank (a linear chain, back edges excluded from
/// ranking as in [`solve`]). Forks/fans share a rank and keep the native
/// solver; two-node links do as well because the native gap-center label y
/// matches the reference there whereas the doubled solver's integer rank
/// arithmetic is 0.002 off for the object-diagram node heights.
///
/// Ported criterion: graphviz `lib/dotgen/rank.c::edgelabel_ranks` doubles
/// every graph with a label, but the pure-Rust doubled solver
/// (`labeled_layout.rs`) only reproduces the singleton-rank case.
pub fn is_singleton_rank_chain(
    sorted: &[(&String, &EntityData)],
    links: &[ParsedLink],
) -> bool {
    let names: Vec<&String> = sorted.iter().map(|(n, _)| *n).collect();
    let known: HashSet<&String> = names.iter().copied().collect();
    // `(tail, head, dot minlen)`: a single-dash arrow contracts to rank 0.
    let real: Vec<(&String, &String, usize)> = links
        .iter()
        .filter(|l| known.contains(&l.from) && known.contains(&l.to))
        .map(|l| (&l.from, &l.to, edge_minlen(&l.arrow)))
        .collect();
    let has_label = links
        .iter()
        .any(|l| l.label.is_some() && known.contains(&l.from) && known.contains(&l.to));
    if !has_label {
        return false;
    }

    // `acyclic_edges` drops the minimum feedback set and returns name
    // references, so recover each surviving tuple's minlen from a multiset
    // keyed on the endpoint values (parallel edges allowed).
    let views: Vec<(&String, &String)> =
        real.iter().map(|(a, b, _)| (*a, *b)).collect();
    let kept = acyclic_edges(&names, &views);
    let mut minlen_pool: HashMap<(&str, &str), Vec<usize>> = HashMap::new();
    for (a, b, minlen) in &real {
        minlen_pool
            .entry((a.as_str(), b.as_str()))
            .or_default()
            .push(*minlen);
    }
    let kept_minlen: Vec<usize> = kept
        .iter()
        .map(|(a, b)| {
            minlen_pool
                .get_mut(&(a.as_str(), b.as_str()))
                .and_then(Vec::pop)
                .unwrap_or(1)
        })
        .collect();
    let mut rank: HashMap<&String, usize> = names.iter().map(|n| (*n, 0usize)).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for ((a, b), &minlen) in kept.iter().zip(kept_minlen.iter()) {
            let nr = rank[a] + minlen;
            if nr > rank[b] {
                rank.insert(*b, nr);
                changed = true;
            }
        }
    }
    let max_rank = rank.values().copied().max().unwrap_or(0);
    if max_rank < 2 {
        return false;
    }
    let mut counts = vec![0usize; max_rank + 1];
    for r in rank.values() {
        counts[*r] += 1;
    }
    counts.into_iter().all(|c| c == 1)
}

/// Run the native solver.
///
/// `sorted` is source-line ordered `(name, svek half extents)` of leaves only.
/// `leaf_group` maps a leaf to its immediate containing group key; a leaf absent
/// from the map is unpackaged. Only links whose both endpoints are entities are
/// routed. Within a cluster ranks keep the 60px `ranksep`; at a cluster boundary
/// the cluster-inflated rank half-heights plus the 16px cluster margin apply.
/// `group_lines` gives each group key's title line count, which sizes the
/// upward channel reserve (`position.c::clust_ht` cluster label border).
pub fn solve(
    sorted: &[(&String, &EntityData)],
    links: &[ParsedLink],
    leaf_group: &HashMap<&String, &String>,
    group_lines: &HashMap<&String, usize>,
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
    // Cluster-inflated per-rank half extents (`position.c::set_ycoords` +
    // `clust_ht`). The primitive rank half-height (`rank_ht`) is symmetric
    // and still drives NODE centres (`rank_y`). A titled folder is emitted as
    // three nested 8px-margin subgraphs (`p0`, the labelled cluster, `p1`);
    // only the labelled level carries the title table. Its requirements
    // propagate to the folder's min rank upward (ht2) and max rank downward
    // (ht1), and a rank takes the maximum over every folder touching it:
    //   bottom (ht1, max rank): three 8px margins = +24 (no bottom border).
    //   top    (ht2, min rank): +24 margins + title top border, where the
    //           border is the 19px-per-line table plus 8 = `19*L + 8`, so
    //           the total upward reserve is `19*L + 32` (one line -> +51).
    // These feed ONLY the edge channel boxes, reproducing the tall head band
    // that bends a cross-folder edge through the inter-cluster gap.
    let mut rank_ht1 = rank_ht.clone();
    let mut rank_ht2 = rank_ht.clone();
    let mut group_rank_span: HashMap<&String, (usize, usize)> = HashMap::new();
    for (leaf, grp) in leaf_group {
        let r = rank[leaf];
        group_rank_span
            .entry(*grp)
            .and_modify(|(lo, hi)| {
                *lo = (*lo).min(r);
                *hi = (*hi).max(r);
            })
            .or_insert((r, r));
    }
    for (grp, (minr, maxr)) in group_rank_span {
        // The three 8px subgraph margins always bound both ends; the label
        // table border only enlarges the top of a TITLED folder.
        rank_ht2[minr] = rank_ht2[minr].max(rank_ht[minr] + 24.0);
        rank_ht1[maxr] = rank_ht1[maxr].max(rank_ht[maxr] + 24.0);
        if let Some(&lines) = group_lines.get(grp) {
            rank_ht2[minr] = rank_ht2[minr].max(rank_ht[minr] + 19.0 * lines as f64 + 32.0);
        }
    }
    // A labeled edge reserves a 19px FIXEDSIZE table in the rank gap.
    let mut label_gap = vec![0.0f64; max_rank + 1];
    for e in &dag_edges {
        if e.has_label && e.minlen > 0 {
            let r = rank[e.a];
            label_gap[r] = label_gap[r].max(19.0);
        }
    }

    // ── y coordinates (set_ycoords) ──────────────────────────────────────
    // Per-rank set of immediate-group keys (absent for unpackaged leaves).
    let rank_groups: Vec<HashSet<&String>> = members
        .iter()
        .map(|r| {
            r.iter()
                .filter_map(|n| leaf_group.get(n).copied())
                .collect()
        })
        .collect();

    // Box-to-box gap between an upper rank `r` and the rank below. It is 78
    // only when a grouped member above and a grouped member below belong to
    // DIFFERENT clusters (two stacked clusters): the upper cluster's 16px
    // bottom margin, the 24px boundary, and the lower cluster's 38px tab all
    // occupy the gap (16 + 24 + 38 = 78). Otherwise — same cluster, two loose
    // nodes, or a loose node above a cluster — the primitive 60px ranksep
    // dominates and any lower tab fits inside it.
    let gap_below = |r: usize| -> f64 {
        let above = &rank_groups[r];
        let below = &rank_groups[r + 1];
        // 78 only when the two ranks belong to fully disjoint folder sets
        // (clusters stacked vertically). Side-by-side sibling folders share
        // the rank band, so a member that stays in the SAME folder across the
        // boundary (above ∩ below ≠ ∅) keeps the primitive 60px ranksep; the
        // neighbouring cluster's vertical padding overlaps that band and adds
        // no gap. Verified: WebApp→Cache stays in Application Server while
        // Database Server sits beside it, and graphviz gives a 60px gap.
        let stacked =
            !above.is_empty() && !below.is_empty() && above.intersection(below).next().is_none();
        if stacked {
            78.0
        } else {
            RANKSEP
        }
    };

    let mut rank_y = vec![0.0f64; max_rank + 1];
    rank_y[max_rank] = rank_ht[max_rank];
    if max_rank > 0 {
        for r in (0..max_rank).rev() {
            let gap = gap_below(r) + label_gap[r];
            rank_y[r] = rank_y[r + 1] + rank_ht[r + 1] + rank_ht[r] + gap;
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

    // graphviz solves the horizontal constraint system in integer point
    // units (`ND_coord.x = ND_rank`, an int): leaf anchors are truncated
    // half-widths (`gv_nodesize`: `rw = trunc(w/2)`), within-rank packing
    // uses those integer boxes, and a multi-child barycentre is an integer
    // (floored) mean. The sub-pixel residual (e.g. a 45.079px half-width vs
    // the integer 45) is re-introduced once, globally, by the frame
    // normalization below, which anchors on the fractional leaf box edge.
    //
    // The integer path is enabled only on a labelled graph that fans out
    // (one tail to two inter-rank heads sharing a rank): the topology whose
    // rank-doubled label/edge-pair vnodes graphviz's horizontal network
    // simplex resolves to an integer barycentre distinct from the f64 mean.
    let has_labeled_edge = dag_edges.iter().any(|e| e.has_label && e.minlen > 0);
    // Graphviz's horizontal network simplex resolves every real node to an
    // integer centre. On a labelled tree this is bit-exact whenever a rank
    // carries two or more real members: that rank is the anchor — its first
    // member sits on its integer half-width and the rest are packed with the
    // half-up `ROUND(rw + lw + nodesep)` clearance (`position.c
    // ::make_LR_constraints`) — and every other rank's sole member takes the
    // floored mean of its already-placed neighbours (`make_edge_pairs` slack
    // minimization), propagated outward in both directions so both fan-out
    // and fan-in converge identically.
    let wide: Vec<usize> = (0..=max_rank).filter(|&r| members[r].len() >= 2).collect();
    // Two leaves on the same rank belong to different sibling folders when
    // their immediate groups differ; that forces graphviz's cluster-envelope
    // separation even without an edge label.
    let rank_crosses_folder = wide.iter().any(|&r| {
        let mut folder = None;
        for n in &members[r] {
            let g = leaf_group.get(n).copied();
            if g.is_some() {
                if folder.is_none() {
                    folder = g;
                } else if folder != g {
                    return true;
                }
            }
        }
        false
    });
    let integer_x = !wide.is_empty() && (has_labeled_edge || rank_crosses_folder);

    let mut center_x: HashMap<&String, f64> = HashMap::new();
    // `ideal_x`: position from the barycentre alone, before within-rank
    // packing. The delta (packed - ideal) is propagated down unique-child
    // subtrees on the fractional path.
    let mut ideal_x: HashMap<&String, f64> = HashMap::new();
    if integer_x {
        // Top-level folder of each leaf. PlantUML renders a `node`/`package`
        // body as a graphviz cluster wrapped in three nested 8px-margin
        // subgraphs (`p0`, the cluster, `p1`). Leaves in the same folder
        // share one column; leaves in distinct sibling folders get their own
        // column even when an edge joins them.
        let folder_of = |name: &&String| leaf_group.get(*name).copied();
        // Envelope half of a folder = the rounded half-width of its widest
        // leaf (`contain_nodes` bounds ln/rn from each member).
        let mut folder_half: HashMap<&String, f64> = HashMap::new();
        for name in &names {
            if let Some(g) = folder_of(name) {
                let e = folder_half.entry(g).or_insert(0.0);
                *e = e.max(raw[name].half_w.round());
            }
        }
        // Two leaves align across an edge unless they sit in different
        // folders; sibling-folder columns are positioned only by the anchor
        // rank envelope clearance, never by the cross-folder edge.
        let aligns = |a: &&String, b: &&String| match (folder_of(a), folder_of(b)) {
            (Some(ga), Some(gb)) => ga == gb,
            _ => true,
        };

        // Widest rank anchors the integer frame; ties take the lower rank.
        let anchor_rank = *wide
            .iter()
            .max_by(|&&a, &&b| members[a].len().cmp(&members[b].len()).then(b.cmp(&a)))
            .unwrap();
        let mut prev: Option<&String> = None;
        for name in &members[anchor_rank] {
            let mut c = raw[name].half_w.trunc();
            if let Some(p) = prev {
                let clearance = match (folder_of(&p), folder_of(name)) {
                    (Some(gp), Some(gn)) if gp != gn => {
                        // Sibling folders overlapping this rank: three nested
                        // 8px margins from each leaf edge to its outer folder
                        // edge (24) plus the 8px inter-cluster gap.
                        folder_half[&gp] + 56.0 + folder_half[&gn]
                    }
                    _ => {
                        // `ROUND(rw(p) + lw(n) + nodesep)`, half-up.
                        (raw[p].half_w + raw[name].half_w + NODESEP + 0.5).floor()
                    }
                };
                c = c.max(center_x[p] + clearance);
            }
            center_x.insert(*name, c);
            prev = Some(*name);
        }

        // Inter-rank adjacency over the real edge graph, restricted to edges
        // whose endpoints align (same folder or an ungrouped leaf).
        let mut adj: HashMap<&String, Vec<&String>> =
            names.iter().map(|n| (*n, Vec::new())).collect();
        for e in &dag_edges {
            if e.minlen > 0 && aligns(&e.a, &e.b) {
                adj.get_mut(e.a).expect("tail").push(e.b);
                adj.get_mut(e.b).expect("head").push(e.a);
            }
        }

        // Floored-barycentre propagation from the anchor until every real
        // node is placed; a node averages only neighbours already placed.
        let mut placed: std::collections::HashSet<&String> =
            members[anchor_rank].iter().copied().collect();
        while placed.len() < names.len() {
            let mut progressed = false;
            for name in &names {
                if placed.contains(name) {
                    continue;
                }
                let known: Vec<f64> = adj[name]
                    .iter()
                    .filter(|n| placed.contains(*n))
                    .map(|n| center_x[*n])
                    .collect();
                if !known.is_empty() {
                    let mean = known.iter().sum::<f64>() / known.len() as f64;
                    center_x.insert(*name, mean.floor());
                    placed.insert(*name);
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
        // No packing displacement on the anchored integer solution.
        for name in &names {
            ideal_x.insert(*name, center_x[name]);
        }
    } else {
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
    for rank_members in &members {
        for name in rank_members {
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
    // ── Frame normalization ──────────────────────────────────────────────
    // Each cluster box stays symmetric about its member leaves' centre: the
    // box half-width is the ROUNDED content half-extent plus the 16px cluster
    // margin (`round(maxhalf + 16)`), so the left edge is `centre − extent`,
    // not a rounded absolute edge. The frame origin is the outer-most cluster
    // edge (or a loose leaf's edge); normalizing it to zero makes a stacked
    // column land on an integer centre after the diagram margin is applied.
    let mut group_members: HashMap<&String, Vec<&String>> = HashMap::new();
    for (leaf, grp) in leaf_group {
        group_members.entry(*grp).or_default().push(*leaf);
    }

    let mut origin = 0.0f64;
    for leaves in group_members.values() {
        // Member leaves' centre (they coincide for a single stacked column).
        let cc = {
            let s: f64 = leaves.iter().map(|l| center_x[l]).sum();
            s / leaves.len() as f64
        };
        // Content reaches this far left of `cc`.
        let inner = leaves
            .iter()
            .map(|l| cc - (center_x[l] - raw[l].half_w))
            .fold(0.0_f64, f64::max);
        let extent = (inner + 16.0).round();
        origin = origin.min(cc - extent);
    }
    // Unpackaged leaves anchor the frame on their box edge.
    for name in &names {
        if !leaf_group.contains_key(name) {
            origin = origin.min(center_x[name] - raw[name].half_w);
        }
    }
    for name in &names {
        let cx = center_x[name] - origin;
        raw.get_mut(name).expect("node present").center.x = cx;
    }

    // Titled facing edge of each folder in the normalized routing frame:
    // `col ± (round(widest member half) + 16)`. This is graphviz's inner `p1`
    // subgraph edge (`col ± (roundhalf + 8)`) plus the 8px `Splinesep`, and it
    // is the boundary at which a cross-folder edge's channel box is clipped
    // (`dotsplines.c::maximal_bbox` + `cl_bound`), forcing the route through
    // the inter-folder gap.
    let mut folder_half: HashMap<&String, f64> = HashMap::new();
    let mut folder_col: HashMap<&String, f64> = HashMap::new();
    for name in &names {
        if let Some(&g) = leaf_group.get(name) {
            let half = raw[name].half_w.round();
            folder_half
                .entry(g)
                .and_modify(|h| *h = h.max(half))
                .or_insert(half);
            folder_col.insert(g, raw[name].center.x);
        }
    }
    let folder_face: HashMap<&String, (f64, f64)> = folder_half
        .iter()
        .map(|(g, &h)| {
            let c = folder_col[g];
            (*g, (c - h - 16.0, c + h + 16.0))
        })
        .collect();
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

    // Quantize to dot SVG text precision (2 decimals), used when emitting the
    // edge spline coordinates that graphviz would print as text.
    let q2 = |v: f64| (v * 100.0).round() / 100.0;
    // Emit each placed node box quantized to the precision graphviz prints:
    // the external-dot path (and the Smetana output these tests pin) carries
    // 2-decimal coordinates. Cluster boxes are derived from the placed leaf
    // centres but their rounding absorbs this quantization.
    let mut out_nodes = HashMap::new();
    for (idx, name) in names.iter().enumerate() {
        let node = &raw[name];
        let color = (idx + 6) as i32;
        if node.is_ellipse {
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
            let min_x = q2(node.center.x - node.half_w);
            let max_y = q2(node.center.y + node.half_h);
            out_nodes.insert(
                color,
                DotNodePos {
                    min_x,
                    min_y: -max_y,
                },
            );
        }
    }

    // ── Route edges ──────────────────────────────────────────────────────
    let mut out_edges = HashMap::new();
    // Per-edge label placement, keyed by the edge color the consumer uses:
    // `label_vnode_y` is the label virtual node's rank coordinate (y-up);
    // `label_box_left` is the label table's left edge in the same frame.
    let mut label_vnode_y: HashMap<i32, f64> = HashMap::new();
    let mut label_box_left: HashMap<i32, f64> = HashMap::new();
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
        // A neighbour in a different folder bounds at that folder's titled
        // facing edge (graphviz `cl_bound`: the inner p1 edge + Splinesep),
        // which routes a cross-folder edge out through the gap rather than
        // straight to the node; same-folder neighbours use the node half-width
        // plus half `nodesep`.
        let maximal_bbox = |node_name: &String| -> (f64, f64) {
            let n = &raw[node_name];
            let order = n.order;
            let r = &members[n.rank];
            let node_folder = leaf_group.get(node_name).copied();

            let mut b = n.center.x - n.half_w - FUDGE;
            let llx = if order > 0 {
                let lname = r[order - 1];
                let left = &raw[&lname];
                let nb = match (node_folder, leaf_group.get(&lname).copied()) {
                    (Some(nf), Some(lf)) if nf != lf => folder_face[&lf].1,
                    _ => left.center.x + left.half_w + NODESEP / 2.0,
                };
                b = b.min(nb);
                b.round()
            } else {
                b.round().min(left_bound)
            };

            let mut b = n.center.x + n.half_w + FUDGE;
            let urx = if order + 1 < r.len() {
                let rname = r[order + 1];
                let right = &raw[&rname];
                let nb = match (node_folder, leaf_group.get(&rname).copied()) {
                    (Some(nf), Some(rf)) if nf != rf => folder_face[&rf].0,
                    _ => right.center.x - right.half_w - NODESEP / 2.0,
                };
                b = b.max(nb);
                b.round()
            } else {
                b.round().max(right_bound)
            };
            (llx, urx)
        };

        let (t_ll, t_ur) = maximal_bbox(tail_name);
        let (h_ll, h_ur) = maximal_bbox(head_name);


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
            out_edges.insert(first_color + i as i32, DotEdgePath { points: pts.to_vec() });
            continue;
        }

        // beginpath/endpath: the edge leaves the tail and enters the head at
        // their centers. Tail box spans the lower (ht1) half of the tail
        // rank, the inter-rank corridor runs from the head's upper cluster
        // extent (ht2) to the tail's lower extent (ht1), and the head box
        // spans the head's upper (ht2) half. These use the cluster-inflated
        // extents so a cross-folder edge bends around the tall title band.
        let tail_box = GvBox {
            ll: Point::new(t_ll, tail.center.y - rank_ht1[rt]),
            ur: Point::new(t_ur, tail.center.y),
        };
        let start = Point::new(tail.center.x, tail.center.y - 1.0);
        let inter_box = GvBox {
            ll: Point::new(left_bound, head.center.y + rank_ht2[rh]),
            ur: Point::new(right_bound, tail.center.y - rank_ht1[rt]),
        };
        let head_box = GvBox {
            ll: Point::new(h_ll, head.center.y),
            ur: Point::new(h_ur, head.center.y + rank_ht2[rh]),
        };
        let end = Point::new(head.center.x, head.center.y + 1.0);

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
        if re.has_label && !same_rank {
            // Graphviz inserts a fixed 19px label table on a doubled rank:
            // the virtual node's rank coordinate is the lower node's top
            // plus halved ranksep (30) plus half the table (9.5), and the
            // label table's left edge is the greatest x reached in that band
            // by `limitBoxes` coarse-sampling the *straight* centre-to-centre
            // cubic (`routespl.c::limitBoxes`, `dotsplines.c::recover_slack`).
            let vn_y = head.center.y + head.half_h + 39.5;
            let start = Point::new(tail.center.x, tail.center.y - 1.0);
            let end = Point::new(head.center.x, head.center.y + 1.0);
            let raw_x =
                crate::labeled_layout::label_box_left_straight(start, end, vn_y);
            label_vnode_y.insert(first_color + i as i32, q2(vn_y));
            label_box_left.insert(first_color + i as i32, q2(raw_x));
        }
        if fitted.len() >= 4 {
            // A route around a tall cluster splits into two or more cubics
            // sharing junction points (`really_route` recursion): the stream
            // is p0,c1,c2,p3,c4,c5,p6,… Clip only the FIRST cubic against
            // the tail and the LAST cubic against the head, leaving interior
            // junctions untouched, then emit the whole multi-segment stream.
            let n = fitted.len();
            let tail_ep = node_endpoint(tail);
            let head_ep = node_endpoint(head);
            let mut first = [fitted[0], fitted[1], fitted[2], fitted[3]];
            bezier_clip(&mut first, tail_ep, true);
            fitted[0..4].copy_from_slice(&first);
            let mut last = [fitted[n - 4], fitted[n - 3], fitted[n - 2], fitted[n - 1]];
            bezier_clip(&mut last, head_ep, false);
            fitted[n - 4..n].copy_from_slice(&last);
        }
        // Emit in SVG frame (y = -y_up), quantized to dotsvg precision. The
        // point stream advances three controls per cubic (`emit_edge_path`).
        let pts: Vec<(f64, f64)> = fitted
            .iter()
            .map(|p| (q2(p.x), q2(-p.y)))
            .collect();
        out_edges.insert(first_color + i as i32, DotEdgePath { points: pts });
    }

    let raw_yup: HashMap<String, (f64, f64)> = names
        .iter()
        .map(|name| ((**name).clone(), (raw[name].center.y, raw[name].half_h)))
        .collect();

    DotSvgResult {
        nodes: out_nodes,
        edges: out_edges,
        raw_yup,
        label_box_left,
        label_vnode_y,
        draw_order: Vec::new(),
    }
}

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


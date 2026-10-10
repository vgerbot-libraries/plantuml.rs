//! Autarkic composite-state folding.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/dot/CucaDiagramSimplifierState.java`
//! - `net/sourceforge/plantuml/svek/GroupMakerState.java`
//! - `net/sourceforge/plantuml/svek/InnerStateAutonom.java`
//!
//! Before the outer Svek layout, a state group whose links are all internal is
//! laid out independently with the graphviz default `nodesep` (0.25in = 18px)
//! and `ranksep` (0.5in = 36px), then rewritten from a cluster subgraph into an
//! ordinary rounded-rectangle leaf that carries the inner diagram as nested
//! content. Without this pass the group stays a folder cluster, links incident
//! on it are dropped, and the start/end pseudo-states collapse into it.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::cuca_layout::{
    CucaLayout, DotSvgResult, EntityData, build_entity_data, compute_layout_ranked,
};
use crate::entity_link_parser::{EntityKind, ParsedEntity, ParsedLink};
use crate::labeled_layout::solve_labeled;

/// Inner sublayout default node separation: graphviz 0.25in = 18px.
const INNER_NODESEP: f64 = 18.0;
/// Inner sublayout default rank separation: graphviz 0.5in = 36px.
const INNER_RANKSEP: f64 = 36.0;
/// `IEntityImage.MARGIN` and `MARGIN_LINE` are both 5; the composite reserves
/// their sum (10) on each horizontal side and below the body.
const MARGIN: f64 = 5.0;
const MARGIN_LINE: f64 = 5.0;
/// Edge-label baseline offset below the rank-gap center.
const LABEL_BASELINE_OFFSET: f64 = 5.397;
/// Left readback border of an inner `shape=rect,style=rounded` STATE node as
/// Smetana emits it: the rounded-corner path overshoots the nominal 1px
/// `URectangle` border by 0.3475, so the readback min is `box − 1.3475`.
const INNER_STATE_BORDER_X: f64 = 1.3475;
const FONT_SIZE_NAME_INNER: i32 = 14;
/// Font size of an edge label.
const LABEL_FONT_SIZE_INNER: i32 = 13;
/// A positioned state leaf inside a folded composite, in image-local
/// coordinates.
#[derive(Debug, Clone)]
pub struct FoldNode {
    /// Global entity id (`ent0002`, ...).
    pub entity_id: String,
    /// Bare display label.
    pub display: String,
    /// Top-left X of the rounded box.
    pub x: f64,
    /// Top-left Y of the rounded box.
    pub y: f64,
    /// Box width.
    pub width: f64,
    /// Box height.
    pub height: f64,
    /// Name text X.
    pub text_x: f64,
    /// Name text baseline Y.
    pub text_y: f64,
    /// Name rendered width (`textLength`).
    pub text_width: f64,
    /// Source line of the state declaration.
    pub source_line: usize,
}

/// A positioned transition inside a folded composite, in image-local
/// coordinates.
#[derive(Debug, Clone)]
pub struct FoldEdge {
    /// Global link id (`lnk4`, ...).
    pub link_id: String,
    /// Path id (`Running-to-Paused`).
    pub path_id: String,
    /// Raw cubic control points (tail→head), image-local.
    pub points: Vec<(f64, f64)>,
    /// Optional edge label.
    pub label: Option<String>,
    /// Optional label baseline anchor.
    pub label_anchor: Option<(f64, f64)>,
    /// Source line of the transition.
    pub source_line: usize,
    /// Global entity id of the tail endpoint.
    pub from_entity: String,
    /// Global entity id of the head endpoint.
    pub to_entity: String,
    /// Comment text (`Running to Paused`).
    pub comment: String,
}

/// The independently laid-out inner diagram of a folded composite state.
#[derive(Debug, Clone)]
pub struct FoldFragment {
    /// Outer leaf (group) name.
    pub name: String,
    /// Composite leaf width.
    pub width: f64,
    /// Composite leaf height.
    pub height: f64,
    /// Height of the title band; the inner content is drawn below it.
    pub name_height: f64,
    /// Inner state leaves in source order (image-local coordinates).
    pub nodes: Vec<FoldNode>,
    /// Inner transitions in draw order (image-local coordinates).
    pub edges: Vec<FoldEdge>,
}

/// Entity and link ids assigned from one global creation-order stream over the
/// whole parsed diagram (entities and links interleaved by source line), as
/// PlantUML assigns them at parse time, before folding.
#[derive(Debug, Clone, Default)]
pub struct GlobalIds {
    /// Entity id by qualified name.
    pub entity: HashMap<String, u32>,
    /// Link id by source line.
    pub link: HashMap<usize, u32>,
}

impl GlobalIds {
    /// Formats an entity id.
    #[must_use]
    pub fn entity_id(&self, name: &str) -> String {
        format!("ent{:04}", self.entity.get(name).copied().unwrap_or(0))
    }
    /// Formats a link id.
    #[must_use]
    pub fn link_id(&self, source_line: usize) -> String {
        format!("lnk{}", self.link.get(&source_line).copied().unwrap_or(0))
    }
}

/// Assigns global ids by interleaving every entity (group and leaf) and link in
/// source order; at a shared line the entity is created before its link.
fn assign_global_ids(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
) -> GlobalIds {
    // `(source_line, kind)` where kind 0 = entity, 1 = link.
    let mut events: Vec<(usize, u8, Option<&str>)> = entities
        .iter()
        .map(|(name, e)| (e.source_line, 0, Some(name.as_str())))
        .collect();
    for link in links {
        events.push((link.source_line, 1, None));
    }
    events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut ids = GlobalIds::default();
    let mut counter = 0u32;
    for (line, kind, name) in events {
        counter += 1;
        if kind == 0 {
            ids.entity.insert(name.unwrap_or("").to_string(), counter);
        } else {
            ids.link.insert(line, counter);
        }
    }
    ids
}

/// Returns true when a state group is autarkic: a STATE group with no link
/// crossing its boundary.
///
/// Ported from `Entity.isAutarkic` and `EntityUtils.isPureInnerLink3`. A link
/// with exactly one endpoint inside the group crosses the boundary and
/// disqualifies folding; links wholly inside or wholly outside do not.
fn is_autarkic(
    group_key: &str,
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
) -> bool {
    let Some(group) = entities.get(group_key) else {
        return false;
    };
    if !group.group || group.kind != EntityKind::State {
        return false;
    }
    let prefix = format!("{group_key}.");
    links.iter().all(|link| {
        let inside_from = link.from.starts_with(&prefix);
        let inside_to = link.to.starts_with(&prefix);
        inside_from == inside_to
    })
}

/// Builds the inner entity set and pure-inner links for one group, stripping
/// the group qualifier from keys and endpoints.
fn build_inner(
    group_key: &str,
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
) -> (HashMap<String, ParsedEntity>, Vec<ParsedLink>) {
    let prefix = format!("{group_key}.");
    let mut inner = HashMap::new();
    for (key, entity) in entities {
        if let Some(bare) = key.strip_prefix(&prefix) {
            if !bare.contains('.') {
                let mut e = entity.clone();
                e.name = bare.to_string();
                e.parent = None;
                e.group = false;
                inner.insert(bare.to_string(), e);
            }
        }
    }
    let mut inner_links = Vec::new();
    for link in links {
        let from = strip_bare(&link.from, &prefix);
        let to = strip_bare(&link.to, &prefix);
        if let (Some(from), Some(to)) = (from, to) {
            inner_links.push(ParsedLink {
                from: from.to_string(),
                to: to.to_string(),
                arrow: link.arrow.clone(),
                label: link.label.clone(),
                source_line: link.source_line,
            });
        }
    }
    (inner, inner_links)
}

/// Strips the group qualifier, returning the bare name only for a direct
/// (single-segment) inner endpoint.
fn strip_bare<'a>(resolved: &'a str, prefix: &str) -> Option<&'a str> {
    let bare = resolved.strip_prefix(prefix)?;
    (!bare.contains('.')).then_some(bare)
}
/// Result of folding: rewritten outer entities/links, nested fragments, and
/// the global id map for relabelling the outer layout.
pub struct FoldResult {
    /// Outer entities after rewriting.
    pub entities: HashMap<String, ParsedEntity>,
    /// Outer links after removing pure-inner links.
    pub links: Vec<ParsedLink>,
    /// Nested inner fragments keyed by outer leaf name.
    pub fragments: HashMap<String, FoldFragment>,
    /// Global creation-order ids.
    pub ids: GlobalIds,
}


/// Name-band height of a state rendered by `EntityImageState`
/// (`MARGIN + 14pt line + MARGIN_LINE`).
const STATE_NAME_H: f64 = 5.0 + 14.0 * 1.361_994 + 5.0;
/// Baseline offset of a state's centered name from its box top.
const STATE_TEXT_DY: f64 = 19.966;

/// Builds the nested fragment for one autarkic group by running the labeled
/// edge solver directly on the inner entities and then applying the same
/// `moveDelta(6 - minX, 6 - minY)` normalization `SvekResult.calculateDimension`
/// uses.
fn build_fragment(
    group_key: &str,
    inner: &HashMap<String, ParsedEntity>,
    inner_links: &[ParsedLink],
    ids: &GlobalIds,
) -> FoldFragment {
    let entity_data = build_entity_data(inner, plantuml_core::DiagramType::State);

    // Source-ordered inner leaves, the same order `solve_labeled` numbers
    // nodes (color = idx + 6).
    let mut leaf_sorted: Vec<&String> = inner
        .iter()
        .filter(|(_, e)| !e.group)
        .map(|(name, _)| name)
        .collect();
    leaf_sorted.sort_by_key(|n| inner[*n].source_line);
    let sorted_data: Vec<(&String, &EntityData)> = leaf_sorted
        .iter()
        .map(|name| (*name, &entity_data[*name]))
        .collect();

    let dot: DotSvgResult = solve_labeled(&sorted_data, inner_links, INNER_NODESEP, INNER_RANKSEP);
    let first_color = (leaf_sorted.len() + 6) as i32;

    // Move-delta minimum: inner node placed left extent plus the state
    // border readback (`min_border_x(State)`), folded with the recovered
    // edge-label box / control-point minimum exactly as build_layout_from_dot.
    let inner_border = |name: &str| {
        if inner[name].kind == EntityKind::State {
            -INNER_STATE_BORDER_X
        } else {
            crate::cuca_layout::min_border_x(inner[name].kind)
        }
    };
    let node_min_x = leaf_sorted
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            let pos = dot.nodes.get(&((idx + 6) as i32))?;
            Some(pos.min_x + inner_border(name))
        })
        .fold(f64::INFINITY, f64::min);
    let edge_min_x = dot
        .edges
        .values()
        .flat_map(|e| e.points.iter().map(|p| p.0))
        .fold(f64::INFINITY, f64::min);
    let min_x = node_min_x.min(edge_min_x);

    let node_min_y = leaf_sorted
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            let pos = dot.nodes.get(&((idx + 6) as i32))?;
            Some(pos.min_y + crate::cuca_layout::min_border_y(inner[*name].kind))
        })
        .fold(f64::INFINITY, f64::min);

    let dx = 6.0 - min_x;
    let dy = 6.0 - node_min_y;

    let nodes: Vec<FoldNode> = leaf_sorted
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            let entity = &inner[*name];
            let ed = &entity_data[*name];
            let pos = dot.nodes.get(&((idx + 6) as i32))?;
            let x = pos.min_x + dx;
            let y = pos.min_y + dy;
            Some(FoldNode {
                entity_id: ids.entity_id(&format!("{group_key}.{name}")),
                display: entity.display.clone(),
                x,
                y,
                width: ed.svek_w,
                height: ed.svek_h,
                text_x: x + (ed.svek_w - ed.text_w) / 2.0,
                text_y: y + STATE_TEXT_DY,
                text_width: ed.text_w,
                source_line: entity.source_line,
            })
        })
        .collect();

    let node_pos: HashMap<&str, (f64, f64)> = nodes
        .iter()
        .map(|n| (n.display.as_str(), (n.x, n.y)))
        .collect();
    let edges = inner_links
        .iter()
        .enumerate()
        .map(|(ei, link)| {
            let color = first_color + ei as i32;
            let pts = dot
                .edges
                .get(&color)
                .map(|e| crate::cuca_layout::shift_points(&e.points, dx, dy))
                .unwrap_or_default();
            let label_anchor = link.label.as_ref().map(|_| {
                let left = dot.label_box_left.get(&color).copied().unwrap_or(0.0);
                let from_y = node_pos.get(link.from.as_str()).map_or(0.0, |p| p.1);
                let to_y = node_pos.get(link.to.as_str()).map_or(0.0, |p| p.1);
                let gap_center = (from_y + 50.0 + to_y) / 2.0;
                (left + dx + 1.0, gap_center + LABEL_BASELINE_OFFSET)
            });
            let path_id = format!("{}-to-{}", link.from, link.to);
            FoldEdge {
                link_id: ids.link_id(link.source_line),
                path_id,
                points: pts,
                label: link.label.clone(),
                label_anchor,
                source_line: link.source_line,
                from_entity: ids.entity_id(&format!("{group_key}.{}", link.from)),
                to_entity: ids.entity_id(&format!("{group_key}.{}", link.to)),
                comment: format!("link {} to {}", link.from, link.to),
            }
        })
        .collect();

    // Inner image size. Content min is pinned to 6 by the move-delta; SvekResult
    // reports `getDimension().delta(15, 15)`. The x extent is set by the widest
    // edge-label spacer (`label-anchor x + measured text + 1`), the y extent by
    // the lowest inner node; edges stay inside the node column.
    let mut max_spacer = nodes_iter_max(&nodes, |n| n.x + n.width);
    for (ei, link) in inner_links.iter().enumerate() {
        if let Some(text) = &link.label {
            let color = first_color + ei as i32;
            let left = dot.label_box_left.get(&color).copied().unwrap_or(0.0);
            let measured = crate::cuca_layout::measure_text_size_pub(text, LABEL_FONT_SIZE_INNER);
            // The label is rendered with a 3-decimal `textLength`; the image
            // spacer sizes from that rounded width, not the raw measure.
            let text_w = (measured * 1000.0).round() / 1000.0;
            let right = left + dx + 1.0 + text_w + 1.0;
            max_spacer = max_spacer.max(right);
        }
    }
    let max_y = nodes_iter_max(&nodes, |n| n.y + n.height - 1.0);
    let im_w = (max_spacer - 6.0) + 15.0;
    // SvekResult y max records the rounded-rect far edge at y+height−1.
    let im_h = (max_y - 6.0) + 15.0;

    let name_height = STATE_NAME_H;
    let title_w = crate::cuca_layout::measure_text_size_pub(group_key, FONT_SIZE_NAME_INNER);
    // InnerStateAutonom reserves MARGIN+MARGIN_LINE (10) on each horizontal
    // side around the wider of the title and the inner image.
    let wrap_x = 2.0 * (MARGIN + MARGIN_LINE);
    let width = im_w.max(title_w) + wrap_x;
    // Vertical: title band at top, then a MARGIN+MARGIN_LINE (10) below im.
    let height = name_height + im_h + (MARGIN + MARGIN_LINE);

    FoldFragment {
        name: group_key.to_string(),
        width,
        height,
        name_height,
        nodes,
        edges,
    }
}

fn nodes_iter_max<F>(nodes: &[FoldNode], f: F) -> f64
where
    F: Fn(&FoldNode) -> f64,
{
    nodes.iter().map(f).fold(f64::NEG_INFINITY, f64::max)
}
/// Folds every autarkic state group.
#[must_use]
pub fn fold_autarkic_states(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
) -> FoldResult {
    let ids = assign_global_ids(entities, links);

    let mut group_keys: Vec<&String> =
        entities.values().filter(|e| e.group).map(|e| &e.name).collect();
    group_keys.sort_by_key(|name| entities.get(*name).map_or(0, |e| e.source_line));

    let mut out_entities = entities.clone();
    let mut fragments: HashMap<String, FoldFragment> = HashMap::new();

    for group_key in group_keys {
        if !is_autarkic(group_key, entities, links) {
            continue;
        }
        let (inner_entities, inner_links) = build_inner(group_key, entities, links);
        let fragment = build_fragment(group_key, &inner_entities, &inner_links, &ids);

        // Inner leaves disappear from the outer graph.
        let prefix = format!("{group_key}.");
        for key in inner_entities.keys() {
            out_entities.remove(&format!("{prefix}{key}"));
        }
        // The group becomes an ordinary state leaf sized to the fragment.
        let (frag_w, frag_h) = (fragment.width, fragment.height);
        if let Some(g) = out_entities.get_mut(group_key) {
            g.group = false;
            g.kind = EntityKind::State;
            g.forced_size = Some((frag_w, frag_h));
        }
        fragments.insert(group_key.clone(), fragment);
    }

    // Drop pure-inner links of every folded group; keep links that touch the
    // outer leaf (start/end transitions).
    let inner_prefixes: Vec<String> = fragments.keys().map(|k| format!("{k}.")).collect();
    let is_inner = |resolved: &str| inner_prefixes.iter().any(|p| resolved.starts_with(p.as_str()));
    let mut out_links = Vec::new();
    for link in links {
        if is_inner(&link.from) || is_inner(&link.to) {
            continue;
        }
        out_links.push(link.clone());

    }

    FoldResult {
        entities: out_entities,
        links: out_links,
        fragments,
        ids,
    }
}


/// Resolves written endpoints, then folds autarkic state groups. Returns the
/// fold result only for a STATE diagram that actually has a foldable group;
/// otherwise `None`, leaving the caller on the ordinary layout path.
#[must_use]
pub fn prepare(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
    diagram_type: plantuml_core::DiagramType,
) -> Option<FoldResult> {
    if diagram_type != plantuml_core::DiagramType::State {
        return None;
    }
    // Resolve written bracket-form endpoints (`[*]`) to qualified keys, using
    // source order for stable lookup.
    let mut by_source: Vec<(&String, &ParsedEntity)> = entities.iter().collect();
    by_source.sort_by_key(|(_, e)| e.source_line);
    let ordered: IndexMap<String, ParsedEntity> = by_source
        .iter()
        .map(|(name, entity)| ((**name).clone(), (**entity).clone()))
        .collect();
    let resolved: Vec<ParsedLink> = links
        .iter()
        .map(|link| ParsedLink {
            from: crate::entity_link_parser::resolve_endpoint(&link.from, true, &ordered),
            to: crate::entity_link_parser::resolve_endpoint(&link.to, false, &ordered),
            arrow: link.arrow.clone(),
            label: link.label.clone(),
            source_line: link.source_line,
        })
        .collect();

    let has_foldable = entities.values().any(|e| e.group && e.kind == EntityKind::State);
    if !has_foldable {
        return None;
    }
    let result = fold_autarkic_states(entities, &resolved);
    if result.fragments.is_empty() {
        None
    } else {
        Some(result)
    }
}
/// Produces the outer layout for a folded STATE diagram, with every node and
/// link relabelled from the global creation-order id map, together with the
/// nested fragments the renderer draws inside the folded leaves.
#[must_use]
pub fn layout_with_fold(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
) -> Option<(CucaLayout, HashMap<String, FoldFragment>)> {
    let fold = prepare(entities, links, plantuml_core::DiagramType::State)?;
    let mut layout = compute_layout_ranked(
        &fold.entities,
        &fold.links,
        plantuml_core::DiagramType::State,
        35.0,
        60.0,
        true,
    );
    // Relabel node ids from the global creation-order map (keyed by name).
    for node in &mut layout.nodes {
        if let Some(&uid) = fold.ids.entity.get(&node.name) {
            node.entity_id = format!("ent{uid:04}");
        }
    }
    // Relabel each link's id and endpoint node ids from the global map. The
    // layout assigns local ids per subgraph; the outer links reference the
    // whole-diagram creation-order ids (start=ent0006, ...).
    for link in &mut layout.links {
        if let Some(&uid) = fold.ids.link.get(&link.source_line) {
            link.link_id = format!("lnk{uid}");
        }
        let relabel = |endpoint: &str| {
            fold.ids.entity.get(endpoint).map(|uid| format!("ent{uid:04}"))
        };
        if let Some(id) = relabel(&link.from) {
            link.from_id = id;
        }
        if let Some(id) = relabel(&link.to) {
            link.to_id = id;
        }
    }
    Some((layout, fold.fragments))
}

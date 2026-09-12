//! Simplified layout engine for CucaDiagram types.
//!
//! Ported from: `net/sourceforge/plantuml/sdot/CucaDiagramFileMakerSmetana.java`.
//!
//! The Java Smetana engine uses rank-based graph layout with edge routing.
//! This Rust implementation uses a simpler grid-based layout:
//! entities are positioned in a grid, with rows determined by topological
//! ordering of the link graph.

use std::collections::{HashMap, HashSet};

use crate::entity_link_parser::{ParsedEntity, ParsedLink};

/// A positioned entity in the layout.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// Entity name.
    pub name: String,
    /// Display label.
    pub display: String,
    /// X position (top-left corner).
    pub x: f64,
    /// Y position (top-left corner).
    pub y: f64,
    /// Box width.
    pub width: f64,
    /// Box height.
    pub height: f64,
    /// Center X.
    pub center_x: f64,
    /// Center Y.
    pub center_y: f64,
    /// Row index in the grid.
    pub row: usize,
    /// Column index in the grid.
    pub col: usize,
}

/// A positioned link between two layout nodes.
#[derive(Debug, Clone)]
pub struct LayoutLink {
    /// Source node name.
    pub from: String,
    /// Target node name.
    pub to: String,
    /// Start point (x, y) — edge of source box.
    pub start: (f64, f64),
    /// End point (x, y) — edge of target box.
    pub end: (f64, f64),
    /// Optional label.
    pub label: Option<String>,
    /// Arrow direction.
    pub has_arrow: bool,
}

/// Computed layout for a CucaDiagram.
#[derive(Debug, Clone)]
pub struct CucaLayout {
    /// Positioned entity nodes.
    pub nodes: Vec<LayoutNode>,
    /// Positioned links.
    pub links: Vec<LayoutLink>,
    /// Total width.
    pub total_width: f64,
    /// Total height.
    pub total_height: f64,
}

// ── Layout constants ─────────────────────────────────────────────────────

/// Horizontal padding inside entity boxes.
const BOX_PADDING_H: f64 = 10.0;
/// Vertical padding inside entity boxes.
const BOX_PADDING_V: f64 = 6.0;
/// Horizontal gap between columns.
const COL_GAP: f64 = 40.0;
/// Vertical gap between rows.
const ROW_GAP: f64 = 50.0;
/// Minimum box width.
const MIN_BOX_WIDTH: f64 = 80.0;
/// Line height for body text.
const LINE_HEIGHT: f64 = 16.0;
/// Page margin.
const PAGE_MARGIN: f64 = 10.0;
/// Font size for entity names.
const FONT_SIZE: i32 = 12;

/// Approximate text width.
fn text_width(text: &str) -> f64 {
    (FONT_SIZE as f64) * 0.6 * text.chars().count() as f64
}

/// Computes the layout for a set of entities and links.
///
/// Uses a topological-sort-based row assignment: entities with no incoming
/// links go in row 0, entities whose dependencies are all in earlier rows
/// go in the next available row.
#[must_use]
pub fn compute_layout(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
) -> CucaLayout {
    if entities.is_empty() && !links.iter().any(|l| l.from == "[*]" || l.to == "[*]") {
        return CucaLayout {
            nodes: Vec::new(),
            links: Vec::new(),
            total_width: PAGE_MARGIN * 2.0,
            total_height: PAGE_MARGIN * 2.0,
        };
    }

    // Auto-create [*] pseudo-entities for initial/final state transitions.
    let mut entities = entities.clone();
    for link in links {
        if link.from == "[*]" && !entities.contains_key("[*]") {
            entities.insert("[*]".to_string(), ParsedEntity {
                name: "[*]".to_string(),
                display: "[*]".to_string(),
                kind: crate::entity_link_parser::EntityKind::State,
                stereotype: None,
                body: Vec::new(),
            });
        }
        if link.to == "[*]" && !entities.contains_key("[*]") {
            entities.insert("[*]".to_string(), ParsedEntity {
                name: "[*]".to_string(),
                display: "[*]".to_string(),
                kind: crate::entity_link_parser::EntityKind::State,
                stereotype: None,
                body: Vec::new(),
            });
        }
    }

    if entities.is_empty() {
        return CucaLayout {
            nodes: Vec::new(),
            links: Vec::new(),
            total_width: PAGE_MARGIN * 2.0,
            total_height: PAGE_MARGIN * 2.0,
        };
    }

    // Build adjacency: for each entity, which entities it depends on (incoming).
    let entity_names: HashSet<&String> = entities.keys().collect();
    let mut incoming: HashMap<&String, HashSet<&String>> = HashMap::new();
    let mut outgoing: HashMap<&String, HashSet<&String>> = HashMap::new();

    for link in links {
        if entity_names.contains(&link.from) && entity_names.contains(&link.to) {
            incoming.entry(&link.to).or_default().insert(&link.from);
            outgoing.entry(&link.from).or_default().insert(&link.to);
        }
    }

    // Topological sort with row assignment.
    let mut row_of: HashMap<&String, usize> = HashMap::new();
    let mut remaining: HashSet<&String> = entity_names.iter().copied().collect();
    let mut current_row = 0;

    while !remaining.is_empty() {
        let mut ready: Vec<&String> = remaining
            .iter()
            .filter(|name| {
                incoming
                    .get(*name)
                    .is_none_or(|deps| deps.iter().all(|dep| row_of.contains_key(*dep)))
            })
            .copied()
            .collect();

        if ready.is_empty() {
            // Cycle detected — assign remaining to current row.
            ready = remaining.iter().copied().collect();
        }

        // Sort ready entities by name for deterministic layout.
        ready.sort();

        for name in &ready {
            row_of.insert(*name, current_row);
            remaining.remove(name);
        }
        current_row += 1;
    }

    // Assign columns within each row.
    let mut col_of: HashMap<&String, usize> = HashMap::new();
    let mut row_cols: HashMap<usize, Vec<&String>> = HashMap::new();

    for (name, &row) in &row_of {
        row_cols.entry(row).or_default().push(name);
    }

    for names in row_cols.values_mut() {
        names.sort();
        for (col, name) in names.iter().enumerate() {
            col_of.insert(*name, col);
        }
    }

    // Compute box dimensions for each entity.
    let mut node_dims: HashMap<&String, (f64, f64)> = HashMap::new();
    for (name, entity) in &entities {
        if name.as_str() == "[*]" {
            // [*] is rendered as a small circle, not a box.
            node_dims.insert(name, (20.0, 20.0));
        } else {
            let label_w = text_width(&entity.display);
            let body_lines = entity.body.len();
            let width = label_w.max(MIN_BOX_WIDTH) + BOX_PADDING_H * 2.0;
            let height = LINE_HEIGHT + BOX_PADDING_V * 2.0 + body_lines as f64 * LINE_HEIGHT;
            node_dims.insert(name, (width, height));
        }
    }

    // Compute column widths (max width in each column).
    let max_col = col_of.values().copied().max().unwrap_or(0);
    let mut col_widths = vec![0.0f64; max_col + 1];
    for (name, &(w, _)) in &node_dims {
        let col = col_of[name];
        if col_widths[col] < w {
            col_widths[col] = w;
        }
    }

    // Compute row heights (max height in each row).
    let max_row = row_of.values().copied().max().unwrap_or(0);
    let mut row_heights = vec![0.0f64; max_row + 1];
    for (name, &(_, h)) in &node_dims {
        let row = row_of[name];
        if row_heights[row] < h {
            row_heights[row] = h;
        }
    }

    // Compute column X offsets.
    let mut col_x = vec![PAGE_MARGIN; max_col + 1];
    for c in 1..=max_col {
        col_x[c] = col_x[c - 1] + col_widths[c - 1] + COL_GAP;
    }

    // Compute row Y offsets.
    let mut row_y = vec![PAGE_MARGIN; max_row + 1];
    for r in 1..=max_row {
        row_y[r] = row_y[r - 1] + row_heights[r - 1] + ROW_GAP;
    }

    // Create layout nodes.
    let mut nodes = Vec::new();
    for (name, entity) in &entities {
        let row = row_of[name];
        let col = col_of[name];
        let (w, h) = node_dims[name];
        let x = col_x[col] + (col_widths[col] - w) / 2.0;
        let y = row_y[row];

        nodes.push(LayoutNode {
            name: name.clone(),
            display: entity.display.clone(),
            x,
            y,
            width: w,
            height: h,
            center_x: x + w / 2.0,
            center_y: y + h / 2.0,
            row,
            col,
        });
    }

    // Sort nodes by row then col for deterministic order.
    nodes.sort_by(|a, b| (a.row, a.col).cmp(&(b.row, b.col)));

    // Create layout links.
    let node_map: HashMap<&String, &LayoutNode> = nodes.iter().map(|n| (&n.name, n)).collect();
    let mut layout_links = Vec::new();
    for link in links {
        if let (Some(from_node), Some(to_node)) = (node_map.get(&link.from), node_map.get(&link.to)) {
            // Compute start/end points at box edges.
            let (start, end) = compute_edge_points(from_node, to_node);
            layout_links.push(LayoutLink {
                from: link.from.clone(),
                to: link.to.clone(),
                start,
                end,
                label: link.label.clone(),
                has_arrow: link.direction != crate::entity_link_parser::LinkDirection::None,
            });
        }
    }

    // Compute total dimensions.
    let total_width = col_x.last().map_or(PAGE_MARGIN * 2.0, |&x| x + col_widths.last().unwrap_or(&0.0) + PAGE_MARGIN);
    let total_height = row_y.last().map_or(PAGE_MARGIN * 2.0, |&y| y + row_heights.last().unwrap_or(&0.0) + PAGE_MARGIN);

    CucaLayout {
        nodes,
        links: layout_links,
        total_width,
        total_height,
    }
}

/// Computes the edge connection points between two boxes.
fn compute_edge_points(from: &LayoutNode, to: &LayoutNode) -> ((f64, f64), (f64, f64)) {
    // Determine which edges to connect based on relative positions.
    let dx = to.center_x - from.center_x;
    let dy = to.center_y - from.center_y;

    if dy.abs() > dx.abs() {
        // Vertical connection: top/bottom edges.
        if dy > 0.0 {
            // From is above to.
            ((from.center_x, from.y + from.height), (to.center_x, to.y))
        } else {
            // From is below to.
            ((from.center_x, from.y), (to.center_x, to.y + to.height))
        }
    } else {
        // Horizontal connection: left/right edges.
        if dx > 0.0 {
            // From is left of to.
            ((from.x + from.width, from.center_y), (to.x, to.center_y))
        } else {
            // From is right of to.
            ((from.x, from.center_y), (to.x + to.width, to.center_y))
        }
    }
}

//! Layout engine for CucaDiagram types using external `dot` (Graphviz).
//!
//! Ported from: `net/sourceforge/plantuml/svek/CucaDiagramFileMakerSvek.java`.
//!
//! The Java Svek engine generates a dot string, runs the external `dot`
//! binary, and parses the resulting SVG for node positions and edge spline
//! paths. This Rust implementation replicates that pipeline for exact
//! behavioral parity with Java PlantUML 1.2026.6.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::process::{Command, Stdio};

use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::{FontStyle, UFont};
use plantuml_klimt::string_bounder_svg::StringBounderSvg;

use crate::entity_link_parser::{EntityKind, ParsedEntity, ParsedLink};

// ── Layout constants ─────────────────────────────────────────────────────

/// Horizontal gap between same-rank nodes in pixels (DotStringFactory.getMinNodeSep = 35).
const NODESEP_PX: f64 = 35.0;
/// Vertical gap between ranks in pixels (DotStringFactory.getMinRankSep = 60).
const RANKSEP_PX: f64 = 60.0;
/// Canvas margin for dot layout (Java SvekResult shifts content to (7, 7)).
const CANVAS_MARGIN: f64 = 7.0;
/// Canvas margin for fallback layout (use cases, etc.).
const FALLBACK_MARGIN: f64 = 6.0;
/// Font size for entity labels.
const FONT_SIZE: i32 = 14;
/// Actor stickman width.
const ACTOR_WIDTH: f64 = 27.0;
/// Actor stickman height.
const ACTOR_HEIGHT: f64 = 60.0;
/// Text height = font_size * 1.361994 (Java AWT FontMetrics.getStringBounds:
/// 19.067917 at 14pt). Slightly less than 1.362, affects clamped-alpha ellipses.
const TEXT_HEIGHT: f64 = 14.0 * 1.361_994;

// ── Structs ─────────────────────────────────────────────────────────────

/// A positioned entity in the layout.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// Entity name.
    pub name: String,
    /// Display label.
    pub display: String,
    /// Entity kind (actor, usecase, etc.).
    pub kind: EntityKind,
    /// Source line number.
    pub source_line: usize,
    /// Entity ID (ent0001, ent0002, ...).
    pub entity_id: String,
    /// Top-left X corner.
    pub x: f64,
    /// Top-left Y corner.
    pub y: f64,
    /// SvekNode width.
    pub width: f64,
    /// SvekNode height.
    pub height: f64,
    /// Center X (head center for actor, ellipse center for usecase).
    pub center_x: f64,
    /// Center Y (head center for actor, ellipse center for usecase).
    pub center_y: f64,
    /// Ellipse rx (usecase only).
    pub rx: f64,
    /// Ellipse ry (usecase only).
    pub ry: f64,
    /// Text X position.
    pub text_x: f64,
    /// Text Y position (baseline).
    pub text_y: f64,
    /// Text width (for textLength attribute).
    pub text_width: f64,
    /// Rank in the layout.
    pub rank: usize,
}

/// A positioned link in the layout.
#[derive(Debug, Clone)]
pub struct LayoutLink {
    /// Source entity name.
    pub from: String,
    /// Target entity name.
    pub to: String,
    /// Source entity ID.
    pub from_id: String,
    /// Target entity ID.
    pub to_id: String,
    /// Link ID (lnk4, lnk5, ...).
    pub link_id: String,
    /// Link type (always "dependency" for use case diagrams).
    pub link_type: String,
    /// Path ID (e.g. "User-to-Login").
    pub path_id: String,
    /// Source line number.
    pub source_line: usize,
    /// Bezier path "d" attribute.
    pub path_d: String,
    /// Arrow polygon points string.
    pub arrow_points: String,
    /// Whether this link is a simple straight line (class/state diagrams).
    pub is_line: bool,
    /// Line start point (when is_line is true).
    pub start: (f64, f64),
    /// Line end point (when is_line is true).
    pub end: (f64, f64),
    /// Optional label text displayed on the link.
    pub label: Option<String>,
    /// Arrow type string (e.g. "-->", "<|--", "<|..").
    pub arrow: String,
}

/// Computed layout for a CucaDiagram.
#[derive(Debug, Clone)]
pub struct CucaLayout {
    /// Positioned entity nodes.
    pub nodes: Vec<LayoutNode>,
    /// Positioned links.
    pub links: Vec<LayoutLink>,
    /// Total SVG width.
    pub total_width: f64,
    /// Total SVG height.
    pub total_height: f64,
}

// ── Text measurement ────────────────────────────────────────────────────

/// Measures text width using StringBounderSvg (matches Java AWT).
fn measure_text(text: &str) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(FONT_SIZE);
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

/// Measures text width with italic font (for interface/abstract entity names).
fn measure_text_italic(text: &str) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(FONT_SIZE).with_style(FontStyle::italic());
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

// ── Entity sizing ───────────────────────────────────────────────────────

/// Computes usecase ellipse rx, ry from text width.
///
/// Ported from: `TextBlockInEllipse.java` and `ContainingEllipse.java`.
fn usecase_ellipse(text_w: f64) -> (f64, f64) {
    let text_h = TEXT_HEIGHT;
    let alpha = (text_h / text_w).clamp(0.2, 0.8);
    let y_range = text_h / alpha;
    let x_range = text_w;
    let r = 0.5 * x_range.hypot(y_range);
    let rx = r + 3.0;
    let ry = r * alpha + 3.0;
    (rx, ry)
}

/// Computes actor SvekNode dimensions.
fn actor_dimensions(text_w: f64) -> (f64, f64) {
    let width = ACTOR_WIDTH.max(text_w);
    let height = ACTOR_HEIGHT + TEXT_HEIGHT;
    (width, height)
}

// ── Dot string generation ───────────────────────────────────────────────

/// Generates the dot string matching Java's `DotStringFactory.createDotString()`.
///
/// Format: `digraph unix { nodesep=...; ranksep=...; ... }`
fn generate_dot_string(
    sorted_entities: &[(&String, &ParsedEntity)],
    entity_data: &HashMap<&String, EntityData>,
    links: &[ParsedLink],
) -> (String, HashMap<String, i32>) {
    let nodesep_in = NODESEP_PX / 72.0;
    let ranksep_in = RANKSEP_PX / 72.0;

    let mut sb = String::new();
    writeln!(sb, "digraph unix {{").unwrap();
    writeln!(sb, "nodesep={nodesep_in:.6};").unwrap();
    writeln!(sb, "ranksep={ranksep_in:.6};").unwrap();
    writeln!(sb, "remincross=true;").unwrap();
    writeln!(sb, "searchsize=500;").unwrap();

    // Node definitions — color sequence starts at 6 (matching Java ColorSequence)
    let mut color = 6i32;
    let mut node_colors: HashMap<String, i32> = HashMap::new();

    for (name, entity) in sorted_entities {
        let ed = entity_data.get(*name).unwrap();
        let shape = match entity.kind {
            EntityKind::Actor => "rect",
            EntityKind::Usecase => "ellipse",
            _ => "box",
        };
        // Round to 4 decimals before converting to inches: the external dot
        // binary is sensitive to sub-pixel width differences.  Rust's text
        // measurement produces floats like 36.469844… that round to 36.4698
        // in the SVG (3-decimal cleaner) but differ from Java's exact 36.4698
        // in the 5th decimal place, shifting dot positions by 0.01px.
        let width_in = (ed.svek_w * 10000.0).round() / 10000.0 / 72.0;
        let height_in = (ed.svek_h * 10000.0).round() / 10000.0 / 72.0;
        writeln!(
            sb,
            "sh{color:04} [shape={shape},label=\"\",width={width_in:.6},height={height_in:.6},color=\"#{color:06x}\"];"
        ).unwrap();
        node_colors.insert((*name).clone(), color);
        color += 1;
    }

    // Edge definitions — sequential colors after nodes
    for link in links {
        if let (Some(&from_color), Some(&to_color)) =
            (node_colors.get(&link.from), node_colors.get(&link.to))
        {
            writeln!(
                sb,
                "sh{from_color:04}->sh{to_color:04}[arrowtail=none,arrowhead=none,minlen=1,color=\"#{color:06x}\"];"
            ).unwrap();
            color += 1;
        }
    }

    // Add invisible edges for entities without any real edges.
    // Smetana's internal Graphviz splits unconnected nodes into multiple ranks
    // (roughly half per rank). We replicate this by adding invisible edges
    // from the first half to the second half.
    let entities_with_edges: std::collections::HashSet<&String> = links
        .iter()
        .flat_map(|l| [&l.from, &l.to])
        .collect();
    let unconnected: Vec<&String> = sorted_entities
        .iter()
        .filter_map(|(name, _)| {
            (!entities_with_edges.contains(name)).then_some(*name)
        })
        .collect();
    if unconnected.len() > 2 {
        let split_point = unconnected.len().div_ceil(2);
        for i in 0..split_point.min(unconnected.len() - split_point) {
            let from_name = unconnected[i];
            let to_name = unconnected[split_point + i];
            if let (Some(&from_color), Some(&to_color)) =
                (node_colors.get(from_name), node_colors.get(to_name))
            {
                writeln!(
                    sb,
                    "sh{from_color:04}->sh{to_color:04}[style=invis];"
                ).unwrap();
            }
        }
    }

    writeln!(sb).unwrap();
    writeln!(sb, "}}").unwrap();
    (sb, node_colors)
}

// ── Dot binary execution ────────────────────────────────────────────────

/// Runs the `dot -Tsvg` binary with the given dot string, returns SVG output.
fn run_dot(dot_string: &str) -> Option<String> {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(dot_string.as_bytes()).ok()?;
    }
    // Drop stdin to signal EOF
    drop(child.stdin.take());

    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        None
    }
}

// ── Dot SVG parsing ─────────────────────────────────────────────────────

/// Parsed node position from dot SVG (after Y-flip).
#[derive(Debug, Clone)]
struct DotNodePos {
    min_x: f64,
    min_y: f64,
    #[allow(dead_code)]
    width: f64,
    #[allow(dead_code)]
    height: f64,
}

/// Parsed edge path from dot SVG (after Y-flip). Single cubic bezier: start, ctrl1, ctrl2, end.
#[derive(Debug, Clone)]
struct DotEdgePath {
    points: [(f64, f64); 4],
}

/// Result of parsing the dot SVG output.
struct DotSvgResult {
    #[allow(dead_code)]
    full_height: f64,
    #[allow(dead_code)]
    svg_width: f64,
    #[allow(dead_code)]
    svg_height: f64,
    nodes: HashMap<i32, DotNodePos>,
    edges: HashMap<i32, DotEdgePath>,
}

/// Parses the dot SVG output to extract node positions and edge paths.
///
/// The dot SVG has:
/// - `<svg width="Wpt" height="Hpt" ...>` — graph dimensions
/// - `<polygon ... points="x1,y1 x2,y2 ...">` — rect nodes (by color)
/// - `<ellipse ... cx="X" cy="Y" rx="RX" ry="RY">` — ellipse nodes (by color)
/// - `<path ... stroke="#XXXXXX" d="M x,y C x1,y1 x2,y2 x,y">` — edges (by stroke color)
fn parse_dot_svg(svg: &str, node_colors: &HashMap<String, i32>) -> Option<DotSvgResult> {
    // Extract fullHeight from <svg width="Wpt" height="Hpt"
    let svg_height = extract_attr_f64(svg, "svg", "height")?;
    let svg_width = extract_attr_f64(svg, "svg", "width")?;
    let full_height = svg_height;

    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();

    // Parse nodes by color
    for (name, &color) in node_colors {
        let color_str = format!("#{color:06x}");
        // Find element with this color
        if let Some(pos) = svg.find(&color_str) {
            // Look backwards for the element tag start
            let tag_start = svg[..pos].rfind('<').unwrap_or(0);
            let tag_end = svg[pos..].find('>').map_or(pos, |i| pos + i);
            let element = &svg[tag_start..=tag_end];

            if element.contains("ellipse") {
                // Ellipse node: extract cx, cy, rx, ry
                let cx = extract_attr_f64_in(element, "cx")?;
                let cy = extract_attr_f64_in(element, "cy")?;
                let rx = extract_attr_f64_in(element, "rx")?;
                let ry = extract_attr_f64_in(element, "ry")?;
                let min_x = cx - rx;
                let min_y = cy + full_height - ry;
                nodes.insert(
                    color,
                    DotNodePos {
                        min_x,
                        min_y,
                        width: 2.0 * rx,
                        height: 2.0 * ry,
                    },
                );
            } else if element.contains("polygon") {
                // Rect node: extract polygon points
                let points_str = extract_attr_str_in(element, "points")?;
                let pts = parse_points(&points_str);
                if pts.len() >= 2 {
                    let min_x = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
                    let min_y_dot = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
                    let max_x = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
                    let max_y_dot = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
                    let min_y = min_y_dot + full_height;
                    let width = max_x - min_x;
                    let height = max_y_dot - min_y_dot;
                    nodes.insert(
                        color,
                        DotNodePos {
                            min_x,
                            min_y,
                            width,
                            height,
                        },
                    );
                }
            }
        }
        let _ = name; // name not used for parsing
    }

    // Parse edges by stroke color
    // Edge colors are sequential after node colors
    let max_node_color = node_colors.values().copied().max().unwrap_or(5);
    for edge_color in (max_node_color + 1)..(max_node_color + 100) {
        let color_str = format!("#{edge_color:06x}");
        // Find path element with this stroke color
        let search = format!("stroke=\"{color_str}\"");
        if let Some(pos) = svg.find(&search) {
            // Find the d="..." attribute in the same element
            let tag_start = svg[..pos].rfind('<').unwrap_or(0);
            let tag_end = svg[pos..].find('>').map_or(pos, |i| pos + i);
            let element = &svg[tag_start..=tag_end];

            if let Some(d_str) = extract_attr_str_in(element, "d") {
                if let Some(path) = parse_bezier_path(&d_str, full_height) {
                    edges.insert(edge_color, path);
                }
            }
        }
    }

    Some(DotSvgResult {
        full_height,
        svg_width,
        svg_height,
        nodes,
        edges,
    })
}

/// Extracts a float attribute value from an XML element string.
fn extract_attr_f64_in(element: &str, attr: &str) -> Option<f64> {
    let pattern = format!("{attr}=\"");
    let start = element.find(&pattern)? + pattern.len();
    let end = element[start..].find('"')?;
    let val = &element[start..start + end];
    // Strip "pt" suffix if present
    let val = val.trim_end_matches("pt");
    val.parse().ok()
}

/// Extracts a string attribute value from an XML element string.
fn extract_attr_str_in(element: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = element.find(&pattern)? + pattern.len();
    let end = element[start..].find('"')?;
    Some(element[start..start + end].to_string())
}

/// Extracts a float attribute from the first occurrence of a tag in the SVG.
fn extract_attr_f64(svg: &str, tag: &str, attr: &str) -> Option<f64> {
    let pattern = format!("<{tag}");
    let start = svg.find(&pattern)?;
    let end = svg[start..].find('>')?;
    let element = &svg[start..=start + end];
    extract_attr_f64_in(element, attr)
}

/// Parses "x1,y1 x2,y2 ..." into a list of (x, y) points.
fn parse_points(s: &str) -> Vec<(f64, f64)> {
    s.split_whitespace()
        .filter_map(|pair| {
            let mut parts = pair.split(',');
            let x: f64 = parts.next()?.parse().ok()?;
            let y: f64 = parts.next()?.parse().ok()?;
            Some((x, y))
        })
        .collect()
}

/// Parses "M x,y C x1,y1 x2,y2 x,y" into a cubic bezier (4 points).
/// Applies Y-flip: y → y + full_height.
fn parse_bezier_path(d: &str, full_height: f64) -> Option<DotEdgePath> {
    // Expected format: M x,yC x1,y1 x2,y2 x,y (or with spaces)
    // dot SVG uses: M x,yC x1,y1 x2,y2 x,y (no space after M, C has space-separated points)
    let d = d.trim();

    // Find M and C
    let m_pos = d.find('M')?;
    let c_pos = d.find('C')?;

    let m_part = &d[m_pos + 1..c_pos];
    let c_part = &d[c_pos + 1..];

    // Parse M point
    let m_str = m_part.trim();
    let (mx, my) = parse_point(m_str)?;
    let start = (mx, my + full_height);

    // Parse C points (3 points: ctrl1, ctrl2, end)
    let c_points: Vec<(f64, f64)> = c_part
        .split_whitespace()
        .filter_map(|pair| {
            let mut parts = pair.split(',');
            let x: f64 = parts.next()?.parse().ok()?;
            let y: f64 = parts.next()?.parse().ok()?;
            Some((x, y + full_height))
        })
        .collect();

    if c_points.len() < 3 {
        return None;
    }

    Some(DotEdgePath {
        points: [start, c_points[0], c_points[1], c_points[2]],
    })
}

/// Parses "x,y" into (x, y).
fn parse_point(s: &str) -> Option<(f64, f64)> {
    let s = s.trim();
    let mut parts = s.split(',');
    let x: f64 = parts.next()?.parse().ok()?;
    let y: f64 = parts.next()?.parse().ok()?;
    Some((x, y))
}

// ── Layout computation ─────────────────────────────────────────────────

/// Computes the layout for a set of entities and links.
#[must_use]
pub fn compute_layout(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
    diagram_type: plantuml_core::DiagramType,
) -> CucaLayout {
    if entities.is_empty() && !links.iter().any(|l| l.from == "[*]" || l.to == "[*]") {
        return CucaLayout {
            nodes: Vec::new(),
            links: Vec::new(),
            total_width: CANVAS_MARGIN * 2.0,
            total_height: CANVAS_MARGIN * 2.0,
        };
    }

    // Auto-create [*] pseudo-entities for initial/final state transitions.
    let mut entities = entities.clone();
    for link in links {
        if link.from == "[*]" && !entities.contains_key("[*]") {
            entities.insert(
                "[*]".to_string(),
                ParsedEntity {
                    name: "[*]".to_string(),
                    display: "[*]".to_string(),
                    kind: EntityKind::State,
                    stereotype: None,
                    body: Vec::new(),
                    source_line: 0,
                },
            );
        }
        if link.to == "[*]" && !entities.contains_key("[*]") {
            entities.insert(
                "[*]".to_string(),
                ParsedEntity {
                    name: "[*]".to_string(),
                    display: "[*]".to_string(),
                    kind: EntityKind::State,
                    stereotype: None,
                    body: Vec::new(),
                    source_line: 0,
                },
            );
        }
    }

    if entities.is_empty() {
        return CucaLayout {
            nodes: Vec::new(),
            links: Vec::new(),
            total_width: CANVAS_MARGIN * 2.0,
            total_height: CANVAS_MARGIN * 2.0,
        };
    }

    // Sort entities by source_line for correct entity ID assignment
    let mut sorted_entities: Vec<(&String, &ParsedEntity)> = entities.iter().collect();
    sorted_entities.sort_by_key(|(_, e)| e.source_line);

    // Compute entity dimensions and SvekNode dimensions.
    let mut entity_data: HashMap<&String, EntityData> = HashMap::new();
    for (name, entity) in &entities {
        let text_w = match entity.kind {
            EntityKind::Interface | EntityKind::Abstract => measure_text_italic(&entity.display),
            _ => measure_text(&entity.display),
        };
        let (svek_w, svek_h, rx, ry) = match entity.kind {
            EntityKind::Actor => {
                let (w, h) = actor_dimensions(text_w);
                (w, h, 8.0, 8.0)
            }
            EntityKind::Usecase => {
                let (rx, ry) = usecase_ellipse(text_w);
                (2.0 * rx, 2.0 * ry, rx, ry)
            }
            _ => {
                // Class/interface/abstract: width = max(name_text + 32, body_text + 26)
                // Height = 32 + num_fields * LINE_HEIGHT + 8 + num_methods * LINE_HEIGHT + 8
                let name_w = text_w + 32.0;
                let mut max_body_w = 0.0_f64;
                let mut num_fields = 0usize;
                let mut num_methods = 0usize;
                for line in &entity.body {
                    let trimmed = line.trim();
                    let rest = match trimmed.chars().next() {
                        Some('+' | '-' | '#') => trimmed[1..].trim(),
                        _ => trimmed,
                    };
                    let body_w = measure_text(rest) + 26.0;
                    max_body_w = max_body_w.max(body_w);
                    if rest.contains('(') {
                        num_methods += 1;
                    } else {
                        num_fields += 1;
                    }
                }
                let w = name_w.max(max_body_w);
                let line_h = 14.0 * 1.361_994;
                let h = 32.0 + num_fields as f64 * line_h + 8.0 + num_methods as f64 * line_h + 8.0;
                (w, h, 0.0, 0.0)
            }
        };
        entity_data.insert(
            name,
            EntityData {
                text_w,
                svek_w,
                svek_h,
                rx,
                ry,
            },
        );
    }

    // Build adjacency for rank assignment (needed for LayoutNode.rank field).
    let entity_names: HashSet<&String> = entities.keys().collect();
    let mut incoming: HashMap<&String, HashSet<&String>> = HashMap::new();
    let mut outgoing: HashMap<&String, HashSet<&String>> = HashMap::new();

    for link in links {
        if entity_names.contains(&link.from) && entity_names.contains(&link.to) {
            incoming.entry(&link.to).or_default().insert(&link.from);
            outgoing.entry(&link.from).or_default().insert(&link.to);
        }
    }

    // Topological sort with rank assignment.
    let mut rank_of: HashMap<&String, usize> = HashMap::new();
    let mut remaining: HashSet<&String> = entity_names.iter().copied().collect();
    let mut current_rank = 0;

    while !remaining.is_empty() {
        let mut ready: Vec<&String> = remaining
            .iter()
            .filter(|name| {
                incoming
                    .get(*name)
                    .is_none_or(|deps| deps.iter().all(|dep| rank_of.contains_key(*dep)))
            })
            .copied()
            .collect();

        if ready.is_empty() {
            ready = remaining.iter().copied().collect();
        }

        ready.sort();

        for name in &ready {
            rank_of.insert(*name, current_rank);
            remaining.remove(name);
        }
        current_rank += 1;
    }


    // Use dot-based layout for all diagram types (matches Java's Smetana engine).
    let (dot_string, node_colors) =
        generate_dot_string(&sorted_entities, &entity_data, links);
    let dot_svg = run_dot(&dot_string);
    if let Some(svg) = dot_svg {
        if let Some(dot_result) = parse_dot_svg(&svg, &node_colors) {
            return build_layout_from_dot(
                &sorted_entities,
                &entity_data,
                links,
                &node_colors,
                &dot_result,
                &rank_of,
                diagram_type,
            );
        }
    }

    build_fallback_layout(&entities, &entity_data, links, &rank_of)
}

#[allow(clippy::similar_names)]
fn build_layout_from_dot(
    sorted_entities: &[(&String, &ParsedEntity)],
    entity_data: &HashMap<&String, EntityData>,
    links: &[ParsedLink],
    node_colors: &HashMap<String, i32>,
    dot_result: &DotSvgResult,
    rank_of: &HashMap<&String, usize>,
    diagram_type: plantuml_core::DiagramType,
) -> CucaLayout {
    let margin = if matches!(diagram_type, plantuml_core::DiagramType::Class) {
        CANVAS_MARGIN
    } else {
        FALLBACK_MARGIN
    };


    // Compute moveDelta: (6 - min_min_x, 6 - min_min_y) over all nodes
    let min_min_x = dot_result
        .nodes
        .values()
        .map(|n| n.min_x)
        .fold(f64::INFINITY, f64::min);
    // For actor (rect) nodes, the LimitFinder's drawEllipse adds the head
    // at y=thickness()=0.5 within the SvekNode, so the effective minY is
    // node.min_y + 0.5. For usecase (ellipse) nodes, minY is as-is.
    let min_min_y = sorted_entities
        .iter()
        .filter_map(|(name, entity)| {
            let color = node_colors.get(*name).copied()?;
            let pos = dot_result.nodes.get(&color)?;
            let y = if entity.kind == EntityKind::Actor {
                pos.min_y + 0.5
            } else {
                pos.min_y
            };
            Some(y)
        })
        .fold(f64::INFINITY, f64::min);

    let dx = margin - min_min_x;
    let dy = margin - min_min_y;

    // Create layout nodes — sorted by source_line for correct entity ID order
    let mut nodes = Vec::new();
    let mut entity_counter = 0u32;

    for (name, entity) in sorted_entities {
        entity_counter += 1;
        let entity_id = format!("ent{entity_counter:04}");

        let ed = entity_data.get(*name).unwrap();
        let color = node_colors.get(*name).copied().unwrap_or(0);
        let dot_pos = dot_result.nodes.get(&color);

        let (node_x_corner, node_y_corner, cx, cy, text_x, text_y) = if let Some(pos) = dot_pos {
            let corner_x = pos.min_x + dx;
            let corner_y = pos.min_y + dy;

            match entity.kind {
                EntityKind::Actor => {
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + 8.5; // head center = top + thickness(0.5) + headDiam/2(8)
                    let text_x = corner_x + (ed.svek_w - ed.text_w) / 2.0;
                    let text_y = corner_y + ACTOR_HEIGHT + 14.9659;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                EntityKind::Usecase => {
                    let cx = corner_x + ed.rx;
                    let cy = corner_y + ed.ry;
                    let text_x = corner_x + (2.0 * ed.rx - ed.text_w) / 2.0;
                    let text_y = cy + 6.0339;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                _ => {
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + ed.svek_h / 2.0;
                    let text_x = corner_x + 10.0;
                    let text_y = corner_y + TEXT_HEIGHT;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
            }
        } else {
            // Fallback if node not found in dot output
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

        let rank = rank_of.get(*name).copied().unwrap_or(0);

        nodes.push(LayoutNode {
            name: (*name).clone(),
            display: entity.display.clone(),
            kind: entity.kind,
            source_line: entity.source_line,
            entity_id,
            x: node_x_corner,
            y: node_y_corner,
            width: ed.svek_w,
            height: ed.svek_h,
            center_x: cx,
            center_y: cy,
            rx: ed.rx,
            ry: ed.ry,
            text_x,
            text_y,
            text_width: ed.text_w,
            rank,
        });
    }

    // Create layout links
    let node_map: HashMap<&String, &LayoutNode> = nodes.iter().map(|n| (&n.name, n)).collect();
    let mut link_counter = entity_counter;
    let mut layout_links = Vec::new();

    // Edge colors are sequential after node colors
    let max_node_color = node_colors.values().copied().max().unwrap_or(5);
    let mut edge_color = max_node_color + 1;

    for link in links {
        if let (Some(from_node), Some(to_node)) =
            (node_map.get(&link.from), node_map.get(&link.to))
        {
            link_counter += 1;
            let link_id = format!("lnk{link_counter}");
            let is_extension = link.arrow.contains("<|") || link.arrow.contains("|<");
            let link_type = if is_extension { "extension" } else { "dependency" };
            let path_id = if is_extension {
                format!("{}-backto-{}", link.from, link.to)
            } else {
                format!("{}-to-{}", link.from, link.to)
            };

            // Get the edge path from dot SVG
            let (path_d, arrow_points) = if let Some(edge_path) = dot_result.edges.get(&edge_color)
            {
                // Apply moveDelta to all points
                let pts: [(f64, f64); 4] = [
                    (edge_path.points[0].0 + dx, edge_path.points[0].1 + dy),
                    (edge_path.points[1].0 + dx, edge_path.points[1].1 + dy),
                    (edge_path.points[2].0 + dx, edge_path.points[2].1 + dy),
                    (edge_path.points[3].0 + dx, edge_path.points[3].1 + dy),
                ];

                // Check if path needs to be reversed (compare start/end with source/target centers)
                let dist_normal = (pts[0].0 - from_node.center_x).hypot(pts[0].1 - from_node.center_y)
                    + (pts[3].0 - to_node.center_x).hypot(pts[3].1 - to_node.center_y);
                let dist_reversed = (pts[3].0 - from_node.center_x).hypot(pts[3].1 - from_node.center_y)
                    + (pts[0].0 - to_node.center_x).hypot(pts[0].1 - to_node.center_y);

                let pts = if dist_reversed < dist_normal {
                    // Reverse: swap start/end and control points
                    [pts[3], pts[2], pts[1], pts[0]]
                } else {
                    pts
                };
                // For extension links (inheritance/realization), the arrow is at
                // the START (parent end). For dependency links, the arrow is at
                // the END (child end).
                // Extension uses a hollow triangle (xWing=18, yAperture=6, decLen=18).
                // Dependency uses a filled arrow (decLen=5).
                if is_extension {
                    let dec_len = 18.0;
                    // Angle from start towards control1 (into the path)
                    let path_angle = (pts[1].1 - pts[0].1).atan2(pts[1].0 - pts[0].0);
                    // Shorten start by dec_len in path direction
                    let sdx = dec_len * path_angle.cos();
                    let sdy = dec_len * path_angle.sin();
                    let shortened_start = (pts[0].0 + sdx, pts[0].1 + sdy);
                    let shortened_ctrl1 = (pts[1].0 + sdx, pts[1].1 + sdy);

                    let path_d = format!(
                        "M{},{} C{},{} {},{} {},{}",
                        fmt_coord(shortened_start.0),
                        fmt_coord(shortened_start.1),
                        fmt_coord(shortened_ctrl1.0),
                        fmt_coord(shortened_ctrl1.1),
                        fmt_coord(pts[2].0),
                        fmt_coord(pts[2].1),
                        fmt_coord(pts[3].0),
                        fmt_coord(pts[3].1),
                    );
                    // Triangle arrow: tip at start, pointing away from path
                    let arrow_angle = path_angle + std::f64::consts::PI;
                    let arrow_points = compute_triangle_polygon(pts[0].0, pts[0].1, arrow_angle, 18.0, 6.0);
                    (path_d, arrow_points)
                } else {
                    let dec_len = 5.0;
                    let end_angle = (pts[3].1 - pts[2].1).atan2(pts[3].0 - pts[2].0);
                    let sdx = dec_len * end_angle.cos();
                    let sdy = dec_len * end_angle.sin();
                    let shortened_end = (pts[3].0 - sdx, pts[3].1 - sdy);
                    let shortened_ctrl2 = (pts[2].0 - sdx, pts[2].1 - sdy);

                    let path_d = format!(
                        "M{},{} C{},{} {},{} {},{}",
                        fmt_coord(pts[0].0),
                        fmt_coord(pts[0].1),
                        fmt_coord(pts[1].0),
                        fmt_coord(pts[1].1),
                        fmt_coord(shortened_ctrl2.0),
                        fmt_coord(shortened_ctrl2.1),
                        fmt_coord(shortened_end.0),
                        fmt_coord(shortened_end.1),
                    );
                    let arrow_points = compute_arrow_polygon(pts[3].0, pts[3].1, end_angle);
                    (path_d, arrow_points)
                }
            } else {
                // Fallback: simple straight line
                let path_d = format!(
                    "M{},{} C{},{} {},{} {},{}",
                    fmt_coord(from_node.center_x),
                    fmt_coord(from_node.center_y + from_node.ry),
                    fmt_coord(from_node.center_x),
                    fmt_coord(from_node.center_y + from_node.ry),
                    fmt_coord(to_node.center_x),
                    fmt_coord(to_node.center_y - to_node.ry),
                    fmt_coord(to_node.center_x),
                    fmt_coord(to_node.center_y - to_node.ry),
                );
                let arrow_points =
                    compute_arrow_polygon(to_node.center_x, to_node.center_y - to_node.ry, std::f64::consts::FRAC_PI_2);
                (path_d, arrow_points)
            };

            layout_links.push(LayoutLink {
                from: link.from.clone(),
                to: link.to.clone(),
                from_id: from_node.entity_id.clone(),
                to_id: to_node.entity_id.clone(),
                link_id,
                link_type: link_type.to_string(),
                path_id,
                source_line: link.source_line,
                path_d,
                arrow_points,
                is_line: false,
                start: (0.0, 0.0),
                end: (0.0, 0.0),
                label: link.label.clone(),
                arrow: link.arrow.clone(),
            });

            edge_color += 1;
        }
    }

    // Compute SVG dimensions matching Java Smetana's formula.
    // Java Smetana: total = minMax.getDimension().delta(15, 15)
    // where minMax is from LimitFinder. The LimitFinder's drawRectangle uses
    // addPoint(x-1, y-1) and addPoint(x+width-1, y+height-1), giving minX=-1.
    // For entities with body, drawEmpty (from TextBlockMarged) uses addPoint
    // without the -1 offset, extending maxX by 1 beyond the entity rect.
    // After moveDelta(6-(-1), 6-(-1)) = moveDelta(7,7), entities are at (7,7).
    //
    // Empirically verified against plantuml.jar 1.2026.6:
    //   total_width  = total_margin + floor(max_right)
    //   total_height = total_margin + floor(max_bottom)
    // where total_margin = margin + 8 for 2+ entities, margin + 6 for a single
    // entity (the +8/+6 comes from LimitFinder delta(15,15) minus stroke-width
    // adjustments; for class diagrams margin=7 → 15/13, for others margin=6 →
    // 14/12).
    let total_margin = if nodes.len() > 1 { margin + 8.0 } else { margin + 6.0 };
    let max_right = nodes.iter().map(|n| n.x + n.width).fold(0.0_f64, f64::max);
    let max_bottom = nodes.iter().map(|n| n.y + n.height).fold(0.0_f64, f64::max);
    let total_width = total_margin + max_right.floor();
    let total_height = total_margin + max_bottom.floor();

    CucaLayout {
        nodes,
        links: layout_links,
        total_width,
        total_height,
    }
}

/// Fallback layout when dot binary is not available.
fn build_fallback_layout(
    entities: &HashMap<String, ParsedEntity>,
    entity_data: &HashMap<&String, EntityData>,
    links: &[ParsedLink],
    rank_of: &HashMap<&String, usize>,
) -> CucaLayout {
    // Simple vertical stack layout
    let mut nodes = Vec::new();
    let mut sorted: Vec<(&String, &ParsedEntity)> = entities.iter().collect();
    sorted.sort_by_key(|(_, e)| e.source_line);

    let mut y = FALLBACK_MARGIN;
    let mut entity_counter = 0u32;

    for (name, entity) in &sorted {
        entity_counter += 1;
        let entity_id = format!("ent{entity_counter:04}");
        let ed = entity_data.get(*name).unwrap();
        let rank = rank_of.get(*name).copied().unwrap_or(0);

        let (cx, cy, text_x, text_y) = match entity.kind {
            EntityKind::Actor => {
                let cx = FALLBACK_MARGIN + ed.svek_w / 2.0;
                let cy = y + 8.0;
                (cx, cy, FALLBACK_MARGIN + (ed.svek_w - ed.text_w) / 2.0, y + ACTOR_HEIGHT + 14.4659)
            }
            EntityKind::Usecase => {
                let cx = FALLBACK_MARGIN + ed.rx;
                let cy = y + ed.ry;
                (cx, cy, FALLBACK_MARGIN + (2.0 * ed.rx - ed.text_w) / 2.0, cy + 6.0339)
            }
            _ => {
                let cx = FALLBACK_MARGIN + ed.svek_w / 2.0;
                let cy = y + ed.svek_h / 2.0;
                (cx, cy, FALLBACK_MARGIN + 10.0, y + TEXT_HEIGHT)
            }
        };

        nodes.push(LayoutNode {
            name: (*name).clone(),
            display: entity.display.clone(),
            kind: entity.kind,
            source_line: entity.source_line,
            entity_id,
            x: FALLBACK_MARGIN,
            y,
            width: ed.svek_w,
            height: ed.svek_h,
            center_x: cx,
            center_y: cy,
            rx: ed.rx,
            ry: ed.ry,
            text_x,
            text_y,
            text_width: ed.text_w,
            rank,
        });

        y += ed.svek_h + RANKSEP_PX;
    }

    let total_width = nodes
        .iter()
        .map(|n| n.x + n.width)
        .fold(0.0_f64, f64::max)
        + FALLBACK_MARGIN;
    let total_height = y + FALLBACK_MARGIN;


    // Build links as straight lines between entity centers.
    let node_map: HashMap<&String, &LayoutNode> = nodes.iter().map(|n| (&n.name, n)).collect();
    let entity_count = nodes.len();
    let mut link_counter = entity_count;
    let mut layout_links = Vec::new();
    for link in links {
        if let (Some(from_node), Some(to_node)) =
            (node_map.get(&link.from), node_map.get(&link.to))
        {
            link_counter += 1;
            let start = (from_node.center_x, from_node.center_y + from_node.height / 2.0);
            let end = (to_node.center_x, to_node.center_y - to_node.height / 2.0);
            let path_d = format!(
                "M{},{} C{},{} {},{} {},{}",
                fmt_coord(start.0), fmt_coord(start.1),
                fmt_coord(start.0), fmt_coord(start.1),
                fmt_coord(end.0), fmt_coord(end.1),
                fmt_coord(end.0), fmt_coord(end.1),
            );
            let arrow_angle = std::f64::consts::FRAC_PI_2;
            let arrow_points = compute_arrow_polygon(end.0, end.1, arrow_angle);
            let is_extension = link.arrow.contains("<|") || link.arrow.contains("|<");
            let link_type = if is_extension { "extension" } else { "dependency" };
            let path_id = if is_extension {
                format!("{}-backto-{}", link.from, link.to)
            } else {
                format!("{}-to-{}", link.from, link.to)
            };
            layout_links.push(LayoutLink {
                from: link.from.clone(),
                to: link.to.clone(),
                from_id: from_node.entity_id.clone(),
                to_id: to_node.entity_id.clone(),
                link_id: format!("lnk{link_counter}"),
                link_type: link_type.to_string(),
                path_id,
                source_line: link.source_line,
                path_d,
                arrow_points,
                is_line: true,
                start,
                end,
                label: link.label.clone(),
                arrow: link.arrow.clone(),
            });
        }
    }

    CucaLayout {
        nodes,
        links: layout_links,
        total_width,
        total_height,
    }
}

// ── Arrow polygon ───────────────────────────────────────────────────────

/// Formats a coordinate like Java's `%.4f` with trailing zeros and decimal point stripped.
fn fmt_coord(v: f64) -> String {
    let s = format!("{v:.4}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Computes the arrow polygon points at a given position and angle.
///
/// Ported from: `ExtremityArrow.java`.
/// Arrow shape: (0,0), (-9,-4), (-5,0), (-9,4), (0,0) — 5 points with closing tip.
fn compute_arrow_polygon(x: f64, y: f64, angle: f64) -> String {
    // Arrow polygon points (before rotation): tip, left-wing, notch, right-wing, tip
    let pts = [
        (0.0, 0.0),
        (-9.0, -4.0),
        (-5.0, 0.0),
        (-9.0, 4.0),
        (0.0, 0.0),
    ];
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let mut result = String::new();
    for (i, (px, py)) in pts.iter().enumerate() {
        let rx = x + px * cos_a - py * sin_a;
        let ry = y + px * sin_a + py * cos_a;
        if i > 0 {
            result.push(',');
        }
        write!(result, "{},{}", fmt_coord(rx), fmt_coord(ry)).unwrap();
    }
    result
}

/// Computes a hollow triangle arrow polygon (for extension/inheritance links).
/// Ported from: `ExtremityTriangle.java` and `ExtremityFactoryTriangle.java`.
/// Shape: (0,0), (-xWing,-yAperture), (-xWing,yAperture), (0,0) — 4 points.
fn compute_triangle_polygon(x: f64, y: f64, angle: f64, x_wing: f64, y_aperture: f64) -> String {
    let pts = [
        (0.0, 0.0),
        (-x_wing, -y_aperture),
        (-x_wing, y_aperture),
        (0.0, 0.0),
    ];
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let mut result = String::new();
    for (i, (px, py)) in pts.iter().enumerate() {
        let rx = x + px * cos_a - py * sin_a;
        let ry = y + px * sin_a + py * cos_a;
        if i > 0 {
            result.push(',');
        }
        write!(result, "{},{}", fmt_coord(rx), fmt_coord(ry)).unwrap();
    }
    result
}

// ── Helper struct ───────────────────────────────────────────────────────

struct EntityData {
    text_w: f64,
    svek_w: f64,
    svek_h: f64,
    rx: f64,
    ry: f64,
}

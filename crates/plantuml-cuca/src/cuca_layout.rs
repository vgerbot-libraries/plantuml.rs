//! Layout engine for CucaDiagram types using external `dot` (Graphviz).
//!
//! Ported from: `net/sourceforge/plantuml/svek/CucaDiagramFileMakerSvek.java`.
//!
//! The Java Svek engine generates a dot string, runs the external `dot`
//! binary, and parses the resulting SVG for node positions and edge spline
//! paths. This Rust implementation replicates that pipeline for exact
//! behavioral parity with Java PlantUML 1.2026.6.

use indexmap::IndexMap;
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
/// Font size for an edge label (one point below the entity label).
const LABEL_FONT_SIZE: i32 = 13;
/// Baseline offset of an edge label below the rank-gap midpoint.
///
/// The 19px label table sits in the gap center; the 13px baseline lands
/// 5.397 (≈ half the 19.0679 line height − cap adjustment) below center.
const LABEL_BASELINE_OFFSET: f64 = 5.397;
/// Inset of the cluster label bbox above the folder origin: Smetana lays the
/// label out as an inset node reaching 0.602px above the drawn origin.
const LABEL_INSET: f64 = 0.602;

// ── Structs ─────────────────────────────────────────────────────────────

/// A positioned entity in the layout.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// Entity name.
    pub name: String,
    /// Fully qualified name (parent chain joined with `.`), for
    /// `data-qualified-name`.
    pub qualified_name: String,
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
    /// Whether the node's size is a forced embedded inner image (folded
    /// composite), whose far corner sizes the canvas with a full 15 delta.
    pub forced_size: bool,
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
    /// Baseline anchor of the optional edge label (post-move placed space).
    pub label_anchor: Option<(f64, f64)>,
    /// Far corner (right, bottom) of the edge-label `UEmpty` spacer, used to
    /// size the canvas; `None` for an unlabeled link.
    pub label_spacer_right: Option<f64>,
    /// Arrow type string (e.g. "-->", "<|--", "<|..").
    pub arrow: String,
}

/// A positioned package/group cluster drawn as a folder around its members.
#[derive(Debug, Clone)]
pub struct LayoutCluster {
    /// Group entity name.
    pub name: String,
    /// Group display label (may contain `\n` for a multi-line title).
    pub display: String,
    /// Source line number.
    pub source_line: usize,
    /// Entity ID (ent0001, ...), shared with the leaf uid sequence.
    pub entity_id: String,
    /// Folder bounding-box top-left X (rounded).
    pub x: f64,
    /// Folder bounding-box top-left Y.
    pub y: f64,
    /// Folder bounding-box width.
    pub width: f64,
    /// Folder bounding-box height.
    pub height: f64,
    /// Title rendered width (textLength of the widest title line).
    pub title_width: f64,
    /// Number of title lines.
    pub title_lines: usize,
}
/// Raw folder geometry in the solver frame before moveDelta and title
/// measurement: `(source_line, name, left, top, width, height, bottom, lines)`.
type RawCluster<'a> = (usize, &'a String, f64, f64, f64, f64, f64, usize);

/// Computed layout for a CucaDiagram.
#[derive(Debug, Clone)]
pub struct CucaLayout {
    /// Positioned entity nodes (leaves only).
    pub nodes: Vec<LayoutNode>,
    /// Positioned package/group clusters.
    pub clusters: Vec<LayoutCluster>,
    /// Positioned links.
    pub links: Vec<LayoutLink>,
    /// Total SVG width.
    pub total_width: f64,
    /// Total SVG height.
    pub total_height: f64,
}

// ── Text measurement ────────────────────────────────────────────────────

/// Measures text width using StringBounderSvg (matches Java AWT).
fn measure_text_size(text: &str, size: i32) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(size);
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

/// Crate-visible wrapper for [`measure_text_size`] used by the labeled solver.
pub(crate) fn measure_text_size_pub(text: &str, size: i32) -> f64 {
    measure_text_size(text, size)
}

/// Measures text width using StringBounderSvg (matches Java AWT).
fn measure_text(text: &str) -> f64 {
    measure_text_size(text, FONT_SIZE)
}

/// Measures text width with italic font (for interface/abstract entity names).
fn measure_text_italic(text: &str) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(FONT_SIZE).with_style(FontStyle::italic());
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

/// Measures text width with bold font (for cluster/package titles).
fn measure_text_bold(text: &str) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(FONT_SIZE).with_style(FontStyle::bold());
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
    // Allow callers (and parity tests) to force the pure-Rust layout solver,
    // which is the path WASM actually uses. Without this gate, native test
    // runs silently take Graphviz while the site takes `native_layout`.
    if std::env::var_os("PLANTUML_NO_DOT").is_some() {
        return None;
    }
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
pub(crate) struct DotNodePos {
    pub(crate) min_x: f64,
    pub(crate) min_y: f64,
}

/// Parsed edge path from dot SVG (after Y-flip).
///
/// A sequence of cubic Bézier control points. Four points (start, two
/// controls, end) is a single cubic; `4 + 3k` points is `1 + k` chained
/// cubic segments, each segment's start being the previous end.
#[derive(Debug, Clone)]
pub(crate) struct DotEdgePath {
    pub(crate) points: Vec<(f64, f64)>,
}
/// Result of parsing the dot SVG output.
pub(crate) struct DotSvgResult {
    pub(crate) nodes: HashMap<i32, DotNodePos>,
    pub(crate) edges: HashMap<i32, DotEdgePath>,
    /// Raw y-up `(center y, half height)` of each leaf before 2-decimal
    /// quantization, from the native solver. Cluster bbox edges are derived
    /// from these unquantized values (graphviz `dot_compute_bb`); using the
    /// placed leaf would fold in a sub-pixel quantization error. Empty on the
    /// external-dot path.
    pub(crate) raw_yup: HashMap<String, (f64, f64)>,
    /// Internal-frame left x of each routed edge's recovered label box, keyed
    /// by edge color, for diagrams routed through the labeled solver. The
    /// edge-label text starts one pixel right of it after the move delta.
    /// y-up internal-frame center y of each labeled edge's virtual node, keyed
    /// by edge color. Graphviz positions the edge label at this rank
    /// coordinate; reconstructing it from placed node corners folds in a
    /// sub-pixel move-delta error for a folded leaf. Empty on other paths.
    pub(crate) label_vnode_y: HashMap<i32, f64>,
    pub(crate) label_box_left: HashMap<i32, f64>,
    /// Edge colors in graphviz's forward-converted draw order (upper rank
    /// ascending, forward edge before its back twin). Empty on paths that do
    /// not compute it; the consumer then uses source order.
    pub(crate) draw_order: Vec<i32>,
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
                nodes.insert(color, DotNodePos { min_x, min_y });
            } else if element.contains("polygon") {
                // Rect node: extract polygon points
                let points_str = extract_attr_str_in(element, "points")?;
                let pts = parse_points(&points_str);
                if pts.len() >= 2 {
                    let min_x = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
                    let min_y_dot = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
                    let min_y = min_y_dot + full_height;
                    nodes.insert(color, DotNodePos { min_x, min_y });
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
        nodes,
        edges,
        raw_yup: HashMap::new(),
        label_vnode_y: HashMap::new(),
        label_box_left: HashMap::new(),
        draw_order: Vec::new(),
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

    // Parse every cubic control triplet after the C. One C may carry
    // multiple triplets (chained cubic segments): 3 points for a single
    // cubic, 3(1+k) for k+1 segments.
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

    let mut points = Vec::with_capacity(1 + c_points.len());
    points.push(start);
    points.extend(c_points);
    Some(DotEdgePath { points })
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
    // Top-level Svek layout: graphviz is fed nodesep=35 and ranksep=60.
    compute_layout_ranked(entities, links, diagram_type, 35.0, 60.0, false)
}

/// Computes each entity's rendered and SvekNode dimensions for the given
/// diagram type.
#[must_use]
pub(crate) fn build_entity_data(
    entities: &HashMap<String, ParsedEntity>,
    diagram_type: plantuml_core::DiagramType,
) -> HashMap<&String, EntityData> {

    // Compute entity dimensions and SvekNode dimensions.
    let mut entity_data: HashMap<&String, EntityData> = HashMap::new();
    for (name, entity) in entities {
        let text_w = match entity.kind {
            EntityKind::Interface
                if matches!(diagram_type, plantuml_core::DiagramType::Description) =>
            {
                // Lollipop label below the circle is drawn upright.
                measure_text(&entity.display)
            }
            EntityKind::Abstract | EntityKind::Interface => measure_text_italic(&entity.display),
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
            EntityKind::Node => {
                // USymbolNode, Margin(15, 25, 20, 10):
                // 3D box wraps the label plus 40 horizontal / 30 vertical.
                (text_w + 40.0, TEXT_HEIGHT + 30.0, 0.0, 0.0)
            }
            EntityKind::Component => {
                // USymbolComponent2 (UML 2 notation), Margin(15, 25, 20, 10):
                // box wraps the label plus 40 horizontal / 30 vertical.
                (text_w + 40.0, TEXT_HEIGHT + 30.0, 0.0, 0.0)
            }
            EntityKind::Database => {
                // USymbolDatabase, Margin(10, 10, 24, 5):
                // cylinder wraps the label plus 20 horizontal / 29 vertical.
                (text_w + 20.0, TEXT_HEIGHT + 29.0, 0.0, 0.0)
            }
            EntityKind::Interface
                if matches!(diagram_type, plantuml_core::DiagramType::Description) =>
            {
                // Dot receives an HTML shield table: an 18x18 port cell
                // padded to the label width and a 19.0679 line each side,
                // then inflates the plaintext node by +16 wide / +8 tall
                // (floored at 54x36) after rounding the table to integers.
                let table_w = text_w.max(18.0).round();
                let table_h = (18.0 + 2.0 * TEXT_HEIGHT).round();
                let w = (table_w + 16.0).max(54.0);
                let h = (table_h + 8.0).max(36.0);
                (w, h, 0.0, 0.0)
            }
            EntityKind::State => {
                // Rounded state box: at least 50x50, wide enough for the name
                // plus a 10px margin each side.
                (50.0_f64.max(text_w + 20.0), 50.0, 0.0, 0.0)
            }
            EntityKind::Start => {
                // Initial pseudo-state: filled r=10 circle.
                (20.0, 20.0, 10.0, 10.0)
            }
            EntityKind::End => {
                // Final pseudo-state: outer r=11 ring (drives the box).
                (22.0, 22.0, 11.0, 11.0)
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
        let (svek_w, svek_h) = match entity.forced_size {
            Some((w, h)) => (w, h),
            None => (svek_w, svek_h),
        };
        let port_circle = if matches!(entity.kind, EntityKind::Interface)
            && matches!(diagram_type, plantuml_core::DiagramType::Description)
        {
            // Edges clip to the 18x18 centre port cell (half extent 9).
            Some(9.0)
        } else {
            None
        };
        entity_data.insert(
            name,
            EntityData {
                text_w,
                svek_w,
                svek_h,
                rx,
                ry,
                port_circle,
            },
        );
    }
    entity_data
}
/// Core layout with an explicit base `ranksep`.
///
/// A folded composite state's independent inner sublayout
/// (`GroupMakerState`) passes no `ranksep`, so graphviz uses its 0.5in
/// (36px) default; the outer/top-level layout passes 60px.
#[must_use]
pub(crate) fn compute_layout_ranked(
    entities: &HashMap<String, ParsedEntity>,
    links: &[ParsedLink],
    diagram_type: plantuml_core::DiagramType,
    nodesep: f64,
    base_ranksep: f64,
    force_labeled: bool,
) -> CucaLayout {
    if entities.is_empty() {
        return CucaLayout {
            nodes: Vec::new(),
            clusters: Vec::new(),
            links: Vec::new(),
            total_width: CANVAS_MARGIN * 2.0,
            total_height: CANVAS_MARGIN * 2.0,
        };
    }

    // Sort entities by source_line for correct entity ID assignment
    let mut sorted_entities: Vec<(&String, &ParsedEntity)> = entities.iter().collect();
    sorted_entities.sort_by_key(|(_, e)| e.source_line);

    // Resolve written bracket-form endpoints (`[Web Server]`, `[*]`) to the
    // qualified entity keys layout/routing use. Build a source-ordered
    // IndexMap so `resolve_endpoint` sees a stable lookup order.
    let ordered: IndexMap<String, ParsedEntity> = sorted_entities
        .iter()
        .map(|(name, entity)| ((**name).clone(), (**entity).clone()))
        .collect();
    let links: Vec<ParsedLink> = links
        .iter()
        .map(|link| ParsedLink {
            from: crate::entity_link_parser::resolve_endpoint(&link.from, true, &ordered),
            to: crate::entity_link_parser::resolve_endpoint(&link.to, false, &ordered),
            arrow: link.arrow.clone(),
            label: link.label.clone(),
            source_line: link.source_line,
        })
        .collect();

    let entity_data: HashMap<&String, EntityData> = build_entity_data(entities, diagram_type);

    // Leaves (non-group entities) are the only nodes dot/Smetana places; a
    // group is a cluster subgraph drawn around its members, not a box node.
    let leaf_sorted: Vec<(&String, &ParsedEntity)> = sorted_entities
        .iter()
        .copied()
        .filter(|(_, e)| !e.group)
        .collect();

    // Each leaf's immediate containing group, and each group's title line count.
    let leaf_group: HashMap<&String, &String> = leaf_sorted
        .iter()
        .filter_map(|(_, e)| e.parent.as_ref().map(|p| (&e.name, p)))
        .collect();

    // Build adjacency for rank assignment (needed for LayoutNode.rank field).
    let entity_names: HashSet<&String> = leaf_sorted.iter().map(|(n, _)| *n).collect();
    let mut incoming: HashMap<&String, HashSet<&String>> = HashMap::new();
    let mut outgoing: HashMap<&String, HashSet<&String>> = HashMap::new();

    for link in &links {
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


    // Pure-Rust layout solver; no external `dot` required (works under WASM).
    // Only leaves are placed; groups are drawn as clusters around members.
    let sorted_data: Vec<(&String, &EntityData)> = leaf_sorted
        .iter()
        .map(|(name, _)| (*name, &entity_data[name]))
        .collect();

    // Map each leaf to its immediate containing group so the solver can choose
    // the 60px within-cluster gap or the 78px cross-cluster gap.

    // Prefer the external `dot` engine when it is on PATH (native builds;
    // bit-exact with Java/Smetana). Under WASM, or wherever `dot` is missing,
    // fall back to the pure-Rust solver instead of the old vertical stack.
    let (dot_string, node_colors) =
        generate_dot_string(&leaf_sorted, &entity_data, &links);
    // Select the pure-Rust solver. Graphviz switches to rank doubling for the
    // whole graph whenever any edge carries a label (`lib/dotgen/rank.c
    // ::edgelabel_ranks`). The default `native_layout` reproduces flat
    // diagrams; the doubled-rank `solve_labeled` is required only by the
    // autarkic-fold outer frame, which requests it explicitly through
    // `force_labeled`. Keeping ordinary graphs on the native solver preserves
    // the clean-baseline flat-diagram output.
    let native_solve = || {
        if force_labeled {
            crate::labeled_layout::solve_labeled(&sorted_data, &links, nodesep, base_ranksep)
        } else {
            crate::native_layout::solve(&sorted_data, &links, &leaf_group)
        }
    };
    let dot_result = if let Some(svg) = run_dot(&dot_string) {
        match parse_dot_svg(&svg, &node_colors) {
            Some(parsed) => parsed,
            None => native_solve(),
        }
    } else {
        native_solve()
    };
    build_layout_from_dot(
        &sorted_entities,
        &entity_data,
        &links,
        &node_colors,
        &dot_result,
        &rank_of,
        diagram_type,
    )
}

/// LimitFinder border added to a node's placed bounding-box minimum, by shape.
///
/// A component's outer symbol is a `URectangle`, whose `drawRectangle`
/// records `(x-1, y-1)`, so its effective min is one pixel beyond the placed
/// box. The actor head ellipse starts at thickness()=0.5 on the y axis.
/// Other symbols (class `UPath`, use case ellipse, database `UPath`) start
/// exactly at the placed origin.
pub(crate) fn min_border_x(kind: EntityKind) -> f64 {
    match kind {
        // URectangle-backed symbols record (x-1).
        EntityKind::Component | EntityKind::State => -1.0,
        // The 3D node's fold flap starts 10px left of the placed box.
        EntityKind::Node => -10.0,
        _ => 0.0,
    }
}
pub(crate) fn min_border_y(kind: EntityKind) -> f64 {
    match kind {
        // URectangle-backed symbols record (y-1).
        EntityKind::Component | EntityKind::State => -1.0,
        EntityKind::Actor => 0.5,
        _ => 0.0,
    }
}

/// LimitFinder extension beyond a node's placed bounding-box maximum, by shape.
///
/// `USymbolDatabase.drawDatabase` finishes by drawing an `UEmpty(10,10)`
/// translated to `(width,height)`; its `drawEmpty` records the far corner
fn max_extent_x(kind: EntityKind) -> f64 {
    match kind {
        EntityKind::Database => 10.0,
        // The 3D node's back face extends ~11px right for routing/bbox.
        EntityKind::Node => 11.0,
        _ => 0.0,
    }
}
fn max_extent_y(kind: EntityKind) -> f64 {
    match kind {
        EntityKind::Database => 10.0,
        _ => 0.0,
    }
}

/// Collects every leaf key nested under group `root`, descending through
/// nested groups, in depth-first source order.
fn collect_member_leaves<'a>(
    root: &String,
    sorted_entities: &'a [(&String, &ParsedEntity)],
    out: &mut Vec<&'a String>,
) {
    let entity_of = |key: &String| -> Option<&'a ParsedEntity> {
        sorted_entities
            .iter()
            .find(|(n, _)| *n == key)
            .map(|(_, e)| *e)
    };
    let Some(root_ent) = entity_of(root) else {
        return;
    };
    if !root_ent.group {
        out.push(&root_ent.name);
        return;
    }
    let mut stack: Vec<&String> = root_ent.members.iter().collect();
    while let Some(cur) = stack.pop() {
        let Some(ent) = entity_of(cur) else {
            continue;
        };
        if ent.group {
            // Push so the first member is processed first (reverse to preserve
            // order).
            for m in ent.members.iter().rev() {
                stack.push(m);
            }
        } else {
            out.push(&ent.name);
        }
    }
}

/// Applies a move-delta to every edge point.
pub(crate) fn shift_points(points: &[(f64, f64)], dx: f64, dy: f64) -> Vec<(f64, f64)> {
    points.iter().map(|&(x, y)| (x + dx, y + dy)).collect()
}

/// Formats a chained-cubic edge path from its control points, optionally
/// shortening the start (extension) or end (dependency) by the arrow length,
/// and returns `(path d, arrow polygon)`. Association shortens neither end.
pub(crate) fn emit_edge_path(
    pts: &[(f64, f64)],
    is_extension: bool,
    is_association: bool,
) -> (String, String) {
    let n = pts.len();
    // Local mutable copy so the arrow shortening moves only this edge's
    // terminal point and its adjacent control, as graphviz redraws the segment.
    let mut p = pts.to_vec();

    // Tangent at the start points along the first segment (into the path).
    let start_angle = (p[1].1 - p[0].1).atan2(p[1].0 - p[0].0);
    // Tangent at the end points along the last segment, out of the path.
    let end_angle = (p[n - 1].1 - p[n - 2].1).atan2(p[n - 1].0 - p[n - 2].0);

    if is_extension {
        // Hollow triangle (decLen 18) sits at the start; pull the start and its
        // first control 18px into the path.
        let len = 18.0;
        let (sx, sy) = (len * start_angle.cos(), len * start_angle.sin());
        p[0].0 += sx;
        p[0].1 += sy;
        p[1].0 += sx;
        p[1].1 += sy;
    } else if !is_association {
        // Filled arrow (decLen 5) sits at the end; pull the end and its last
        // control 5px back along the reverse tangent.
        let len = 5.0;
        let (sx, sy) = (len * end_angle.cos(), len * end_angle.sin());
        p[n - 1].0 -= sx;
        p[n - 1].1 -= sy;
        p[n - 2].0 -= sx;
        p[n - 2].1 -= sy;
    }

    let mut d = format!("M{},{}", fmt_coord(p[0].0), fmt_coord(p[0].1));
    // Points advance three at a time: each cubic covers p[s..=s+3].
    let mut s = 0usize;
    while s + 3 < n {
        write!(
            d,
            " C{},{} {},{} {},{}",
            fmt_coord(p[s + 1].0),
            fmt_coord(p[s + 1].1),
            fmt_coord(p[s + 2].0),
            fmt_coord(p[s + 2].1),
            fmt_coord(p[s + 3].0),
            fmt_coord(p[s + 3].1),
        )
        .unwrap_or_default();
        s += 3;
    }

    let arrow = if is_extension {
        // Tip at the (original, unshortened) start pointing back out.
        let tip = pts[0];
        compute_triangle_polygon(tip.0, tip.1, start_angle + std::f64::consts::PI, 18.0, 6.0)
    } else if is_association {
        String::new()
    } else {
        let tip = pts[n - 1];
        compute_arrow_polygon(tip.0, tip.1, end_angle)
    };
    (d, arrow)
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


    // Groups in source order: these become folder clusters, never box nodes.
    let group_sorted: Vec<(&String, &ParsedEntity)> = sorted_entities
        .iter()
        .copied()
        .filter(|(_, e)| e.group)
        .collect();

    // ── Raw-frame leaf boxes ─────────────────────────────────────────────
    // Placed (un-shifted) box of each leaf: top-left from the solver, size
    // after the same points round-trip the solver applied to its half extents.
    let leaf_box_raw: HashMap<&String, (f64, f64, f64, f64)> = sorted_entities
        .iter()
        .filter(|(_, e)| !e.group)
        .filter_map(|(name, _)| {
            let ed = entity_data.get(*name)?;
            let color = node_colors.get(*name).copied()?;
            let pos = dot_result.nodes.get(&color)?;
            Some((
                *name,
                (
                    pos.min_x,
                    pos.min_y,
                    crate::native_layout::dot_roundtrip_px(ed.svek_w),
                    crate::native_layout::dot_roundtrip_px(ed.svek_h),
                ),
            ))
        })
        .collect();

    // (member-leaf collection is a free helper: see `collect_member_leaves`.)

    // ── Clusters (folder decoration) in the raw frame ────────────────────
    // Above its top leaf a folder reserves 16px margin plus the title block
    // (19px per line + 3px): 38 for one line, 57 for two. Below the bottom
    // leaf it reserves the 16px cluster margin. Horizontally the folder is the
    // member extent ± 16, rounded.
    let mut clusters_raw: Vec<RawCluster<'_>> = Vec::new();
    // A stacked column has ONE centre shared by every leaf and cluster. The
    // native normalization pins the outer frame edge to zero so that centre is
    // the rounded outer extent — an integer. Compute it once from all leaves
    // (corner reconstruction perturbs per-leaf centres by ≤0.005) and round.
    let global_center = {
        let (sum, n) = leaf_box_raw
            .values()
            .map(|(x, _, w, _)| (x + w / 2.0, 1u32))
            .fold((0.0_f64, 0u32), |(s, n), (c, k)| (s + c, n + k));
        if n == 0 {
            None
        } else {
            Some((sum / f64::from(n)).round())
        }
    };

    let mut member_leaves: Vec<&String> = Vec::new();
    for (gname, gent) in &group_sorted {
        member_leaves.clear();
        collect_member_leaves(gname, sorted_entities, &mut member_leaves);
        // Member leaves' placed centre and the content half-extent reaching
        // left/right of it. The cluster box is symmetric about that centre:
        // edges are `centre ± round(inner + 16)` — rounding the half-extent,
        // not the absolute edge (Frontend width 86, Backend width 94).
        let mut inner_left = 0.0f64;
        let mut inner_right = 0.0f64;
        let mut min_y = f64::INFINITY;
        let mut max_b = f64::NEG_INFINITY;
        for ln in &member_leaves {
            if let Some((x, y, w, h)) = leaf_box_raw.get(*ln) {
                let c = x + w / 2.0;
                inner_left = inner_left.max(c - x);
                inner_right = inner_right.max(x + w - c);
                min_y = min_y.min(*y);
                max_b = max_b.max(y + h);
            }
        }
        if min_y.is_infinite() {
            continue;
        }
        let ccx = global_center.expect("at least one leaf");
        let left_extent = (inner_left + 16.0).round();
        let right_extent = (inner_right + 16.0).round();
        let left = ccx - left_extent;
        let right = ccx + right_extent;
        let lines = 1 + gent.display.matches('\n').count();
        let top_extent = 16.0 + (19.0 * lines as f64 + 3.0);
        let top = ((min_y - top_extent) * 100.0).round() / 100.0;
        // graphviz sets the cluster bbox from the RAW (un-quantized) node
        // coordinate: yup bottom = center − (half_h + cluster margin). Using
        // the q2-placed leaf edge folds in a sub-pixel quantization error.
        let bottom_raw = if dot_result.raw_yup.is_empty() {
            max_b + 16.0
        } else {
            member_leaves
                .iter()
                .filter_map(|ln| {
                    dot_result
                        .raw_yup
                        .get(*ln)
                        .map(|&(cy, hh)| -cy + hh + 16.0)
                })
                .fold(f64::NEG_INFINITY, f64::max)
        };
        // Graphviz serializes the cluster polygon rounded to 2 decimals, and
        // PlantUML parses those rounded points (`DotStringFactory.solve`). The
        // rounding is total-graph dependent via the rank `ht1` feedback, so
        // replicate it here rather than carrying the unrounded leaf value.
        let bottom = (bottom_raw * 100.0).round() / 100.0;
        clusters_raw.push((
            gent.source_line,
            gname,
            left,
            top,
            right - left,
            bottom - top,
            bottom,
            lines,
        ));
    }

    // Compute moveDelta: `margin - min` over all nodes, including each
    // shape's LimitFinder border.
    // moveDelta is driven by the placed LEAF boxes only. The folder tab
    // extends a fixed 38px (one-line title) above its top leaf, but the
    // cluster label is laid out as an inset node, so the tab top lands 0.602px
    // below the cluster origin — folding the raw tab into the minimum would
    // pin it to the margin and shift every leaf up by that 0.602. Keeping the
    // leaf-relative geometry intact reproduces the tab at `margin + 0.602`.
    // A labeled edge's control points reach beyond every node box: graphviz
    // sizes the graph bbox from the routed spline points, not the node edges.
    // Fold the minimum edge control-point x into the move-delta minimum or the
    // node column stays pinned to the margin and the canvas comes up narrow.
    let edge_min_x = dot_result
        .edges
        .values()
        .flat_map(|e| e.points.iter().map(|p| p.0))
        .fold(f64::INFINITY, f64::min);
    let min_min_x = sorted_entities
        .iter()
        .filter_map(|(name, entity)| {
            let color = node_colors.get(*name).copied()?;
            let pos = dot_result.nodes.get(&color)?;
            Some(pos.min_x + min_border_x(entity.kind))
        })
        .fold(f64::INFINITY, f64::min)
        .min(
            clusters_raw
                .iter()
                .map(|(_, _, left, _, _, _, _, _)| *left)
                .fold(f64::INFINITY, f64::min),
        );
    let min_min_x = if dot_result.label_box_left.is_empty() {
        min_min_x
    } else {
        min_min_x.min(edge_min_x)
    };
    // The folder tab sets the vertical minimum: the top leaf sits 38px (or
    // 57px) below it. Smetana lays out the cluster label as an inset node,
    // whose bounding box reaches 0.602px above the drawn folder origin, so the
    // global LimitFinder minimum is `tab_top − 0.602`. That extra 0.602 leaves
    // the rendered tab at `margin + 0.602` instead of pinning it to margin.
    let min_min_y = sorted_entities
        .iter()
        .filter_map(|(name, entity)| {
            let color = node_colors.get(*name).copied()?;
            let pos = dot_result.nodes.get(&color)?;
            Some(pos.min_y + min_border_y(entity.kind))
        })
        .fold(f64::INFINITY, f64::min)
        .min(
            clusters_raw
                .iter()
                .map(|(_, _, _, top, _, _, _, _)| top - LABEL_INSET)
                .fold(f64::INFINITY, f64::min),
        );

    let mut entity_uid: HashMap<&String, u32> = HashMap::new();
    let mut link_uid: Vec<u32> = vec![0; links.len()];
    let dx = margin - min_min_x;
    let dy = margin - min_min_y;

    // Entity uid counter is shared with links (`cpt1`): every created leaf
    // and every created link consume it, in source order; on a single line
    // the leaf is created before its link. Reproduce that ordering to get
    // the same `ent%04d` ids.
    let mut events: Vec<(usize, u8, Option<&String>, Option<usize>)> = sorted_entities
        .iter()
        .map(|(name, entity)| (entity.source_line, 0, Some(*name), None))
        .collect();
    for (i, link) in links.iter().enumerate() {
        events.push((link.source_line, 1, None, Some(i)));
    }
    events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut uid_counter = 0u32;
    for (_, _, name, link_idx) in events {
        uid_counter += 1;
        if let Some(name) = name {
            entity_uid.insert(name, uid_counter);
        }
        if let Some(i) = link_idx {
            link_uid[i] = uid_counter;
        }
    }

    // Create layout nodes — sorted by source_line for correct entity ID order
    let mut nodes = Vec::new();

    for (name, entity) in sorted_entities {
        if entity.group {
            continue;
        }
        let entity_id = format!("ent{:04}", entity_uid.get(name).copied().unwrap_or(0));

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
                EntityKind::Node => {
                    // USymbolNode Margin(15, 25, 20, 10): label at (15, 34.9659).
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + ed.svek_h / 2.0;
                    let text_x = corner_x + 15.0;
                    let text_y = corner_y + 34.9659;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                EntityKind::Component => {
                    // USymbolComponent2 Margin(15, 25, 20, 10): label at (15, 34.9659).
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + ed.svek_h / 2.0;
                    let text_x = corner_x + 15.0;
                    let text_y = corner_y + 34.9659;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                EntityKind::Database => {
                    // USymbolDatabase Margin(10, 10, 24, 5): label at (10, 38.9659).
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + ed.svek_h / 2.0;
                    let text_x = corner_x + 10.0;
                    let text_y = corner_y + 38.9659;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                EntityKind::Interface
                    if matches!(diagram_type, plantuml_core::DiagramType::Description) =>
                {
                    // The circle is the centre port cell at the shield table's
                    // geometric centre; the label is centred on it and drawn
                    // below (circle_cy + 8 + 19.0679 + 4.9 = +31.9659).
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + ed.svek_h / 2.0;
                    let text_x = cx - ed.text_w / 2.0;
                    let text_y = cy + 31.9659;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                EntityKind::State => {
                    let cx = corner_x + ed.svek_w / 2.0;
                    let cy = corner_y + ed.svek_h / 2.0;
                    let text_x = corner_x + (ed.svek_w - ed.text_w) / 2.0;
                    let text_y = corner_y + 19.9659;
                    (corner_x, corner_y, cx, cy, text_x, text_y)
                }
                EntityKind::Start => {
                    let cx = corner_x + 10.0;
                    let cy = corner_y + 10.0;
                    (corner_x, corner_y, cx, cy, corner_x, corner_y)
                }
                EntityKind::End => {
                    let cx = corner_x + 11.0;
                    let cy = corner_y + 11.0;
                    (corner_x, corner_y, cx, cy, corner_x, corner_y)
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
        // The leaf key is already the fully qualified name (e.g.
        // `Frontend.UI`); only its `display` label is bare (`UI`).
        let qualified_name = (*name).clone();
        nodes.push(LayoutNode {
            name: (*name).clone(),
            display: entity.display.clone(),
            kind: entity.kind,
            source_line: entity.source_line,
            entity_id,
            x: node_x_corner,
            y: node_y_corner,
            qualified_name: qualified_name.clone(),
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
            forced_size: entity.forced_size.is_some(),
        });
    }

    // Shift clusters by the same moveDelta as the nodes and resolve their
    // entity id and bold title width.
    let entity_by_name: HashMap<&String, &ParsedEntity> =
        sorted_entities.iter().map(|(n, e)| (*n, *e)).collect();
    let clusters: Vec<LayoutCluster> = clusters_raw
        .iter()
        .map(|(source_line, gname, left, top, w, h, _, lines)| {
            let gent = entity_by_name[gname];
            let title_width = gent
                .display
                .split('\n')
                .map(measure_text_bold)
                .fold(0.0_f64, f64::max);
            LayoutCluster {
                name: (*gname).clone(),
                display: gent.display.clone(),
                source_line: *source_line,
                entity_id: format!(
                    "ent{:04}",
                    entity_uid.get(*gname).copied().unwrap_or(0)
                ),
                x: left + dx,
                y: top + dy,
                width: *w,
                height: *h,
                title_width,
                title_lines: *lines,
            }
        })
        .collect();

    // Create layout links
    let node_map: HashMap<&String, &LayoutNode> = nodes.iter().map(|n| (&n.name, n)).collect();
    let mut layout_links = Vec::new();

    // Edge colors are sequential after node colors
    let max_node_color = node_colors.values().copied().max().unwrap_or(5);
    let mut edge_color_run = max_node_color + 1;
    // Draw order defaults to source order; the labeled solver supplies
    // graphviz's forward-converted order via `dot_result.draw_order`.
    let mut draw_order: Vec<usize> = (0..links.len()).collect();
    // Each source-order link's edge color, independent of draw order, so the
    // spline lookup matches the solver's source-order keying.
    let mut color_by_li: HashMap<usize, i32> = HashMap::new();
    for (li, link) in links.iter().enumerate() {
        if node_map.contains_key(&link.from) && node_map.contains_key(&link.to) {
            color_by_li.insert(li, edge_color_run);
            edge_color_run += 1;
        }
    }
    if !dot_result.draw_order.is_empty() {
        let li_by_color: HashMap<i32, usize> =
            color_by_li.iter().map(|(&li, &c)| (c, li)).collect();
        draw_order = dot_result
            .draw_order
            .iter()
            .filter_map(|c| li_by_color.get(c).copied())
            .collect();
    }

    for &li in &draw_order {
        let link = &links[li];
        let edge_color = color_by_li[&li];
        if let (Some(from_node), Some(to_node)) =
            (node_map.get(&link.from), node_map.get(&link.to))
        {
            let link_id = format!("lnk{}", link_uid[li]);
            let is_extension = link.arrow.contains("<|") || link.arrow.contains("|<");
            // A plain edge with no arrowhead (`--`) is an association; an
            // edge ending in `>` is a dependency.
            let is_association = !is_extension && !link.arrow.contains('>');
            let link_type = if is_extension {
                "extension"
            } else if is_association {
                "association"
            } else {
                "dependency"
            };
            let path_name = |node: &LayoutNode| match node.kind {
                EntityKind::Start => "*start*".to_string(),
                EntityKind::End => "*end*".to_string(),
                _ => node.display.clone(),
            };
            let pn_from = path_name(from_node);
            let pn_to = path_name(to_node);
            let path_id = if is_extension {
                format!("{pn_from}-backto-{pn_to}")
            } else if is_association {
                format!("{pn_from}-{pn_to}")
            } else {
                format!("{pn_from}-to-{pn_to}")
            };


            let (path_d, arrow_points) = if let Some(edge_path) =
                dot_result.edges.get(&edge_color)
            {
                // Apply moveDelta to every point.
                let mut pts = shift_points(&edge_path.points, dx, dy);
                let n = pts.len();

                // Orient tail->head: compare start/end against the source and
                // target centres and reverse when the solver stored it upside.
                let dist_normal = (pts[0].0 - from_node.center_x).hypot(pts[0].1 - from_node.center_y)
                    + (pts[n - 1].0 - to_node.center_x).hypot(pts[n - 1].1 - to_node.center_y);
                let dist_reversed = (pts[n - 1].0 - from_node.center_x)
                    .hypot(pts[n - 1].1 - from_node.center_y)
                    + (pts[0].0 - to_node.center_x).hypot(pts[0].1 - to_node.center_y);
                if dist_reversed < dist_normal {
                    pts.reverse();
                }

                let (d, arrow) = emit_edge_path(&pts, is_extension, is_association);
                (d, arrow)
            } else {
                // Fallback: a straight cubic from the source bottom to the
                // target top.
                let pts = [
                    (from_node.center_x, from_node.center_y + from_node.ry),
                    (from_node.center_x, from_node.center_y + from_node.ry),
                    (to_node.center_x, to_node.center_y - to_node.ry),
                    (to_node.center_x, to_node.center_y - to_node.ry),
                ];
                let (d, _) = emit_edge_path(&pts, is_extension, is_association);
                let arrow = compute_arrow_polygon(
                    to_node.center_x,
                    to_node.center_y - to_node.ry,
                    std::f64::consts::FRAC_PI_2,
                );
                (d, arrow)
            };

            // Edge label: a 13px label with a 1px pad, laid in a
            // `UEmpty` spacer whose far corner has no −1. The text starts one
            // pixel right of the recovered label box; the baseline sits
            // 5.397 below the midpoint of the rank gap.
            let (label_anchor, label_spacer_right) = if let Some(text) = &link.label {
                let text_w = measure_text_size(text, LABEL_FONT_SIZE);
                // Graphviz positions the label at its virtual node's rank
                // coordinate; map that y to the final frame. Reconstructing
                // the gap midpoint from the placed node corners folds in a
                // sub-pixel move-delta error for a folded leaf.
                let gap_center =
                    (from_node.y + from_node.height + to_node.y) / 2.0;
                let baseline_y = match dot_result.label_vnode_y.get(&edge_color) {
                    Some(&y) => -y + dy + LABEL_BASELINE_OFFSET,
                    None => gap_center + LABEL_BASELINE_OFFSET,
                };
                let label_x = match dot_result.label_box_left.get(&edge_color) {
                    Some(left) => q2_halfup(via_g5(left + dx)) + 1.0,
                    None => from_node.center_x + 1.0,
                };
                (
                    Some((label_x, baseline_y)),
                    Some(label_x + text_w + 1.0),
                )
            } else {
                (None, None)
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
                label_anchor,
                label_spacer_right,
                arrow: link.arrow.clone(),
            });

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
    let max_right = nodes
        .iter()
        .map(|n| n.x + n.width + max_extent_x(n.kind))
        .fold(0.0_f64, f64::max);
    let max_bottom = nodes
        .iter()
        .map(|n| n.y + n.height + max_extent_y(n.kind))
        .fold(0.0_f64, f64::max);
    // Width and, for every established shape, height use the empirical
    // placed-coordinate margins verified against the Java jar. A database is
    // new: its trailing `UEmpty(10,10)` extends the drawn bounding box by 10,
    // and Smetana sizes the canvas from the resulting structural span plus
    // `margin + 16` (rather than the anchored placed-edge formula). Use that
    // span only when a database participates, so existing diagrams are
    // untouched.
    let node_width = total_margin + max_right.floor();
    // An edge label sits in a `UEmpty` spacer whose far corner has no −1, so
    // it sizes from the 15px delta directly.
    let max_spacer = layout_links
        .iter()
        .filter_map(|l| l.label_spacer_right)
        .fold(0.0_f64, f64::max);
    // A folder cluster sizes the canvas from its own drawn extent: across
    // every jar-rendered variant the edge is 15 past the floor of the far
    // edge (graphviz `bb` + the LimitFinder delta). Take the maximum so the
    let (clust_right, clust_bottom) = clusters.iter().fold((0.0_f64, 0.0_f64), |(r, b), c| {
        (r.max(c.x + c.width), b.max(c.y + c.height))
    });
    // A folded composite node carries an embedded inner `SvekResult` whose
    // far corner is recorded without −1, exactly like an edge-label spacer,
    // so its extent sizes the canvas from the full 15px delta rather than
    // the 14px placed-node margin.
    let max_forced_right = nodes
        .iter()
        .filter(|n| n.forced_size)
        .map(|n| n.x + n.width)
        .fold(0.0_f64, f64::max);
    let total_width = node_width
        .max(15.0 + max_spacer.floor())
        .max(15.0 + max_forced_right.floor())
        .max(if clusters.is_empty() { 0.0 } else { 15.0 + clust_right.floor() });
    let has_database = nodes.iter().any(|n| n.kind == EntityKind::Database);
    let node_height = if has_database {
        let min_draw_y = nodes.iter().map(|n| n.y).fold(f64::INFINITY, f64::min);
        // Structural span plus the top origin (margin) and a 15px bottom delta.
        let span = max_bottom - min_draw_y;
        span.floor() + min_draw_y + 15.0
    } else {
        total_margin + max_bottom.floor()
    };
    let total_height =
        node_height.max(if clusters.is_empty() { 0.0 } else { 15.0 + clust_bottom.floor() });

    CucaLayout {
        nodes,
        clusters,
        links: layout_links,
        total_width,
        total_height,
    }
}


// ── Arrow polygon ───────────────────────────────────────────────────────

/// Round-trips a coordinate through graphviz's `%.5g` token serialization:
/// format to five significant digits then parse back, as Smetana does when it
/// re-reads emitted label positions.
fn via_g5(v: f64) -> f64 {
    format!("{v:.4e}").parse().unwrap_or(v)
}

fn q2_halfup(v: f64) -> f64 {
    let s = v * 100.0;
    // Half-up to an integer: floor(s + 0.5). The epsilon lifts a value that
    // is exactly x.5 but represented just under it.
    let r = if s >= 0.0 { s + 0.5 + 1e-9 } else { s + 0.5 - 1e-9 };
    r.floor() / 100.0
}

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

pub(crate) struct EntityData {
    pub(crate) text_w: f64,
    pub(crate) svek_w: f64,
    pub(crate) svek_h: f64,
    pub(crate) rx: f64,
    pub(crate) ry: f64,
    /// Radius of a centre circle port when the node is an HTML shield table
    /// (component-diagram lollipop); edges clip to this circle, not the box.
    pub(crate) port_circle: Option<f64>,
}








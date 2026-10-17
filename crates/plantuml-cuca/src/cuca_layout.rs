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

/// Measures text width with italic font at the given size (e.g. the 12pt
/// object stereotype).
fn measure_text_size_italic(text: &str, size: i32) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(size).with_style(FontStyle::italic());
    sb.calculate_dimension(&font, text).width()
}

/// Exact AWT advance of `«`/`»` at 12pt italic SansSerif (measured from
/// PlantUML 1.2026.6): `«A»` − `A` = 2 × 5.7241. The Latin-1 fallback
/// table overstates them, so [`measure_object_stereotype`] corrects them.
const OBJECT_STEREO_GUILLEMET_W: f64 = 5.7241;

/// Width of an object's 12pt italic stereotype label (`«name»`), matching
/// Java AWT advances for the guillemets.
pub(crate) fn measure_object_stereotype(label: &str) -> f64 {
    let text = format!("\u{00ab}{label}\u{00bb}");
    let raw = measure_text_size_italic(&text, 12);
    let guillemet = measure_text_size_italic("\u{00ab}", 12);
    raw - 2.0 * guillemet + 2.0 * OBJECT_STEREO_GUILLEMET_W
}

/// Measures text width with bold font (for cluster/package titles).
fn measure_text_bold(text: &str) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(FONT_SIZE).with_style(FontStyle::bold());
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

// ── Object entity geometry ──────────────────────────────────────────────
// Ported from: `svek/image/EntityImageObject.java` (all values measured
// against Java PlantUML 1.2026.6 SVG output).

/// Header height (rect top → separator) for an object without stereotype.
pub(crate) const OBJECT_HEADER_H: f64 = 23.0679;
/// Height added by the 12pt italic stereotype line in the header.
pub(crate) const OBJECT_STEREO_H: f64 = 16.3441;
/// Reserved strip below the separator for an object with no fields
/// (`TextBlockEmpty(10, 16)` wrapped by `TextBlockLineBefore`).
pub(crate) const OBJECT_EMPTY_BODY_H: f64 = 16.0;
/// Top padding below the separator before the first field baseline block.
pub(crate) const OBJECT_BODY_TOP: f64 = 8.0;
/// Object field row pitch (14pt SansSerif AWT line height).
pub(crate) const OBJECT_ROW_H: f64 = 19.0679;

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
            EntityKind::Object => {
                // `EntityImageObject`: width = max(name + 14, stereo + 10,
                // field + 12); height = header (23.0679, +16.3441 with a
                // stereotype) followed by an empty 16px strip or
                // 8 + n_fields * 19.0679 of attribute rows.
                let mut w = text_w + 14.0;
                if let Some(st) = &entity.stereotype {
                    w = w.max(measure_object_stereotype(st) + 10.0);
                }
                for line in &entity.body {
                    let trimmed = line.trim();
                    let rest = match trimmed.chars().next() {
                        Some('+' | '-' | '#') => &trimmed[1..],
                        _ => trimmed,
                    };
                    w = w.max(measure_text(rest) + 12.0);
                }
                let header_h = OBJECT_HEADER_H
                    + if entity.stereotype.is_some() {
                        OBJECT_STEREO_H
                    } else {
                        0.0
                    };
                let body_h = if entity.body.is_empty() {
                    OBJECT_EMPTY_BODY_H
                } else {
                    OBJECT_BODY_TOP + entity.body.len() as f64 * OBJECT_ROW_H
                };
                (w, header_h + body_h, 0.0, 0.0)
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

    // Title line count of each group (`position.c::clust_ht` label border):
    // the display label split on its embedded line breaks, one line minimum.
    let group_lines: HashMap<&String, usize> = sorted_entities
        .iter()
        .filter(|(_, e)| e.group && !e.display.is_empty())
        .map(|(name, e)| {
            let title_lines = e.display.matches('\n').count() + 1;
            (*name, title_lines)
        })
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
    // Select the pure-Rust solver. Graphviz switches to rank doubling for
    // the whole graph whenever any edge carries a label (`lib/dotgen/rank.c
    // ::edgelabel_ranks`). The default `native_layout` reproduces flat
    // diagrams and fans; the doubled-rank `solve_labeled` is bit-exact on
    // linear chains (one real node per longest-path rank) and is required by
    // the autarkic-fold outer frame, which requests it explicitly through
    // `force_labeled`. Keeping forks on the native solver preserves the
    // clean-baseline flat-diagram output.
    let use_labeled = force_labeled
        || crate::native_layout::is_singleton_rank_chain(&sorted_data, &links);
    let native_solve = || {
        if use_labeled {
            crate::labeled_layout::solve_labeled(&sorted_data, &links, nodesep, base_ranksep)
        } else {
            crate::native_layout::solve(&sorted_data, &links, &leaf_group, &group_lines)
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
        &leaf_group,
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
/// Extra LimitFinder minimum-y extent above a placed node's origin, beyond
/// the `URectangle` `y − 1` corner every box already contributes.
///
/// A stereotyped object draws its 12pt stereo label at baseline
/// `y + 12.8281`; `LimitFinder.drawText` records its top at
/// `baseline − (16.344 − 1.5) = y − 2.016`, i.e. 1.016px higher than the
/// box corner. That label sets the graph minimum, shifting every node down
/// by 1.016 (measured against jar 1.2026.6).
pub(crate) fn leaf_min_border_y(entity: &ParsedEntity) -> f64 {
    if matches!(entity.kind, EntityKind::Object) && entity.stereotype.is_some() {
        return -1.016;
    }
    min_border_y(entity.kind)
}

/// LimitFinder maximum-y extent past a placed node's bottom edge.
///
/// An object with no attributes draws only its outline corner at
/// `y + height − 1` (`TextBlockEmpty.drawU` records nothing); once it has a
/// body the marged body block records a far corner at `y + height`. In a
/// single-node diagram the smaller placed-node margin absorbs that one pixel,
/// so the `−1` applies only with two or more nodes (jar-verified).
pub(crate) fn leaf_max_extent_y(entity: &ParsedEntity, multi_node: bool) -> f64 {
    if matches!(entity.kind, EntityKind::Object) {
        return if entity.body.is_empty() && multi_node {
            -1.0
        } else {
            0.0
        };
    }
    if matches!(entity.kind, EntityKind::State) {
        // `EntityImageState.drawU` strokes its rounded rect, whose LimitFinder
        // far corner is `y + height − 1`. Against the multi-node 14px placed
        // margin that already accounts for that −1 no offset is needed; the
        // smaller single-node margin is two pixels short (jar viewBox single
        // Idle = 71 = (7+50−1) + 15).
        return if multi_node { 0.0 } else { 2.0 };
    }
    max_extent_y(entity.kind)
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
        // Both `USymbolDatabase` and `USymbolNode` finish with a translated
        // `UEmpty(10,10)`, whose `drawEmpty` records a point 10px below the
        // symbol box (LimitFinder structural bottom).
        EntityKind::Database | EntityKind::Node => 10.0,
        // A description-diagram interface draws its name below the circle
        // symbol (`EntityImageImageDescription` stacks `asSmall` then `desc`);
        // the label's recorded far corner lands 2px below the 64px shield
        // box after flooring.
        EntityKind::Interface => 2.0,
        _ => 0.0,
    }
}

/// LimitFinder maximum-x extent past a placed node's right edge.
///
/// An empty object draws only its outline corner `x + width − 1`; a body
/// object's marged block reaches `x + width`. Other symbols keep their
/// shape-specific reach (database/node back face). In a single-node diagram
/// the smaller placed-node margin absorbs that one pixel, so the `−1`
/// applies only with two or more nodes (mirrors [`leaf_max_extent_y`]).
pub(crate) fn leaf_max_extent_x(entity: &ParsedEntity, multi_node: bool) -> f64 {
    if matches!(entity.kind, EntityKind::Object) {
        return if entity.body.is_empty() && multi_node {
            -1.0
        } else {
            0.0
        };
    }
    if matches!(entity.kind, EntityKind::State) {
        // A state always draws its internal name separator as
        // `ULine.hline(width)`; LimitFinder.drawULine records the endpoint at
        // the full `x + width` corner, one pixel past the stroked rect's
        // `x + width − 1`. In a single-node diagram the placed margin is two
        // pixels smaller still, so the reach is +3 there (jar single Idle
        // 57 + 15 = 72; two-state 66.34 → 81; labeled cycle 79.06 → 94).
        return if multi_node { 1.0 } else { 3.0 };
    }
    max_extent_x(entity.kind)
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

#[allow(clippy::too_many_arguments, clippy::similar_names)]
fn build_layout_from_dot(
    sorted_entities: &[(&String, &ParsedEntity)],
    entity_data: &HashMap<&String, EntityData>,
    links: &[ParsedLink],

    node_colors: &HashMap<String, i32>,
    dot_result: &DotSvgResult,
    rank_of: &HashMap<&String, usize>,
    leaf_group: &HashMap<&String, &String>,
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

    // Kind of each group (`Node` groups render as a 3D box, others as a
    // rounded folder tab), used for the asymmetric LimitFinder reserve.
    let group_kind: HashMap<&String, EntityKind> =
        group_sorted.iter().map(|(n, e)| (*n, e.kind)).collect();
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
    // Each titled folder is an asymmetric envelope, not a box symmetric about
    // one shared centre. Graphviz wraps a folder body in three nested 8px
    // subgraphs (`p0` > the titled cluster > `p1`); when two sibling folders
    // overlap a rank the simplex compacts their CONTENT boxes together and the
    // inset title-table node overhangs the OUTER edge:
    //   p1 half      = round(widest member half) + 8
    //   facing edge  = column centre +/− (round half + 16)  (toward the
    //                  sibling sharing the rank)
    //   titled width = title table WIDTH + 16, where the HTML table WIDTH is
    //                  floor(bold title width + 60)
    //   outer edge   = facing edge − titled width   (title overhangs outward)
    // Verified against graphviz 14.1.2 `-Tdot` (Application Server 16..220,
    // Database Server 244..434) and four extra title probes for the +60 rule.

    // Column centre of every group in the raw (un-shifted) frame.
    let group_center: HashMap<&String, f64> = {
        let mut sum: HashMap<&String, f64> = HashMap::new();
        let mut cnt: HashMap<&String, u32> = HashMap::new();
        for (name, _) in sorted_entities.iter().filter(|(_, e)| !e.group) {
            if let Some(&g) = leaf_group.get(name) {
                if let Some((x, _, w, _)) = leaf_box_raw.get(name) {
                    *sum.entry(g).or_insert(0.0) += x + w / 2.0;
                    *cnt.entry(g).or_insert(0) += 1;
                }
            }
        }
        sum.into_iter()
            .filter_map(|(g, s)| cnt.get(&g).map(|&c| (g, s / f64::from(c))))
            .collect()
    };

    // Ranks occupied by each group's own leaves, used to detect sibling
    // folders that overlap a rank (and therefore face each other).
    let group_ranks: HashMap<&String, std::collections::HashSet<usize>> = {
        let mut m: HashMap<&String, std::collections::HashSet<usize>> = HashMap::new();
        for (name, _) in sorted_entities.iter().filter(|(_, e)| !e.group) {
            if let Some(&g) = leaf_group.get(name) {
                m.entry(g).or_default().insert(rank_of.get(name).copied().unwrap_or(0));
            }
        }
        m
    };

    let mut member_leaves: Vec<&String> = Vec::new();
    for (gname, gent) in &group_sorted {
        member_leaves.clear();
        collect_member_leaves(gname, sorted_entities, &mut member_leaves);
        let ccx = match group_center.get(gname) {
            // Graphviz's cluster ln/rn bounds round member centres to the
            // integer column (`contain_nodes`), so folder edges land on
            // integers even though the rendered leaf box carries a sub-pixel
            // glyph-width offset (e.g. 244, not 244.0045).
            Some(&c) => c.round(),
            None => continue,
        };
        // Widest member half-width (rounded) and raw vertical extent.
        let mut max_half = 0.0f64;
        let mut min_y = f64::INFINITY;
        let mut max_b = f64::NEG_INFINITY;
        for ln in &member_leaves {
            if let Some((_, y, w, h)) = leaf_box_raw.get(*ln) {
                max_half = max_half.max((w / 2.0).round());
                min_y = min_y.min(*y);
                max_b = max_b.max(y + h);
            }
        }
        if min_y.is_infinite() {
            continue;
        }
        // Inner `p1` content subgraph reaches `round(maxhalf)+8` each side;
        // the titled cluster adds a further 8, so the content-flush base
        // envelope is `ccx ± (p1_half + 8)`.
        let p1_half = max_half + 8.0;
        let base_left = ccx - p1_half - 8.0;
        let base_right = ccx + p1_half + 8.0;
        // Title table WIDTH = floor(bold + 60); the titled envelope must be at
        // least table + 16 wide.
        let bold = gent
            .display
            .split('\n')
            .map(measure_text_bold)
            .fold(0.0_f64, f64::max);
        let title_table = (bold + 60.0).floor();
        let titled_width = (2.0 * p1_half + 16.0).max(title_table + 16.0);
        let is_node_group = gent.kind == EntityKind::Node;
        // A side-by-side folder shares a rank band; only then does the
        // content flush toward its sibling and the title overhang outward.
        let sibling_right = is_node_group
            && group_sorted.iter().any(|(other, _)| {
                other != gname
                    && group_ranks.get(*other).is_some_and(|r| {
                        group_ranks[gname].intersection(r).next().is_some()
                    })
                    && group_center.get(*other).is_some_and(|&oc| oc > ccx)
            });
        let sibling_left = is_node_group
            && group_sorted.iter().any(|(other, _)| {
                other != gname
                    && group_ranks.get(*other).is_some_and(|r| {
                        group_ranks[gname].intersection(r).next().is_some()
                    })
                    && group_center.get(*other).is_some_and(|&oc| oc < ccx)
            });
        let (left, right) = if sibling_right {
            // Content flushes right toward the sibling; the title table
            // overhangs the outer (left) edge.
            (base_right - titled_width, base_right)
        } else if sibling_left {
            // Content flushes left; the title table overhangs the outer edge.
            (base_left, base_left + titled_width)
        } else if is_node_group {
            // A lone deployment node group is a symmetric envelope about the
            // content column, just like a side-by-side one: graphviz sizes
            // the cluster to the HTML title-table WIDTH + 16 and recenters
            // its member on the widened centre (jar: Server 16..136 around a
            // database at x=76). Rounded folders below keep the plain-text
            // content envelope, whose inset label never widens the box.
            (ccx - titled_width / 2.0, ccx + titled_width / 2.0)
        } else {
            // A lone rounded package: a symmetric envelope about the content
            // column. Its plain-text title is inset and never widens the box
            // the way a deployment node-group's HTML `<TABLE>` label does.
            (base_left, base_right)
        };
        let lines = 1 + gent.display.matches('\n').count();
        // A deployment node group is the USymbolNode 3D box: its polygon
        // `top` is the dog-ear apex, 10px above the stroked top edge. The
        // edge clears the topmost member box by 33px (16 cluster margin + the
        // 19px one-line title block, −2), so the apex is 43px above the
        // topmost member (`top` also drives the global y minimum, so the apex
        // always renders at the diagram margin). Rounded folders use 38.
        let top = if is_node_group {
            ((min_y - (16.0 + 19.0 * lines as f64 + 3.0 + 5.0)) * 100.0).round() / 100.0
        } else {
            let top_extent = 16.0 + (19.0 * lines as f64 + 3.0);
            ((min_y - top_extent) * 100.0).round() / 100.0
        };
        // The drawn box keeps the standard 16px cluster margin below the
        // bottommost member for both node groups and folders.
        let bottom_margin = 16.0;
        // graphviz sets the cluster bbox from the RAW (un-quantized) node
        // coordinate: yup bottom = center − (half_h + cluster margin). Using
        // the q2-placed leaf edge folds in a sub-pixel quantization error.
        let bottom_raw = if dot_result.raw_yup.is_empty() {
            max_b + bottom_margin
        } else {
            member_leaves
                .iter()
                .filter_map(|ln| {
                    dot_result
                        .raw_yup
                        .get(*ln)
                        .map(|&(cy, hh)| -cy + hh + bottom_margin)
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
                .map(|(_, gname, left, _, _, _, _, _)| {
                    // A deployment node group carries the 10px USymbolNode
                    // 3D depth in its LimitFinder min-x (the dog-ear flap is
                    // reserved though only the fold is stroked), so the
                    // global minimum reaches 10px left of the drawn box.
                    let border = if group_kind.get(*gname) == Some(&EntityKind::Node) {
                        10.0
                    } else {
                        0.0
                    };
                    *left - border
                })
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
            Some(pos.min_y + leaf_min_border_y(entity))
        })
        .fold(f64::INFINITY, f64::min)
        .min(
            clusters_raw
                .iter()
                .map(|(_, gname, _, top, _, _, _, _)| {
                    if group_kind.get(*gname) == Some(&EntityKind::Node) {
                        // The 3D dog-ear apex (`top`) is the highest stroked
                        // point; unlike the x axis the 10px depth never extends
                        // above it.
                        *top
                    } else {
                        // A rounded-folder label inset node reaches 0.602px
                        // above the drawn origin.
                        top - LABEL_INSET
                    }
                })
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
        // `color_by_li` only contains links whose both endpoints resolve to
        // a known entity; index it after that guard so a dangling link
        // (e.g. the accepted-but-unresolved `object x : T {`) is skipped
        // instead of panicking with an out-of-bounds HashMap index.
        if let (Some(from_node), Some(to_node)) =
            (node_map.get(&link.from), node_map.get(&link.to))
        {
            let edge_color = color_by_li[&li];
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
            // Edge ids use the short `as` code when the entity declares one
            // (`interface "User API" as API` → `Client-to-API`); nested and
            // unaliased leaves keep their display so component-03 is stable.
            let path_name = |node: &LayoutNode| match node.kind {
                EntityKind::Start => "*start*".to_string(),
                EntityKind::End => "*end*".to_string(),
                _ => entity_by_name
                    .get(&node.name)
                    .and_then(|e| e.alias.clone())
                    .unwrap_or_else(|| node.display.clone()),
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

            // Edge label: a 13px label with a 1px pad, laid in a `UEmpty`
            // spacer whose far corner has no -1. The text starts one pixel
            // right of the recovered label box; the baseline sits 5.397
            // below the midpoint of the rank gap.
            let (label_anchor, label_spacer_right) = if let Some(text) = &link.label {
                let text_w = measure_text_size(text, LABEL_FONT_SIZE);
                // Graphviz positions the label at its virtual node's rank
                // coordinate; map that y to the final frame. The gap center
                // uses the 2dp-rounded placed box height because graphviz
                // quantizes node coords before placing the label vnode
                // (verified links 90.967, attrs friend 138.463).
                let gap_center =
                    (from_node.y + q2_halfup(from_node.height) + to_node.y) / 2.0;
                let baseline_y = match dot_result.label_vnode_y.get(&edge_color) {
                    Some(&y) => -y + dy + LABEL_BASELINE_OFFSET,
                    None => gap_center + LABEL_BASELINE_OFFSET,
                };
                // Native edges populate `label_box_left` from the spline's
                // extent across the label-rank band (limitBoxes), exactly as
                // the fold solver does; the vertical HTTPS edge resolves to
                // the column centre through the same anchor channel.
                let label_x = match dot_result.label_box_left.get(&edge_color) {
                    Some(left) => q2_halfup(via_g5(left + dx)) + 1.0,
                    None => q2_halfup(from_node.center_x) + 1.0,
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
    let entity_by_name: HashMap<&str, &ParsedEntity> =
        sorted_entities.iter().map(|(n, e)| (n.as_str(), *e)).collect();
    let total_margin = if nodes.len() > 1 { margin + 8.0 } else { margin + 6.0 };
    let multi_node = nodes.len() > 1;
    let max_right = nodes
        .iter()
        .filter_map(|n| {
            let entity = entity_by_name.get(n.name.as_str())?;
            Some(n.x + n.width + leaf_max_extent_x(entity, multi_node))
        })
        .fold(0.0_f64, f64::max);
    let max_bottom = nodes
        .iter()
        .filter_map(|n| {
            let entity = entity_by_name.get(n.name.as_str())?;
            // An empty object's `TextBlockEmpty` records nothing, so its
            // far corner is the rect edge `h − 1`. In single-node diagrams
            // the smaller placed-node margin absorbs that one pixel; only
            // multi-node layouts subtract it (verified against the jar).
            let extent = leaf_max_extent_y(entity, multi_node);
            Some(n.y + n.height + extent)
        })
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
        // A deployment node group's 10px USymbolNode 3D depth extends beyond
        // the drawn far-right and bottom edges (java canvas reaches those +10).
        let depth = if group_kind.get(&c.name) == Some(&EntityKind::Node) {
            10.0
        } else {
            0.0
        };
        (
            r.max(c.x + c.width + depth),
            b.max(c.y + c.height + depth),
        )
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
    // Height is anchored by the single lowest leaf. A symbol whose tail
    // draws `UEmpty(10,10)` (`USymbolDatabase`, `USymbolNode`) pushes the
    // LimitFinder structural bottom out by 10px, after which Smetana
    // applies the full 15px delta; other shapes size from the placed-edge
    // margin (`total_margin`). floor(max of bottoms) == max of floors, so
    // this is the per-leaf `.floor() + 15` formula (jar: node-lowest 336,
    // database-lowest 297).
    let bottom_leaf_tailed = nodes
        .iter()
        .filter_map(|n| {
            let entity = entity_by_name.get(n.name.as_str())?;
            let extent = leaf_max_extent_y(entity, multi_node);
            Some((
                n.y + n.height + extent,
                matches!(n.kind, EntityKind::Database | EntityKind::Node),
            ))
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .is_some_and(|(_, tailed)| tailed);
    let node_height = if bottom_leaf_tailed {
        max_bottom.floor() + 15.0
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









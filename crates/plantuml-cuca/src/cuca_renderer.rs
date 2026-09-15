//! CucaDiagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/sdot/CucaDiagramFileMakerSmetana.java`
//! and `cucadiagram/` body rendering classes.
//!
//! Renders entities (actors, use cases, boxes) and links as SVG, matching
//! Java PlantUML's Smetana engine output.

use std::fmt::Write;
use std::collections::HashMap;

use plantuml_core::string_bounder::StringBounder;
use plantuml_core::file_format::FileFormat;
use plantuml_core::u_font::{FontStyle, UFont};
use plantuml_klimt::string_bounder_svg::StringBounderSvg;
use plantuml_svg::{SvgGraphics, SvgOption};

use crate::cuca_layout::{CucaLayout, LayoutLink, LayoutNode};
use crate::entity_link_parser::{EntityKind, ParsedEntity, ParsedNote};

// ── Rendering constants ──────────────────────────────────────────────────

/// Stroke color for entity outlines.
const STROKE_COLOR: &str = "#181818";
/// Fill color for description-diagram entities (use cases, actors).
const FILL_DESC: &str = "#F1F1F1";
/// Text color.
const COLOR_TEXT: &str = "#000000";
/// Stroke color for links.
const STROKE_LINK: &str = "#181818";
/// Stroke width for description-diagram entities.
const STROKE_WIDTH_DESC: f64 = 0.5;
/// Stroke width for links.
const STROKE_WIDTH_LINK: f64 = 1.0;
/// Font size for description-diagram entity labels.
const FONT_SIZE_DESC: i32 = 14;
/// Font family.
const FONT_FAMILY: &str = "sans-serif";

// ── Box entity constants (class, state, component, etc.) ─────────────────

/// Fill color for class/interface/abstract boxes.
const FILL_CLASS: &str = "#F1F1F1";
/// Fill color for state boxes.
const FILL_STATE: &str = "#F1F1F1";
/// Fill color for component boxes.
const FILL_COMPONENT: &str = "#F1F1F1";
/// Fill color for notes.
const FILL_NOTE: &str = "#FBFB77";
/// Stroke width for boxes (matches Java's 0.5).
const STROKE_WIDTH_BOX: f64 = 0.5;
/// Font size for entity names (matches Java's 14).
const FONT_SIZE_NAME: i32 = 14;
/// Font size for body text (matches Java's 14).
const FONT_SIZE_BODY: i32 = 14;
/// Line height for body text = font_size * 1.361994 (Java AWT FontMetrics).
const LINE_HEIGHT: f64 = 14.0 * 1.361_994;
/// Font ascent for 14pt SansSerif (derived from Java AWT FontMetrics).
const ASCENT: f64 = 14.966;
/// Body top margin (from `TextBlockUtils.withMargin(this, 6, 4)`).
const BODY_TOP_MARGIN: f64 = 4.0;
/// Body text y offset from separator line: BODY_TOP_MARGIN + ASCENT - LINE_HEIGHT.
const BODY_TEXT_Y_OFFSET: f64 = BODY_TOP_MARGIN + ASCENT - LINE_HEIGHT;
/// Visibility icon block height = classAttributeIconSize + 1 = 11.
const ICON_BLOCK_HEIGHT: f64 = 11.0;
/// Visibility icon y offset from body content top: 2 + (LINE_HEIGHT - ICON_BLOCK_HEIGHT) / 2.
const VIS_ICON_Y_OFFSET: f64 = 2.0 + (LINE_HEIGHT - ICON_BLOCK_HEIGHT) / 2.0;
/// Circle center offset within icon block (drawCircle at (2,2), center at +3).
const VIS_CIRCLE_CENTER_OFFSET: f64 = 5.0;
/// Square top offset within icon block (drawSquare at (2,2)).
const VIS_SQUARE_TOP_OFFSET: f64 = 2.0;
/// Diamond center offset within icon block (drawDiamond at (1,0), center at +4).
const VIS_DIAMOND_CENTER_OFFSET: f64 = 4.0;
/// Border radius for entity boxes (matches Java's 2.5).
const BOX_RX: f64 = 2.5;
/// Icon radius (matches Java's CircledCharacter radius=11).
const ICON_RADIUS: f64 = 11.0;
/// Icon center offset from entity top-left (cx=x+15, cy=y+16).
const ICON_OFFSET_X: f64 = 15.0;
const ICON_OFFSET_Y: f64 = 16.0;
/// Text start x offset from entity left (icon diameter + gap).
const TEXT_OFFSET_X: f64 = 29.0;
/// Text baseline y offset from entity top.
const TEXT_BASELINE_Y: f64 = 21.432;
/// Separator line y offset from entity top (header height).
const SEP_LINE1_Y: f64 = 32.0;
/// Gap between fields and methods separator.
const FIELD_METHOD_GAP: f64 = 8.0;

// ── Icon colors ──────────────────────────────────────────────────────────

/// Icon fill color for class entities (spotClass).
const ICON_COLOR_CLASS: &str = "#ADD1B2";
/// Icon fill color for interface entities (spotInterface).
const ICON_COLOR_INTERFACE: &str = "#B4A7E5";
/// Icon fill color for abstract entities (spotAbstractClass).
const ICON_COLOR_ABSTRACT: &str = "#A9DCDF";

// ── Visibility modifier colors ───────────────────────────────────────────

const VIS_PUBLIC_STROKE: &str = "#038048";
const VIS_PUBLIC_FILL: &str = "#84BE84";
const VIS_PRIVATE_STROKE: &str = "#C82930";
const VIS_PRIVATE_FILL: &str = "#F24D5C";
const VIS_PROTECTED_STROKE: &str = "#B38D22";

/// Renders a CucaDiagram layout as an SVG string.
///
/// Ported from: `CucaDiagramFileMakerSmetana.drawU()`.
#[must_use]
pub fn render_cuca_svg(
    layout: &CucaLayout,
    entities: &HashMap<String, ParsedEntity>,
    notes: &[ParsedNote],
    diagram_type: plantuml_core::DiagramType,
) -> String {
    let mut option = SvgOption::basic();
    option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    let type_name = match diagram_type {
        plantuml_core::DiagramType::Class => "CLASS",
        plantuml_core::DiagramType::Object => "OBJECT",
        plantuml_core::DiagramType::State => "STATE",
        _ => "DESCRIPTION",
    };
    option.set_root_attribute("data-diagram-type", type_name);

    let mut svg = SvgGraphics::new(0, option);

    // Set SVG dimensions via a hidden background rectangle.
    svg.set_hidden(true);
    svg.svg_rectangle(0.0, 0.0, layout.total_width, layout.total_height, 0.0, 0.0, 0.0);
    svg.set_hidden(false);

    // Render entities first (Java renders entities before links).
    for node in &layout.nodes {
        if node.name == "[*]" {
            render_star_state(&mut svg, node, entities);
        } else if let Some(entity) = entities.get(&node.name) {
            render_entity(&mut svg, node, entity);
        }
    }

    // Render links.
    for link in &layout.links {
        render_link(&mut svg, link, diagram_type);
    }

    // Render notes.
    for (i, note) in notes.iter().enumerate() {
        render_note(&mut svg, note, &layout.nodes, i);
    }

    svg.create_xml()
}

/// Formats a coordinate like Java's `%.4f` with trailing zeros and decimal point stripped.
///
/// Java outputs `22.0000` as `22`, `84.9389` as `84.9389`, `69.2500` as `69.25`.
fn fmt_coord(v: f64) -> String {
    let s = format!("{v:.4}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

// ── Entity rendering ─────────────────────────────────────────────────────

/// Renders an entity with a wrapper `<g>` group.
fn render_entity(svg: &mut SvgGraphics, node: &LayoutNode, entity: &ParsedEntity) {
    let attrs: [(&str, &str); 4] = [
        ("class", "entity"),
        ("data-qualified-name", &entity.display),
        ("data-source-line", &node.source_line.to_string()),
        ("id", &node.entity_id),
    ];
    svg.open_group_with_attrs(&attrs);

    match entity.kind {
        EntityKind::Actor => render_actor(svg, node, entity),
        EntityKind::Usecase => render_usecase(svg, node, entity),
        _ => render_box_entity(svg, node, entity),
    }

    svg.close_group();
}

/// Renders a use case as an ellipse with centered text.
///
/// Ported from: `TextBlockInEllipse.drawU()` and `USymbolUsecase.java`.
fn render_usecase(svg: &mut SvgGraphics, node: &LayoutNode, entity: &ParsedEntity) {
    let cx = node.center_x;
    let cy = node.center_y;
    let rx = node.rx;
    let ry = node.ry;

    // Ellipse.
    svg.set_fill_color(FILL_DESC);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_DESC, None);
    svg.svg_ellipse(cx, cy, rx, ry, 0.0);

    // Text.
    let label = &entity.display;
    let text_w = node.text_width;
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        label,
        node.text_x,
        node.text_y,
        Some(FONT_FAMILY),
        FONT_SIZE_DESC,
        None,
        None,
        None,
        text_w,
        &attrs,
        None,
    );
}

/// Renders an actor as a stick figure with label below.
///
/// Ported from: `ActorStickMan.java` and `USymbolActor.java`.
fn render_actor(svg: &mut SvgGraphics, node: &LayoutNode, entity: &ParsedEntity) {
    let cx = node.center_x;
    let head_cy = node.center_y;
    let head_r = 8.0;

    // Head (ellipse).
    svg.set_fill_color(FILL_DESC);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_DESC, None);
    svg.svg_ellipse(cx, head_cy, head_r, head_r, 0.0);

    // Body: vertical line, arms, legs.
    // Constants from ActorStickMan.java:
    //   headDiam=16, bodyLenght=27, armsLenght=13, legsX=13, legsY=15, armsY=8
    //   head center at (cx, head_cy), head bottom = head_cy + 8
    //   body_top = head_cy + 8 (= headDiam/2)
    //   body_bottom = body_top + 27 (= bodyLenght)
    //   arms_y = body_top + 8 (= armsY)
    //   legs_bottom = body_bottom + 15 (= legsY)
    let body_top = head_cy + 8.0;
    let body_bottom = body_top + 27.0;
    let arms_y = body_top + 8.0;
    let legs_bottom = body_bottom + 15.0;

    svg.set_fill_color("none");
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_DESC, None);
    let path_d = format!(
        "M{},{} L{},{} \
         M{},{} L{},{} \
         M{},{} L{},{} \
         M{},{} L{},{}",
        fmt_coord(cx), fmt_coord(body_top), fmt_coord(cx), fmt_coord(body_bottom),
        fmt_coord(cx - 13.0), fmt_coord(arms_y), fmt_coord(cx + 13.0), fmt_coord(arms_y),
        fmt_coord(cx), fmt_coord(body_bottom), fmt_coord(cx - 13.0), fmt_coord(legs_bottom),
        fmt_coord(cx), fmt_coord(body_bottom), fmt_coord(cx + 13.0), fmt_coord(legs_bottom),
    );
    svg.svg_path(&path_d, 0.0);

    // Label below the figure.
    let label = &entity.display;
    let text_w = node.text_width;
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        label,
        node.text_x,
        node.text_y,
        Some(FONT_FAMILY),
        FONT_SIZE_DESC,
        None,
        None,
        None,
        text_w,
        &attrs,
        None,
    );
}

/// Icon path segment: command + relative offsets from ellipse center.
struct IconPathSegment {
    cmd: &'static str,
    offsets: &'static [(f64, f64)],
}

/// Class "C" icon path (relative to ellipse center).
/// Extracted from Java PlantUML 1.2026.6 SVG output.
const CLASS_ICON_PATH: &[IconPathSegment] = &[
    IconPathSegment { cmd: "M", offsets: &[(2.9688, 5.6406)] },
    IconPathSegment { cmd: "Q", offsets: &[(2.3906, 5.9375), (1.75, 6.0781)] },
    IconPathSegment { cmd: "Q", offsets: &[(1.1094, 6.2344), (0.4063, 6.2344)] },
    IconPathSegment { cmd: "Q", offsets: &[(-2.0937, 6.2344), (-3.4219, 4.5938)] },
    IconPathSegment { cmd: "Q", offsets: &[(-4.7344, 2.9375), (-4.7344, -0.1875)] },
    IconPathSegment { cmd: "Q", offsets: &[(-4.7344, -3.3125), (-3.4219, -4.9687)] },
    IconPathSegment { cmd: "Q", offsets: &[(-2.0937, -6.625), (0.4063, -6.625)] },
    IconPathSegment { cmd: "Q", offsets: &[(1.1094, -6.625), (1.75, -6.4687)] },
    IconPathSegment { cmd: "Q", offsets: &[(2.4063, -6.3125), (2.9688, -6.0156)] },
    IconPathSegment { cmd: "L", offsets: &[(2.9688, -3.2969)] },
    IconPathSegment { cmd: "Q", offsets: &[(2.3438, -3.875), (1.75, -4.1406)] },
    IconPathSegment { cmd: "Q", offsets: &[(1.1563, -4.4219), (0.5313, -4.4219)] },
    IconPathSegment { cmd: "Q", offsets: &[(-0.8125, -4.4219), (-1.5, -3.3437)] },
    IconPathSegment { cmd: "Q", offsets: &[(-2.1875, -2.2812), (-2.1875, -0.1875)] },
    IconPathSegment { cmd: "Q", offsets: &[(-2.1875, 1.9063), (-1.5, 2.9844)] },
    IconPathSegment { cmd: "Q", offsets: &[(-0.8125, 4.0469), (0.5313, 4.0469)] },
    IconPathSegment { cmd: "Q", offsets: &[(1.1563, 4.0469), (1.75, 3.7813)] },
    IconPathSegment { cmd: "Q", offsets: &[(2.3438, 3.5), (2.9688, 2.9219)] },
    IconPathSegment { cmd: "L", offsets: &[(2.9688, 5.6406)] },
    IconPathSegment { cmd: "Z", offsets: &[] },
];

/// Interface "I" icon path (relative to ellipse center).
const INTERFACE_ICON_PATH: &[IconPathSegment] = &[
    IconPathSegment { cmd: "M", offsets: &[(-4.0781, -4.2344)] },
    IconPathSegment { cmd: "L", offsets: &[(-4.0781, -6.3906)] },
    IconPathSegment { cmd: "L", offsets: &[(3.3125, -6.3906)] },
    IconPathSegment { cmd: "L", offsets: &[(3.3125, -4.2344)] },
    IconPathSegment { cmd: "L", offsets: &[(0.8438, -4.2344)] },
    IconPathSegment { cmd: "L", offsets: &[(0.8438, 3.8438)] },
    IconPathSegment { cmd: "L", offsets: &[(3.3125, 3.8438)] },
    IconPathSegment { cmd: "L", offsets: &[(3.3125, 6.0)] },
    IconPathSegment { cmd: "L", offsets: &[(-4.0781, 6.0)] },
    IconPathSegment { cmd: "L", offsets: &[(-4.0781, 3.8438)] },
    IconPathSegment { cmd: "L", offsets: &[(-1.6094, 3.8438)] },
    IconPathSegment { cmd: "L", offsets: &[(-1.6094, -4.2344)] },
    IconPathSegment { cmd: "L", offsets: &[(-4.0781, -4.2344)] },
    IconPathSegment { cmd: "Z", offsets: &[] },
];

/// Abstract "A" icon path (relative to ellipse center).
const ABSTRACT_ICON_PATH: &[IconPathSegment] = &[
    IconPathSegment { cmd: "M", offsets: &[(0.1094, -4.6562)] },
    IconPathSegment { cmd: "L", offsets: &[(-1.0469, 0.4219)] },
    IconPathSegment { cmd: "L", offsets: &[(1.2813, 0.4219)] },
    IconPathSegment { cmd: "L", offsets: &[(0.1094, -4.6562)] },
    IconPathSegment { cmd: "Z", offsets: &[] },
    IconPathSegment { cmd: "M", offsets: &[(-1.375, -6.8906)] },
    IconPathSegment { cmd: "L", offsets: &[(1.6094, -6.8906)] },
    IconPathSegment { cmd: "L", offsets: &[(4.9688, 5.5)] },
    IconPathSegment { cmd: "L", offsets: &[(2.5156, 5.5)] },
    IconPathSegment { cmd: "L", offsets: &[(1.75, 2.4375)] },
    IconPathSegment { cmd: "L", offsets: &[(-1.5312, 2.4375)] },
    IconPathSegment { cmd: "L", offsets: &[(-2.2812, 5.5)] },
    IconPathSegment { cmd: "L", offsets: &[(-4.7188, 5.5)] },
    IconPathSegment { cmd: "L", offsets: &[(-1.375, -6.8906)] },
    IconPathSegment { cmd: "Z", offsets: &[] },
];

/// Generates an SVG path string from icon path segments, translated to (cx, cy).
fn generate_icon_path(segments: &[IconPathSegment], cx: f64, cy: f64) -> String {
    let mut s = String::new();
    for seg in segments {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(seg.cmd);
        for (i, (dx, dy)) in seg.offsets.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            let _ = write!(s, "{},{}", fmt_coord(cx + dx), fmt_coord(cy + dy));
        }
    }
    s.push(' ');
    s
}

/// Renders the entity icon (colored ellipse + letter path).
/// `cx` is the absolute x coordinate of the icon center.
fn render_entity_icon(svg: &mut SvgGraphics, cx: f64, node: &LayoutNode, entity: &ParsedEntity) {
    let cy = node.y + ICON_OFFSET_Y;

    let (fill_color, path_data) = match entity.kind {
        EntityKind::Interface => (ICON_COLOR_INTERFACE, generate_icon_path(INTERFACE_ICON_PATH, cx, cy)),
        EntityKind::Abstract => (ICON_COLOR_ABSTRACT, generate_icon_path(ABSTRACT_ICON_PATH, cx, cy)),
        _ => (ICON_COLOR_CLASS, generate_icon_path(CLASS_ICON_PATH, cx, cy)),
    };

    // Draw icon ellipse.
    svg.set_fill_color(fill_color);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(1.0, None);
    svg.svg_ellipse(cx, cy, ICON_RADIUS, ICON_RADIUS, 0.0);

    // Draw icon letter path.
    svg.set_fill_color(COLOR_TEXT);
    svg.set_stroke_color(None);
    svg.set_stroke_width(0.0, None);
    svg.svg_path(&path_data, 0.0);
}

/// Renders a visibility modifier icon for a body line.
/// `text_y` is the text baseline y. The icon y is computed from `text_y` using
/// Java's PlacementStrategyVisibility offsets.
/// `entity_x` is the entity's absolute x (for computing icon x positions).
fn render_visibility_icon(
    svg: &mut SvgGraphics,
    vis: char,
    is_method: bool,
    entity_x: f64,
    text_y: f64,
) {
    // Compute the visibility modifier label for the <g> wrapper.
    let vis_label = match (vis, is_method) {
        ('+', false) => "PUBLIC_FIELD",
        ('-', false) => "PRIVATE_FIELD",
        ('#', false) => "PROTECTED_FIELD",
        ('+', true) => "PUBLIC_METHOD",
        ('-', true) => "PRIVATE_METHOD",
        ('#', true) => "PROTECTED_METHOD",
        _ => "",
    };

    if vis_label.is_empty() {
        return;
    }

    // Open <g data-visibility-modifier="..."> group.
    svg.open_group_with_attrs(&[("data-visibility-modifier", vis_label)]);

    // Icon x positions: circle/diamond cx = entity_x + 11, square x = entity_x + 8.
    // Icon y positions derived from Java's PlacementStrategyVisibility:
    //   vis_y = VIS_ICON_Y_OFFSET + i * LINE_HEIGHT (relative to body content)
    //   circle cy = vis_y + VIS_CIRCLE_CENTER_OFFSET
    //   square y = vis_y + VIS_SQUARE_TOP_OFFSET
    //   diamond cy = vis_y + VIS_DIAMOND_CENTER_OFFSET
    // And text_y = body_content_y + i * LINE_HEIGHT + ASCENT
    // So: circle cy = text_y - ASCENT + VIS_ICON_Y_OFFSET + VIS_CIRCLE_CENTER_OFFSET
    //              = text_y - (ASCENT - VIS_ICON_Y_OFFSET - VIS_CIRCLE_CENTER_OFFSET)
    let vis_circle_cy = text_y - (ASCENT - VIS_ICON_Y_OFFSET - VIS_CIRCLE_CENTER_OFFSET);
    let vis_square_y = text_y - (ASCENT - VIS_ICON_Y_OFFSET - VIS_SQUARE_TOP_OFFSET);
    let vis_diamond_cy = text_y - (ASCENT - VIS_ICON_Y_OFFSET - VIS_DIAMOND_CENTER_OFFSET);

    match vis {
        '+' => {
            // Public: circle (ellipse rx=3 ry=3)
            let fill = if is_method { VIS_PUBLIC_FILL } else { "none" };
            svg.set_fill_color(fill);
            svg.set_stroke_color(Some(VIS_PUBLIC_STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_ellipse(entity_x + 11.0, vis_circle_cy, 3.0, 3.0, 0.0);
        }
        '-' => {
            // Private: square (rect 6x6)
            let fill = if is_method { VIS_PRIVATE_FILL } else { "none" };
            svg.set_fill_color(fill);
            svg.set_stroke_color(Some(VIS_PRIVATE_STROKE));
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(entity_x + 8.0, vis_square_y, 6.0, 6.0, 0.0, 0.0, 0.0);
        }
        '#' => {
            // Protected: diamond (polygon)
            svg.set_fill_color("none");
            svg.set_stroke_color(Some(VIS_PROTECTED_STROKE));
            svg.set_stroke_width(1.0, None);
            let cx = entity_x + 11.0;
            let cy = vis_diamond_cy;
            let points: Vec<f64> = vec![cx, cy - 4.0, cx + 4.0, cy, cx, cy + 4.0, cx - 4.0, cy];
            svg.svg_polygon(0.0, &points);
        }
        _ => {}
    }

    // Close <g> group.
    svg.close_group();
}

/// Parses a body line to extract visibility modifier, text, and whether it's a method.
fn parse_body_line(line: &str) -> (char, &str, bool) {
    let trimmed = line.trim();
    let (vis, rest) = match trimmed.chars().next() {
        Some('+' | '-' | '#') => {
            (trimmed.chars().next().unwrap(), trimmed[1..].trim())
        }
        _ => (' ', trimmed),
    };
    let is_method = rest.contains('(');
    (vis, rest, is_method)
}

/// Renders a regular box entity (class, component, state, etc.).
/// Matches Java PlantUML's Smetana entity rendering structure.
fn render_box_entity(svg: &mut SvgGraphics, node: &LayoutNode, entity: &ParsedEntity) {
    let fill = match entity.kind {
        EntityKind::State => FILL_STATE,
        EntityKind::Component | EntityKind::Node | EntityKind::Database => FILL_COMPONENT,
        _ => FILL_CLASS,
    };

    // Draw box rect.
    svg.set_fill_color(fill);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_BOX, None);
    svg.svg_rectangle(node.x, node.y, node.width, node.height, BOX_RX, BOX_RX, 0.0);

    // Compute icon and name text positions using HeaderLayout formula.
    // When entity has body and entity_width > header_width, icon and text are centered.
    // circleDim = (26, 32) — icon (diameter 22) + margin (4, 0, 5, 5).
    // nameDim = (text_width + 6, 19.0679) — text + margin (3, 3, 0, 0).
    // headerWidth = 26 + text_width + 6 = 32 + text_width.
    let name_w = entity_text_width(&entity.display, FONT_SIZE_NAME, entity.kind);
    let name_text_w = entity_text_width(&entity.display, FONT_SIZE_NAME, entity.kind);
    let header_width = 32.0 + name_text_w;

    let (icon_cx, name_x) = if !entity.body.is_empty() && node.width > header_width {
        // HeaderLayout centering.
        let supp_with = node.width - header_width;
        let circle_dim_w: f64 = 26.0;
        let h2 = (circle_dim_w / 4.0).min(supp_with * 0.1);
        let h1 = (supp_with - h2) / 2.0;
        let icon_cx = node.x + h1 + ICON_OFFSET_X;
        let name_x = node.x + TEXT_OFFSET_X + h1 + h2;
        (icon_cx, name_x)
    } else {
        // Fixed offsets (no body or entity not wider than header).
        (node.x + ICON_OFFSET_X, node.x + TEXT_OFFSET_X)
    };

    // Draw entity icon (ellipse + letter path) for class/interface/abstract.
    match entity.kind {
        EntityKind::Class | EntityKind::Interface | EntityKind::Abstract => {
            render_entity_icon(svg, icon_cx, node, entity);
        }
        _ => {}
    }

    // Draw entity name text.
    let name_y = node.y + TEXT_BASELINE_Y;
    let font_style = match entity.kind {
        EntityKind::Interface | EntityKind::Abstract => Some("italic"),
        _ => None,
    };
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        &entity.display,
        name_x,
        name_y,
        Some(FONT_FAMILY),
        FONT_SIZE_NAME,
        None,
        font_style,
        None,
        name_w,
        &attrs,
        None,
    );

    // Draw separator lines.
    let sep1_y = node.y + SEP_LINE1_Y;
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_BOX, None);

    if entity.body.is_empty() {
        // Empty body: two separator lines (fields separator + methods separator).
        let sep2_y = node.y + SEP_LINE1_Y + 8.0;
        svg.svg_line(node.x + 1.0, sep1_y, node.x + node.width - 1.0, sep1_y, 0.0);
        svg.svg_line(node.x + 1.0, sep2_y, node.x + node.width - 1.0, sep2_y, 0.0);
    } else {
        // First separator line (between name and body).
        svg.svg_line(node.x + 1.0, sep1_y, node.x + node.width - 1.0, sep1_y, 0.0);

        // Split body into fields and methods.
        let mut fields: Vec<&str> = Vec::new();
        let mut methods: Vec<&str> = Vec::new();
        for line in &entity.body {
            let (_, _, is_method) = parse_body_line(line);
            if is_method {
                methods.push(line);
            } else {
                fields.push(line);
            }
        }

        // Render fields.
        // Body text y = sep_y + (i+1) * LINE_HEIGHT + BODY_TEXT_Y_OFFSET
        // where BODY_TEXT_Y_OFFSET = BODY_TOP_MARGIN + ASCENT - LINE_HEIGHT = -0.102
        for (i, line) in fields.iter().enumerate() {
            let (vis, text, is_method) = parse_body_line(line);
            let line_y = sep1_y + (i as f64 + 1.0) * LINE_HEIGHT + BODY_TEXT_Y_OFFSET;

            // Visibility icon.
            render_visibility_icon(svg, vis, is_method, node.x, line_y);

            // Body text (without visibility prefix).
            let text_w = text_width(text, FONT_SIZE_BODY);
            let mut body_attrs = indexmap::IndexMap::new();
            body_attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
            svg.text(
                text,
                node.x + 20.0,
                line_y,
                Some(FONT_FAMILY),
                FONT_SIZE_BODY,
                None,
                None,
                None,
                text_w,
                &body_attrs,
                None,
            );
        }

        // Separator between fields and methods (if both exist).
        if !fields.is_empty() && !methods.is_empty() {
            let sep_fm_y = sep1_y + fields.len() as f64 * LINE_HEIGHT + FIELD_METHOD_GAP;
            svg.set_stroke_color(Some(STROKE_COLOR));
            svg.set_stroke_width(STROKE_WIDTH_BOX, None);
            svg.svg_line(node.x + 1.0, sep_fm_y, node.x + node.width - 1.0, sep_fm_y, 0.0);
        }

        // Render methods.
        let methods_start_y = if fields.is_empty() {
            sep1_y
        } else {
            sep1_y + fields.len() as f64 * LINE_HEIGHT + FIELD_METHOD_GAP
        };
        for (i, line) in methods.iter().enumerate() {
            let (vis, text, is_method) = parse_body_line(line);
            let line_y = methods_start_y + (i as f64 + 1.0) * LINE_HEIGHT + BODY_TEXT_Y_OFFSET;

            // Visibility icon.
            render_visibility_icon(svg, vis, is_method, node.x, line_y);

            // Body text.
            let text_w = text_width(text, FONT_SIZE_BODY);
            let mut body_attrs = indexmap::IndexMap::new();
            body_attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
            svg.text(
                text,
                node.x + 20.0,
                line_y,
                Some(FONT_FAMILY),
                FONT_SIZE_BODY,
                None,
                None,
                None,
                text_w,
                &body_attrs,
                None,
            );
        }
    }
}

/// Renders a `[*]` initial/final state pseudo-entity.
fn render_star_state(
    svg: &mut SvgGraphics,
    node: &LayoutNode,
    _entities: &HashMap<String, ParsedEntity>,
) {
    let cx = node.center_x;
    let cy = node.center_y;
    let r = 7.0;

    svg.set_fill_color("#000000");
    svg.set_stroke_color(Some("#000000"));
    svg.set_stroke_width(1.0, None);
    svg.svg_ellipse(cx, cy, r, r, 0.0);
}

// ── Link rendering ──────────────────────────────────────────────────────

/// Renders a link (arrow) between two nodes.
///
/// Ported from: `SmetanaEdge.drawU()` and `ExtremityArrow.java`.
fn render_link(svg: &mut SvgGraphics, link: &LayoutLink, diagram_type: plantuml_core::DiagramType) {
    let is_extension = link.link_type == "extension";
    let is_dashed = link.arrow.contains("..");

    let attrs: [(&str, &str); 5] = [
        ("class", "link"),
        ("data-entity-1", &link.from_id),
        ("data-entity-2", &link.to_id),
        ("data-link-type", &link.link_type),
        ("data-source-line", &link.source_line.to_string()),
    ];
    let mut group_attrs = attrs.to_vec();
    group_attrs.push(("id", &link.link_id));
    svg.open_group_with_attrs(&group_attrs);

    // Draw link path.
    svg.set_fill_color("none");
    svg.set_stroke_color(Some(STROKE_LINK));
    svg.set_stroke_width(STROKE_WIDTH_LINK, None);
    if is_dashed {
        svg.set_stroke_dasharray_str(Some("7,7"));
    } else {
        svg.set_stroke_dasharray_str(None);
    }
    if link.is_line {
        svg.svg_line(link.start.0, link.start.1, link.end.0, link.end.1, 0.0);
    } else if matches!(diagram_type, plantuml_core::DiagramType::Class) {
        svg.svg_path_with_id_codeline(&link.path_d, &link.path_id, link.source_line as u32, 0.0);
    } else {
        svg.svg_path_with_id(&link.path_d, &link.path_id, 0.0);
    }
    svg.set_stroke_dasharray_str(None);

    // Arrow polygon: hollow triangle for extension, filled for dependency.
    let points: Vec<f64> = link
        .arrow_points
        .split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    if is_extension {
        svg.set_fill_color("none");
    } else {
        svg.set_fill_color(STROKE_LINK);
    }
    svg.set_stroke_color(Some(STROKE_LINK));
    svg.set_stroke_width(STROKE_WIDTH_LINK, None);
    svg.svg_polygon(0.0, &points);

    // Render link label at midpoint.
    if let Some(ref label) = link.label {
        let mid_x = (link.start.0 + link.end.0) / 2.0;
        let mid_y = (link.start.1 + link.end.1) / 2.0 - 5.0;
        let label_w = text_width(label, FONT_SIZE_BODY);
        let mut label_attrs = indexmap::IndexMap::new();
        label_attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
        svg.text(
            label,
            mid_x,
            mid_y,
            Some(FONT_FAMILY),
            FONT_SIZE_BODY,
            None,
            None,
            None,
            label_w,
            &label_attrs,
            None,
        );
    }

    svg.close_group();
}

// ── Note rendering ──────────────────────────────────────────────────────

/// Renders a note.
fn render_note(svg: &mut SvgGraphics, note: &ParsedNote, nodes: &[LayoutNode], index: usize) {
    let target = note.target.as_ref().and_then(|name| {
        nodes.iter().find(|n| &n.name == name)
    });

    let (nx, ny) = if let Some(t) = target {
        match note.position {
            crate::entity_link_parser::NotePosition::Left => (t.x - 120.0, t.y),
            crate::entity_link_parser::NotePosition::Right => (t.x + t.width + 10.0, t.y),
            crate::entity_link_parser::NotePosition::Top => (t.x, t.y - 40.0),
            crate::entity_link_parser::NotePosition::Bottom => (t.x, t.y + t.height + 10.0),
        }
    } else {
        (400.0 + (index as f64) * 130.0, 10.0)
    };

    let nw = 100.0;
    let nh = 30.0;

    svg.set_fill_color(FILL_NOTE);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_BOX, None);
    svg.svg_rectangle(nx, ny, nw, nh, 0.0, 0.0, 0.0);

    let text_w = text_width(&note.text, FONT_SIZE_BODY);
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        &note.text,
        nx + 5.0,
        ny + (FONT_SIZE_BODY as f64) * 0.85,
        Some(FONT_FAMILY),
        FONT_SIZE_BODY,
        None,
        None,
        None,
        text_w,
        &attrs,
        None,
    );
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Measures text width using StringBounderSvg (matches Java AWT).
fn text_width(text: &str, font_size: i32) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(font_size);
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

/// Measures text width with italic font (for interface/abstract entity names).
fn text_width_italic(text: &str, font_size: i32) -> f64 {
    let sb = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(font_size).with_style(FontStyle::italic());
    let dim = sb.calculate_dimension(&font, text);
    dim.width()
}

/// Returns the appropriate text width based on entity kind.
fn entity_text_width(text: &str, font_size: i32, kind: EntityKind) -> f64 {
    match kind {
        EntityKind::Interface | EntityKind::Abstract => text_width_italic(text, font_size),
        _ => text_width(text, font_size),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cuca_layout::compute_layout;
    use plantuml_core::string_bounder::StringBounder;
    #[test]
    fn render_empty_layout() {
        let layout = CucaLayout {
            nodes: vec![],
            links: vec![],
            total_width: 20.0,
            total_height: 20.0,
        };
        let entities = HashMap::new();
        let svg = render_cuca_svg(
            &layout,
            &entities,
            &[],
            plantuml_core::DiagramType::Description,
        );
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn render_single_usecase() {
        let mut entities = HashMap::new();
        entities.insert(
            "Login".to_string(),
            ParsedEntity {
                name: "Login".to_string(),
                display: "Login".to_string(),
                kind: EntityKind::Usecase,
                stereotype: None,
                body: vec![],
                source_line: 1,
            },
        );
        let layout = compute_layout(&entities, &[], plantuml_core::DiagramType::Description);
        let svg = render_cuca_svg(
            &layout,
            &entities,
            &[],
            plantuml_core::DiagramType::Description,
        );
        assert!(svg.contains("ellipse"));
        assert!(svg.contains("Login"));
    }

    fn make_class_entity(name: &str, body: Vec<&str>) -> ParsedEntity {
        ParsedEntity {
            name: name.to_string(),
            display: name.to_string(),
            kind: EntityKind::Class,
            stereotype: None,
            body: body.iter().map(|s| s.to_string()).collect(),
            source_line: 1,
        }
    }

    #[test]
    fn render_single_class_box() {
        let mut entities = HashMap::new();
        entities.insert("Foo".to_string(), make_class_entity("Foo", vec![]));
        let layout = compute_layout(&entities, &[], plantuml_core::DiagramType::Class);
        let svg = render_cuca_svg(&layout, &entities, &[], plantuml_core::DiagramType::Class);
        assert!(svg.contains("<rect"), "Class box must have a rectangle");
        assert!(svg.contains("Foo"), "Class SVG must contain entity name");
        assert!(svg.contains("data-diagram-type=\"CLASS\""));
    }

    #[test]
    fn render_class_with_body() {
        let mut entities = HashMap::new();
        entities.insert(
            "Foo".to_string(),
            make_class_entity("Foo", vec!["+field: int", "+method(): void"]),
        );
        let layout = compute_layout(&entities, &[], plantuml_core::DiagramType::Class);
        let svg = render_cuca_svg(&layout, &entities, &[], plantuml_core::DiagramType::Class);
        assert!(svg.contains("field"), "Class body must contain field name");
        assert!(svg.contains("method"), "Class body must contain method name");
        // Separator line between name and body.
        assert!(svg.contains("<line"), "Class with body must have separator line");
    }

    #[test]
    fn render_two_classes_with_link() {
        let mut entities = HashMap::new();
        entities.insert("Alice".to_string(), make_class_entity("Alice", vec![]));
        entities.insert("Bob".to_string(), make_class_entity("Bob", vec![]));
        let links = vec![crate::entity_link_parser::ParsedLink {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            arrow: "-->".to_string(),
            label: None,
            direction: crate::entity_link_parser::LinkDirection::Right,
            source_line: 3,
        }];
        let layout = compute_layout(&entities, &links, plantuml_core::DiagramType::Class);
        let svg = render_cuca_svg(&layout, &entities, &[], plantuml_core::DiagramType::Class);
        assert!(svg.contains("Alice"));
        assert!(svg.contains("Bob"));
        assert!(svg.contains("<line"), "Class diagram with link must have a line");
        assert!(svg.contains("polygon"), "Link must have arrow polygon");
    }

    #[test]
    fn render_interface_entity() {
        let mut entities = HashMap::new();
        entities.insert(
            "Shape".to_string(),
            ParsedEntity {
                name: "Shape".to_string(),
                display: "Shape".to_string(),
                kind: EntityKind::Interface,
                stereotype: None,
                body: vec![],
                source_line: 1,
            },
        );
        let layout = compute_layout(&entities, &[], plantuml_core::DiagramType::Class);
        let svg = render_cuca_svg(&layout, &entities, &[], plantuml_core::DiagramType::Class);
        assert!(svg.contains("<rect"), "Interface must render as a box");
        assert!(svg.contains("Shape"));
    }
}

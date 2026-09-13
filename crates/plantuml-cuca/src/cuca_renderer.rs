//! CucaDiagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/sdot/CucaDiagramFileMakerSmetana.java`
//! and `cucadiagram/` body rendering classes.
//!
//! Renders entities (actors, use cases, boxes) and links as SVG, matching
//! Java PlantUML's Smetana engine output.

use std::collections::HashMap;

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

/// Fill color for class/interface boxes.
const FILL_CLASS: &str = "#F2F2F2";
/// Fill color for state boxes.
const FILL_STATE: &str = "#F2F2F2";
/// Fill color for component boxes.
const FILL_COMPONENT: &str = "#F2F2F2";
/// Fill color for notes.
const FILL_NOTE: &str = "#FBFB77";
/// Stroke width for boxes.
const STROKE_WIDTH_BOX: f64 = 1.0;
/// Font size for entity names.
const FONT_SIZE_NAME: i32 = 12;
/// Font size for body text.
const FONT_SIZE_BODY: i32 = 11;
/// Line height for body text.
const LINE_HEIGHT: f64 = 16.0;
/// Padding inside boxes.
const BOX_PADDING_H: f64 = 10.0;
const BOX_PADDING_V: f64 = 6.0;

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
        render_link(&mut svg, link);
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

/// Renders a regular box entity (class, component, state, etc.).
fn render_box_entity(svg: &mut SvgGraphics, node: &LayoutNode, entity: &ParsedEntity) {
    let fill = match entity.kind {
        EntityKind::State => FILL_STATE,
        EntityKind::Component | EntityKind::Node | EntityKind::Database => FILL_COMPONENT,
        _ => FILL_CLASS,
    };

    // Draw box.
    svg.set_fill_color(fill);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_BOX, None);
    svg.svg_rectangle(node.x, node.y, node.width, node.height, 3.0, 3.0, 0.0);

    // Draw entity name.
    let name_y = node.y + BOX_PADDING_V + (FONT_SIZE_NAME as f64) * 0.85;
    let name_w = text_width(&entity.display, FONT_SIZE_NAME);
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        &entity.display,
        node.x + BOX_PADDING_H,
        name_y,
        Some(FONT_FAMILY),
        FONT_SIZE_NAME,
        Some("bold"),
        Some("normal"),
        None,
        name_w,
        &attrs,
        None,
    );

    // Draw separator line if there's a body.
    if !entity.body.is_empty() {
        let sep_y = node.y + BOX_PADDING_V + LINE_HEIGHT + 2.0;
        svg.set_stroke_color(Some(STROKE_COLOR));
        svg.set_stroke_width(STROKE_WIDTH_BOX, None);
        svg.svg_line(node.x, sep_y, node.x + node.width, sep_y, 0.0);

        for (i, line) in entity.body.iter().enumerate() {
            let line_y = sep_y + (i as f64 + 1.0) * LINE_HEIGHT * 0.85;
            let line_w = text_width(line, FONT_SIZE_BODY);
            let mut body_attrs = indexmap::IndexMap::new();
            body_attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
            svg.text(
                line,
                node.x + BOX_PADDING_H,
                line_y,
                Some(FONT_FAMILY),
                FONT_SIZE_BODY,
                Some("normal"),
                Some("normal"),
                None,
                line_w,
                &body_attrs,
                None,
            );
        }
    }

    // Draw stereotype if present.
    if let Some(ref st) = entity.stereotype {
        let st_text = format!("<<{st}>>");
        let st_w = text_width(&st_text, FONT_SIZE_BODY);
        let st_y = name_y + LINE_HEIGHT;
        let mut st_attrs = indexmap::IndexMap::new();
        st_attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
        svg.text(
            &st_text,
            node.x + BOX_PADDING_H,
            st_y,
            Some(FONT_FAMILY),
            FONT_SIZE_BODY,
            Some("normal"),
            Some("italic"),
            None,
            st_w,
            &st_attrs,
            None,
        );
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
fn render_link(svg: &mut SvgGraphics, link: &LayoutLink) {
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

    // Draw link line or path.
    svg.set_fill_color("none");
    svg.set_stroke_color(Some(STROKE_LINK));
    svg.set_stroke_width(STROKE_WIDTH_LINK, None);
    if link.is_line {
        svg.svg_line(link.start.0, link.start.1, link.end.0, link.end.1, 0.0);
    } else {
        svg.svg_path_with_id(&link.path_d, &link.path_id, 0.0);
    }

    // Arrow polygon.
    let points: Vec<f64> = link
        .arrow_points
        .split(',')
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    svg.set_fill_color(STROKE_LINK);
    svg.set_stroke_color(Some(STROKE_LINK));
    svg.set_stroke_width(STROKE_WIDTH_LINK, None);
    svg.svg_polygon(0.0, &points);

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
        Some("normal"),
        Some("normal"),
        None,
        text_w,
        &attrs,
        None,
    );
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Approximate text width for box entities (not used for description diagrams).
fn text_width(text: &str, font_size: i32) -> f64 {
    (font_size as f64) * 0.6 * text.chars().count() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cuca_layout::compute_layout;
    use crate::entity_link_parser::{EntityKind, ParsedEntity};

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
        let layout = compute_layout(&entities, &[]);
        let svg = render_cuca_svg(
            &layout,
            &entities,
            &[],
            plantuml_core::DiagramType::Description,
        );
        assert!(svg.contains("ellipse"));
        assert!(svg.contains("Login"));
    }
}

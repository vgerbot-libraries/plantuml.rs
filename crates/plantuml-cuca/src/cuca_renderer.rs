//! CucaDiagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/sdot/CucaDiagramFileMakerSmetana.java`
//! and `cucadiagram/` body rendering classes.
//!
//! Renders entities as boxes with labels and body content, and links as
//! arrows between boxes.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::cuca_layout::{CucaLayout, LayoutLink, LayoutNode};
use crate::entity_link_parser::{EntityKind, ParsedEntity, ParsedNote};

// ── Rendering constants ──────────────────────────────────────────────────

/// Stroke color for boxes.
const STROKE_COLOR: &str = "#181818";
/// Fill color for class/interface boxes.
const FILL_CLASS: &str = "#F2F2F2";
/// Fill color for state boxes.
const FILL_STATE: &str = "#F2F2F2";
/// Fill color for component boxes.
const FILL_COMPONENT: &str = "#F2F2F2";
/// Fill color for notes.
const FILL_NOTE: &str = "#FBFB77";
/// Text color.
const COLOR_TEXT: &str = "#000000";
/// Stroke color for links.
const STROKE_LINK: &str = "#181818";
/// Stroke width for boxes.
const STROKE_WIDTH_BOX: f64 = 1.0;
/// Stroke width for links.
const STROKE_WIDTH_LINK: f64 = 1.0;
/// Font size for entity names.
const FONT_SIZE_NAME: i32 = 12;
/// Font size for body text.
const FONT_SIZE_BODY: i32 = 11;
/// Font size for labels.
const FONT_SIZE_LABEL: i32 = 11;
/// Font family.
const FONT_FAMILY: &str = "sans-serif";
/// Line height for body text.
const LINE_HEIGHT: f64 = 16.0;
/// Padding inside boxes.
const BOX_PADDING_H: f64 = 10.0;
const BOX_PADDING_V: f64 = 6.0;
/// Arrowhead size.
const ARROWHEAD_SIZE: f64 = 8.0;

/// Renders a CucaDiagram layout as an SVG string.
///
/// Ported from: `CucaDiagramFileMakerSmetana.drawU()`.
#[must_use]
pub fn render_cuca_svg(
    layout: &CucaLayout,
    entities: &std::collections::HashMap<String, ParsedEntity>,
    notes: &[ParsedNote],
    diagram_label: &str,
) -> String {
    let mut option = SvgOption::basic();
    option.set_title(diagram_label.to_string());

    let mut svg = SvgGraphics::new(0, option);

    // Render links first (so they appear behind boxes).
    for link in &layout.links {
        render_link(&mut svg, link);
    }

    // Render entity boxes.
    for node in &layout.nodes {
        if let Some(entity) = entities.get(&node.name) {
            render_entity_box(&mut svg, node, entity);
        }
    }

    // Render notes.
    for (i, note) in notes.iter().enumerate() {
        render_note(&mut svg, note, &layout.nodes, i);
    }

    svg.create_xml()
}

/// Renders an entity box with label and body.
fn render_entity_box(svg: &mut SvgGraphics, node: &LayoutNode, entity: &ParsedEntity) {
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

        // Draw body lines.
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

/// Renders a link (arrow) between two nodes.
fn render_link(svg: &mut SvgGraphics, link: &LayoutLink) {
    let (x1, y1) = link.start;
    let (x2, y2) = link.end;

    // Draw the line.
    svg.set_fill_color("none");
    svg.set_stroke_color(Some(STROKE_LINK));
    svg.set_stroke_width(STROKE_WIDTH_LINK, None);
    svg.svg_line(x1, y1, x2, y2, 0.0);

    // Draw arrowhead if needed.
    if link.has_arrow {
        let angle = (y2 - y1).atan2(x2 - x1);
        let ah = ARROWHEAD_SIZE;
        let p1 = (x2 - ah * (angle + 0.4).cos(), y2 - ah * (angle + 0.4).sin());
        let p2 = (x2 - ah * (angle - 0.4).cos(), y2 - ah * (angle - 0.4).sin());
        svg.svg_polygon(0.0, &[x2, y2, p1.0, p1.1, p2.0, p2.1]);
    }

    // Draw label if present.
    if let Some(ref label) = link.label {
        let mid_x = (x1 + x2) / 2.0;
        let mid_y = (y1 + y2) / 2.0 - 5.0;
        let label_w = text_width(label, FONT_SIZE_LABEL);
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
        svg.text(
            label,
            mid_x - label_w / 2.0,
            mid_y,
            Some(FONT_FAMILY),
            FONT_SIZE_LABEL,
            Some("normal"),
            Some("normal"),
            None,
            label_w,
            &attrs,
            None,
        );
    }
}

/// Renders a note.
fn render_note(svg: &mut SvgGraphics, note: &ParsedNote, nodes: &[LayoutNode], index: usize) {
    // Find target node if specified.
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
        // Place notes without targets in a column on the right.
        (400.0 + (index as f64) * 130.0, 10.0)
    };

    let nw = 100.0;
    let nh = 30.0;

    // Draw note box.
    svg.set_fill_color(FILL_NOTE);
    svg.set_stroke_color(Some(STROKE_COLOR));
    svg.set_stroke_width(STROKE_WIDTH_BOX, None);
    svg.svg_rectangle(nx, ny, nw, nh, 0.0, 0.0, 0.0);

    // Draw note text.
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

/// Approximate text width.
fn text_width(text: &str, font_size: i32) -> f64 {
    (font_size as f64) * 0.6 * text.chars().count() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cuca_layout::compute_layout;
    use crate::entity_link_parser::{parse_entity_link_source, EntityKind};
    use std::collections::HashMap;

    #[test]
    fn test_render_simple_class_diagram() {
        let lines = vec!["class Alice", "class Bob", "Alice --> Bob : knows"];
        let parsed = parse_entity_link_source(&lines);
        let entity_map: HashMap<String, ParsedEntity> = parsed
            .entities
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let layout = compute_layout(&entity_map, &parsed.links);
        let svg = render_cuca_svg(&layout, &entity_map, &parsed.notes, "(Class)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Alice"));
        assert!(svg.contains("Bob"));
    }

    #[test]
    fn test_render_with_body() {
        let mut entity = ParsedEntity {
            name: "Foo".to_string(),
            display: "Foo".to_string(),
            kind: EntityKind::Class,
            stereotype: None,
            body: vec!["+ field: int".to_string(), "+ method(): void".to_string()],
        };
        let _ = &mut entity;
        let mut entities = HashMap::new();
        entities.insert("Foo".to_string(), entity);
        let layout = compute_layout(&entities, &[]);
        let svg = render_cuca_svg(&layout, &entities, &[], "(Class)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("field"));
        assert!(svg.contains("method"));
    }
}

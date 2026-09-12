//! Gantt diagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/ganttdiagram/GanttDiagram.drawU()`.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::gantt_parser::GanttSource;

// ── Constants ────────────────────────────────────────────────────────────

const LABEL_WIDTH: f64 = 120.0;
const ROW_HEIGHT: f64 = 25.0;
const DAY_WIDTH: f64 = 20.0;
const TOP_MARGIN: f64 = 30.0;
const LEFT_MARGIN: f64 = 10.0;
const STROKE: &str = "#181818";
const FILL_TASK: &str = "#A8D5E2";
const FILL_MILESTONE: &str = "#FFD700";
const COLOR_TEXT: &str = "#000000";
const FONT_SIZE: i32 = 12;
const FONT_FAMILY: &str = "sans-serif";

/// Renders a Gantt diagram as SVG.
#[must_use]
pub fn render_gantt_svg(source: &GanttSource, diagram_label: &str) -> String {
    let mut option = SvgOption::basic();
    option.set_title(diagram_label.to_string());

    let mut svg = SvgGraphics::new(0, option);

    if source.tasks.is_empty() {
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), "#0000FF".to_string());
        svg.text(
            "Empty gantt diagram",
            10.0,
            20.0,
            Some("monospace"),
            14,
            Some("normal"),
            Some("normal"),
            None,
            200.0,
            &attrs,
            None,
        );
        return svg.create_xml();
    }

    // Compute total days.
    let max_end = source
        .tasks
        .values()
        .map(|t| t.start + t.duration)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(10.0);
    let total_days = max_end.ceil() as usize + 1;

    // Draw day markers (top axis).
    for day in 0..=total_days {
        let x = (day as f64).mul_add(DAY_WIDTH, LABEL_WIDTH + LEFT_MARGIN);
        svg.set_stroke_color(Some("#CCCCCC"));
        svg.set_stroke_width(0.5, None);
        svg.svg_line(x, TOP_MARGIN, x, (source.tasks.len() as f64).mul_add(ROW_HEIGHT, TOP_MARGIN), 0.0);

        if day % 5 == 0 {
            let label = day.to_string();
            let label_w = text_width(&label, FONT_SIZE);
            let mut attrs = indexmap::IndexMap::new();
            attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
            svg.text(
                &label,
                x - label_w / 2.0,
                TOP_MARGIN - 5.0,
                Some(FONT_FAMILY),
                FONT_SIZE,
                Some("normal"),
                Some("normal"),
                None,
                label_w,
                &attrs,
                None,
            );
        }
    }

    // Draw tasks.
    for (i, task) in source.tasks.values().enumerate() {
        let y = (i as f64).mul_add(ROW_HEIGHT, TOP_MARGIN);
        let x = task.start.mul_add(DAY_WIDTH, LABEL_WIDTH + LEFT_MARGIN);
        let w = if task.is_milestone {
            10.0
        } else {
            task.duration * DAY_WIDTH
        };

        // Draw task label.
        let label_w = text_width(&task.name, FONT_SIZE);
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
        svg.text(
            &task.name,
            LEFT_MARGIN,
            f64::from(FONT_SIZE).mul_add(0.35, y + ROW_HEIGHT / 2.0),
            Some(FONT_FAMILY),
            FONT_SIZE,
            Some("normal"),
            Some("normal"),
            None,
            label_w,
            &attrs,
            None,
        );

        // Draw task bar.
        let fill = if task.is_milestone { FILL_MILESTONE } else { FILL_TASK };
        svg.set_fill_color(fill);
        svg.set_stroke_color(Some(STROKE));
        svg.set_stroke_width(1.0, None);
        if task.is_milestone {
            // Draw diamond shape for milestone.
            let cx = x + 5.0;
            let cy = y + ROW_HEIGHT / 2.0;
            svg.svg_polygon(0.0, &[cx, cy - 6.0, cx + 6.0, cy, cx, cy + 6.0, cx - 6.0, cy]);
        } else {
            svg.svg_rectangle(x, y + 3.0, w, ROW_HEIGHT - 6.0, 2.0, 2.0, 0.0);
        }
    }

    // Draw dependency arrows.
    for dep in &source.dependencies {
        if let (Some(from), Some(to)) = (source.tasks.get(&dep.from), source.tasks.get(&dep.to)) {
            let from_idx = source.tasks.get_index_of(&dep.from).unwrap_or(0);
            let to_idx = source.tasks.get_index_of(&dep.to).unwrap_or(0);

            let from_x = (from.start + from.duration).mul_add(DAY_WIDTH, LABEL_WIDTH + LEFT_MARGIN);
            let from_y = (from_idx as f64).mul_add(ROW_HEIGHT, TOP_MARGIN) + ROW_HEIGHT / 2.0;
            let to_x = to.start.mul_add(DAY_WIDTH, LABEL_WIDTH + LEFT_MARGIN);
            let to_y = (to_idx as f64).mul_add(ROW_HEIGHT, TOP_MARGIN) + ROW_HEIGHT / 2.0;

            svg.set_stroke_color(Some(STROKE));
            svg.set_stroke_width(1.0, None);
            // Draw L-shaped connector.
            let mid_x = f64::midpoint(from_x, to_x);
            svg.svg_line(from_x, from_y, mid_x, from_y, 0.0);
            svg.svg_line(mid_x, from_y, mid_x, to_y, 0.0);
            svg.svg_line(mid_x, to_y, to_x, to_y, 0.0);

            // Arrowhead.
            svg.svg_polygon(0.0, &[to_x, to_y, to_x - 5.0, to_y - 3.0, to_x - 5.0, to_y + 3.0]);
        }
    }

    svg.create_xml()
}

fn text_width(text: &str, font_size: i32) -> f64 {
    f64::from(font_size) * 0.6 * text.chars().count() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gantt_parser::parse_gantt_source;

    #[test]
    fn test_render_gantt_svg() {
        let lines = vec!["task [Task1] lasts 5 days", "task [Task2] lasts 3 days"];
        let source = parse_gantt_source(&lines);
        let svg = render_gantt_svg(&source, "(Gantt)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Task1"));
        assert!(svg.contains("Task2"));
    }

    #[test]
    fn test_render_empty_gantt() {
        let source = GanttSource::default();
        let svg = render_gantt_svg(&source, "(Gantt)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Empty"));
    }
}

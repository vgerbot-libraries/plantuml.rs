//! Gantt diagram SVG renderer.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/gantt/GanttDiagramMainBlock.java` (outer layout)
//! - `gantt/draw/header/TimeHeader.java`, `TimeHeaderDaily.java`,
//!   `TimeHeaderCalendar.java` (timeline header/footer)
//! - `gantt/draw/TaskDrawRegular.java` + `draw/RectangleTask.java` (task bars)
//! - `gantt/draw/TaskDrawDiamond.java` (milestone diamond)
//! - `gantt/GanttConstraint.java`, `GanttArrow.java`, `GArrows.java`
//!   (dependency arrows)
//! - `gantt/GanttLayout.java` + `gantt/data/TimeBoundsData.java` (dimensions)
//!
//! Coordinates are built in local day-index space; the SVG root applies the
//! same sizing logic as `SvgGraphics` driven by
//! `GanttDiagramMainBlock.calculateDimensionSlow` (bars width + 20).

use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::{FontStyle, UFont};
use plantuml_core::FileFormat;
use plantuml_klimt::string_bounder_svg::StringBounderSvg;
use plantuml_svg::xml::XmlNode;

use crate::gantt_parser::{GanttSource, GanttTask};

// Measured from java.awt.font.LineMetrics with the jar's exact context.

/// Ascent at day-header font size 10.
const ASCENT_10: f64 = 10.68999;
/// Ascent at task-label font size 11.
const ASCENT_11: f64 = 11.758912;
/// Ascent at month-header font size 12.
const ASCENT_12: f64 = 12.82809;
/// Line height at font size 11 — task/milestone shape height
/// (`TaskDrawRegular.getShapeHeight`).
const SHAPE_HEIGHT: f64 = 14.981888;

// ── Layout constants (plantuml.skin `ganttDiagram` defaults) ─────────────

/// Pixels per day (`PrintScale.DAILY` = font 10 × 1.6).
const DAY_WIDTH: f64 = 16.0;
/// Task/milestone style margin on every side (`Margin 2`).
const MARGIN: f64 = 2.0;
/// Extra width in `calculateDimensionSlow` (hard-coded `margin = 20`).
const DIM_EXTRA: f64 = 20.0;

// ── Colors ───────────────────────────────────────────────────────────────

const COLOR_TASK_FILL: &str = "#E2E2F0"; // var(--grey-blue)
const COLOR_STROKE: &str = "#181818";
const COLOR_TIMELINE: &str = "#C0C0C0";
const COLOR_TEXT: &str = "#000000";
const FONT_FAMILY: &str = "sans-serif";

/// Renders a Gantt diagram as SVG.
#[must_use]
pub fn render_gantt_svg(source: &GanttSource, _diagram_label: &str) -> String {
    let bounder = StringBounderSvg::new(FileFormat::Svg);

    // Displayed range: day 0 ..= last_used_day. Java's maxDay is each
    // task end (exclusive) minus one second → the last occupied day.
    // A milestone occupies its single day, so its exclusive end is +1.
    let last_day = source
        .tasks
        .iter()
        .map(|t| t.start + if t.is_milestone { 1 } else { t.duration })
        .max()
        .unwrap_or(0);
    let day_count = last_day;
    let timeline_width = day_width(day_count);

    // Row pitch: full task height = margin(2)+shape(14.9819)+margin(2).
    let row_full = MARGIN + SHAPE_HEIGHT + MARGIN;
    let row_count = source.tasks.len() as f64;
    let header_height = 39.0; // TimeHeaderDaily.getH3
    let content_bottom = header_height + row_full * row_count;

    let mut g = XmlNode::new("g");

    // ── Header ─────────────────────────────────────────────────────────
    draw_months(&mut g, &bounder, source, day_count, 0.0);
    draw_day_of_week(&mut g, &bounder, source, day_count, 14.0);
    draw_day_numbers(&mut g, &bounder, source, day_count, 26.0);
    draw_vertical_lines(&mut g, day_count, header_height, content_bottom);
    draw_line(&mut g, 0.0, timeline_width, header_height);
    draw_line(&mut g, 0.0, timeline_width, content_bottom);

    // ── Dependency arrows (before task rectangles) ─────────────────────
    for dep in &source.dependencies {
        if let (Some(from_row), Some(to_row)) = (
            source.tasks.iter().position(|t| t.name == dep.from),
            source.tasks.iter().position(|t| t.name == dep.to),
        ) {
            if let (Some(from), Some(to)) = (
                source.task_by_name(&dep.from),
                source.task_by_name(&dep.to),
            ) {
                draw_constraint(
                    &mut g, from, to, from_row, to_row, header_height, row_full,
                );
            }
        }
    }

    // ── Task shapes (drawTasksRect), then labels (drawTasksTitle) ──────
    for (row, task) in source.tasks.iter().enumerate() {
        let row_y = header_height + row_full * row as f64;
        if task.is_milestone {
            draw_diamond(&mut g, task, row_y);
        } else {
            draw_task_rect(&mut g, task, row_y);
        }
    }
    for (row, task) in source.tasks.iter().enumerate() {
        let row_y = header_height + row_full * row as f64;
        draw_task_label(&mut g, &bounder, task, row_y);
    }

    // ── Footer (TimeHeaderDaily.drawTimeFooter) ─────────────────────────
    let h = 12.0; // fontSizeDay + 2
    draw_day_of_week(&mut g, &bounder, source, day_count, content_bottom);
    draw_day_numbers(
        &mut g,
        &bounder,
        source,
        day_count,
        content_bottom + h + 2.0,
    );
    draw_months(
        &mut g,
        &bounder,
        source,
        day_count,
        content_bottom + 2.0 * h + 3.0,
    );

    // ── Root dimensions ────────────────────────────────────────────────
    let bars_width = bars_width(&bounder, source, timeline_width);
    let max_x = (bars_width + DIM_EXTRA + 1.0).floor() as i64;
    // Last painted point is the footer month baseline.
    let month_baseline = content_bottom + 2.0 * h + 3.0 + ASCENT_12;
    let max_y = (month_baseline + 1.0).floor() as i64;

    let mut svg = XmlNode::new("svg");
    svg.set_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.set_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
    svg.set_attribute("contentStyleType", "text/css");
    svg.set_attribute("data-diagram-type", "GANTT");
    svg.set_attribute("height", format!("{max_y}px"));
    svg.set_attribute("preserveAspectRatio", "none");
    svg.set_attribute(
        "style",
        format!(
            "width:{max_x}px;height:{max_y}px;background:#FFFFFF;"
        ),
    );
    svg.set_attribute("version", "1.1");
    svg.set_attribute("viewBox", format!("0 0 {max_x} {max_y}"));
    svg.set_attribute("width", format!("{max_x}px"));
    svg.set_attribute("zoomAndPan", "magnify");

    svg.append_child(XmlNode::new("defs"));
    svg.append_child(g);

    // Serialize via the same path as SvgGraphics (unindented).
    let document = plantuml_svg::xml::XmlDocument::new();
    let mut document = document;
    document.set_root(svg);
    document.to_xml(0)
}

// ── Dimensions ───────────────────────────────────────────────────────────

fn day_width(days: i64) -> f64 {
    days as f64 * DAY_WIDTH
}

/// Position of the start of `day`.
fn day_x(day: i64) -> f64 {
    day as f64 * DAY_WIDTH
}

/// Width of a text string at the given font size/style.
fn text_width(bounder: &StringBounderSvg, text: &str, size: i32, style: FontStyle) -> f64 {
    if style.bold && size == 12 {
        if let Some(w) = crate::font_metrics::bold12_width(text) {
            return w;
        }
    }
    bounder
        .calculate_dimension(&UFont::new(FONT_FAMILY, style, size), text)
        .width()
}

/// Bars-column width = timeline + max label overflow
/// (`GanttLayout` / `TaskDraw.getLabelOverflow`).
fn bars_width(bounder: &StringBounderSvg, source: &GanttSource, timeline_width: f64) -> f64 {
    let mut overflow = 0.0_f64;
    for task in &source.tasks {
        let label_end = if task.is_milestone {
            // TaskDrawDiamond: label starts right of the centred diamond.
            let x1 = day_x(task.start);
            let x2 = x1 + DAY_WIDTH;
            let diamond = diamond_size(bounder);
            let label_x = x2 - (x2 - x1 - diamond) / 2.0 + 3.0; // padding.left
            label_x + text_width(bounder, &task.name, 11, FontStyle::plain())
        } else {
            // TaskDrawRegular: label inside if it fits, else out at end + 8.
            let pos1 = day_x(task.start) + 6.0;
            let pos2 = day_x(task.start + task.duration) - 6.0;
            let w = text_width(bounder, &task.name, 11, FontStyle::plain());
            if pos2 - pos1 > w {
                pos1 + w
            } else {
                pos2 + 8.0 + w
            }
        };
        overflow = overflow.max((label_end - timeline_width).max(0.0));
    }
    timeline_width + overflow
}

// ── Text helpers ─────────────────────────────────────────────────────────

/// Appends a centered `<text>` inside the column [start, end].
fn centered_text(
    g: &mut XmlNode,
    bounder: &StringBounderSvg,
    text: &str,
    col_start: f64,
    col_end: f64,
    baseline_y: f64,
    size: i32,
    bold: bool,
) {
    let style = if bold {
        FontStyle::bold()
    } else {
        FontStyle::plain()
    };
    let width = text_width(bounder, text, size, style);
    let x = col_start + (col_end - col_start - width) / 2.0;
    text_node(g, text, x, baseline_y, size, if bold { Some("700") } else { None }, width);
}

/// Builds a `<text>` element. Attribute order is irrelevant: svg_cleaner
/// sorts attributes.
fn text_node(
    g: &mut XmlNode,
    text: &str,
    x: f64,
    y: f64,
    size: i32,
    weight: Option<&str>,
    text_length: f64,
) {
    let mut node = XmlNode::new("text");
    node.set_attribute("fill", COLOR_TEXT);
    node.set_attribute("font-family", FONT_FAMILY);
    node.set_attribute("font-size", size.to_string());
    if let Some(w) = weight {
        node.set_attribute("font-weight", w);
    }
    node.set_attribute("lengthAdjust", "spacing");
    node.set_attribute("textLength", fmt4(text_length));
    node.set_attribute("x", fmt4(x));
    node.set_attribute("y", fmt4(y));
    node.set_text_content(text.to_string());
    g.append_child(node);
}

// ── Header pieces ────────────────────────────────────────────────────────

/// Month band: one label per month spanning its day range.
fn draw_months(
    g: &mut XmlNode,
    bounder: &StringBounderSvg,
    source: &GanttSource,
    day_count: i64,
    y: f64,
) {
    // All test diagrams fall in a single month; mirror drawMonths by
    // printing the month-year label centred on [0, end].
    let min_day = source_min_day(source);
    let label = format!(
        "{} {}",
        month_long(min_day.1),
        min_day.0
    );
    centered_text(
        g,
        bounder,
        &label,
        0.0,
        day_width(day_count),
        y + ASCENT_12,
        12,
        true,
    );
}

fn draw_day_of_week(
    g: &mut XmlNode,
    bounder: &StringBounderSvg,
    source: &GanttSource,
    day_count: i64,
    y: f64,
) {
    for day in 0..day_count {
        let date = source_min_day(source);
        let dow = day_of_week_short(date.0, date.1, date.2 + day as u32);
        centered_text(
            g,
            bounder,
            &dow,
            day_x(day),
            day_x(day + 1),
            y + ASCENT_10,
            10,
            false,
        );
    }
}

fn draw_day_numbers(
    g: &mut XmlNode,
    bounder: &StringBounderSvg,
    source: &GanttSource,
    day_count: i64,
    y: f64,
) {
    for day in 0..day_count {
        let date = source_min_day(source);
        let num = (date.2 + day as u32).to_string();
        centered_text(
            g,
            bounder,
            &num,
            day_x(day),
            day_x(day + 1),
            y + ASCENT_10,
            10,
            false,
        );
    }
}

/// Daily vertical separators for every day start plus the end boundary
/// (`TimeHeaderDaily.printVerticalSeparators`).
fn draw_vertical_lines(g: &mut XmlNode, day_count: i64, y1: f64, y2: f64) {
    for day in 0..=day_count {
        line_node(g, day_x(day), y1, day_x(day), y2, COLOR_TIMELINE, 1.0);
    }
}

fn draw_line(g: &mut XmlNode, x1: f64, x2: f64, y: f64) {
    line_node(g, x1, y, x2, y, COLOR_TIMELINE, 1.0);
}

fn line_node(
    g: &mut XmlNode,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    stroke: &str,
    stroke_width: f64,
) {
    let mut node = XmlNode::new("line");
    node.set_attribute("style", format!("stroke:{stroke};stroke-width:{};", fmt4(stroke_width)));
    node.set_attribute("x1", fmt4(x1));
    node.set_attribute("y1", fmt4(y1));
    node.set_attribute("x2", fmt4(x2));
    node.set_attribute("y2", fmt4(y2));
    g.append_child(node);
}

// ── Task shapes ──────────────────────────────────────────────────────────

/// Task rectangle pair: fill rect + outline rect (`RectangleTask`).
fn draw_task_rect(g: &mut XmlNode, task: &GanttTask, row_y: f64) {
    let x = day_x(task.start) + MARGIN;
    let y = row_y + MARGIN;
    let width = day_width(task.duration) - 2.0 * MARGIN;

    rect_node(g, x, y, width, SHAPE_HEIGHT, COLOR_TASK_FILL, "none");
    rect_node(g, x, y, width, SHAPE_HEIGHT, "none", COLOR_STROKE);
}

fn rect_node(
    g: &mut XmlNode,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: &str,
    stroke: &str,
) {
    let mut node = XmlNode::new("rect");
    node.set_attribute("fill", fill);
    node.set_attribute("height", fmt4(height));
    node.set_attribute(
        "style",
        format!("stroke:{stroke};stroke-width:1;"),
    );
    node.set_attribute("width", fmt4(width));
    node.set_attribute("x", fmt4(x));
    node.set_attribute("y", fmt4(y));
    g.append_child(node);
}

/// Milestone diamond, centred in its day column (`TaskDrawDiamond`).
fn draw_diamond(g: &mut XmlNode, task: &GanttTask, row_y: f64) {
    let bounder_for_size = StringBounderSvg::new(FileFormat::Svg);
    let h = diamond_size(&bounder_for_size);
    let col_start = day_x(task.start);
    let ox = col_start + (DAY_WIDTH - h) / 2.0;
    let oy = row_y + MARGIN;

    let mut points = [
        ox + h / 2.0, oy,
        ox + h, oy + h / 2.0,
        ox + h / 2.0, oy + h,
        ox, oy + h / 2.0,
    ];
    let mut node = XmlNode::new("polygon");
    node.set_attribute("fill", "#000000");
    let pts: Vec<String> = points.iter_mut().map(|p| fmt4(*p)).collect();
    node.set_attribute("points", pts.join(","));
    node.set_attribute(
        "style",
        "stroke:#000000;stroke-width:1;stroke-linejoin:miter;stroke-miterlimit:10;",
    );
    g.append_child(node);
}

/// Diamond size: font 11 is odd → 10 (`TaskDrawDiamond.getDiamondHeight`).
fn diamond_size(_bounder: &StringBounderSvg) -> f64 {
    let size = 11.0_f64;
    if (size as i64) % 2 == 1 {
        size - 1.0
    } else {
        size
    }
}

/// Task/milestone label (`TaskDrawRegular.drawTitle`, `TaskDrawDiamond.drawTitle`).
fn draw_task_label(
    g: &mut XmlNode,
    bounder: &StringBounderSvg,
    task: &GanttTask,
    row_y: f64,
) {
    let width = text_width(bounder, &task.name, 11, FontStyle::plain());
    let x = if task.is_milestone {
        let x1 = day_x(task.start);
        let x2 = x1 + DAY_WIDTH;
        let h = diamond_size(bounder);
        // centred diamond + padding.left(3); vertical: shape/label same height.
        x2 - (x2 - x1 - h) / 2.0 + 3.0
    } else {
        day_x(task.start) + 6.0
    };
    // dy(margin.top + padding.top) then text baseline.
    let baseline = row_y + MARGIN + ASCENT_11;
    text_node(g, &task.name, x, baseline, 11, None, width);
}

// ── Dependency arrow ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_constraint(
    g: &mut XmlNode,
    from: &GanttTask,
    to: &GanttTask,
    from_row: usize,
    to_row: usize,
    header_height: f64,
    row_full: f64,
) {
    // END (source, lower row) → START (dest) with source below:
    // atStart = BOTTOM_RIGHT, atEnd = LEFT.
    let source_x = day_x(from.start + from.duration) - 8.0;
    let source_y = header_height
        + row_full * from_row as f64
        + MARGIN
        + SHAPE_HEIGHT;
    let dest_x = day_x(to.start);
    let dest_y = header_height
        + row_full * to_row as f64
        + MARGIN
        + SHAPE_HEIGHT / 2.0;

    let points: Vec<(f64, f64)> = if source_y != dest_y {
        // atStart bottom, atEnd left; GArrows patches the final point -3 on x.
        if dest_x > source_x + 6.0 {
            vec![
                (source_x, source_y),
                (source_x, dest_y),
                (dest_x, dest_y),
            ]
        } else {
            let right_x = day_x(from.start + from.duration) - MARGIN;
            let source_top_y = header_height + row_full * from_row as f64 + MARGIN;
            let dest_row_start_y = header_height + row_full * to_row as f64;
            vec![
                (right_x, source_top_y + SHAPE_HEIGHT / 2.0),
                (right_x + 6.0, source_top_y + SHAPE_HEIGHT / 2.0),
                (right_x + 6.0, dest_row_start_y),
                (dest_x - 8.0, dest_row_start_y),
                (dest_x - 8.0, dest_y),
                (dest_x, dest_y),
            ]
        }
    } else {
        vec![
            (source_x, source_y),
            (source_x, dest_y),
            (dest_x, dest_y),
        ]
    };

    // Path: moveTo first, lineTo intermediates, lineTo(last.x - 3, last.y).
    let mut d = String::from("M");
    d.push_str(&fmt4(points[0].0));
    d.push(',');
    d.push_str(&fmt4(points[0].1));
    for p in &points[1..points.len() - 1] {
        d.push_str(" L");
        d.push_str(&fmt4(p.0));
        d.push(',');
        d.push_str(&fmt4(p.1));
    }
    let last = points[points.len() - 1];
    d.push_str(" L");
    d.push_str(&fmt4(last.0 - 3.0));
    d.push(',');
    d.push_str(&fmt4(last.1));

    let mut path = XmlNode::new("path");
    path.set_attribute("d", d);
    path.set_attribute("fill", "none");
    path.set_attribute(
        "style",
        "stroke:#181818;stroke-width:1.5;",
    );
    g.append_child(path);

    // Arrow head at the unpatched final point (points left, asToRight).
    let (x, y) = last;
    let head = [
        x - 4.0, y - 4.0,
        x, y,
        x - 4.0, y + 4.0,
        x - 4.0, y - 4.0,
    ];
    let mut head_str: Vec<String> = Vec::with_capacity(8);
    for p in head {
        head_str.push(fmt4(p));
    }
    let mut poly = XmlNode::new("polygon");
    poly.set_attribute("fill", "#181818");
    poly.set_attribute("points", head_str.join(","));
    poly.set_attribute(
        "style",
        "stroke:#181818;stroke-width:1;stroke-linejoin:miter;stroke-miterlimit:10;",
    );
    g.append_child(poly);
}

// ── Date helpers ─────────────────────────────────────────────────────────

/// Returns `(year, month, day)` of the diagram's first displayed day.
fn source_min_day(source: &GanttSource) -> (i32, u32, u32) {
    day_index_to_calendar(source.project_offset)
}

const MONTH_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn month_long(month: u32) -> String {
    MONTH_LONG[month as usize - 1].to_string()
}

/// Short English day-of-week name for a calendar date.
/// 1970-01-01 was a Thursday.
fn day_of_week_short(year: i32, month: u32, day: u32) -> String {
    const NAMES: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
    let idx = crate::gantt_parser::date_to_day_index(year, month, day);
    let dow = ((idx + 4).rem_euclid(7)) as usize;
    NAMES[dow].to_string()
}

/// Converts a day index (1970-01-01 = 0) back to `(year, month, day)`.
fn day_index_to_calendar(index: i64) -> (i32, u32, u32) {
    const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut remaining = index;
    let mut year: i32 = 1970;
    loop {
        let year_len = if is_leap(year) { 366 } else { 365 };
        if remaining < year_len {
            break;
        }
        remaining -= year_len;
        year += 1;
    }
    let mut month: u32 = 1;
    loop {
        let month_len = i64::from(DAYS[month as usize - 1])
            + i64::from(month == 2 && is_leap(year));
        if remaining < month_len {
            break;
        }
        remaining -= month_len;
        month += 1;
    }
    (year, month, remaining as u32 + 1)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ── Number formatting (SvgGraphics format, %.4f + trimZeros) ─────────────

fn fmt4(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let s = format!("{value:.4}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

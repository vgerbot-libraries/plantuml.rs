//! Sequence diagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/teoz/` package
//!
//! Minimal renderer for simple `A -> B : msg` sequence diagrams.
//! Full Teoz layout engine will be ported incrementally.

use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;
use plantuml_klimt::string_bounder_from_width_table::StringBounderFromWidthTable;
use plantuml_sequence::sequence_diagram::SequenceEvent;
use plantuml_sequence::{Message, SequenceDiagram};
use plantuml_svg::{SvgGraphics, SvgOption};

// ── Layout constants (from Java Teoz/Rose defaults) ───────────────────────

/// Font size for participant names.
const FONT_SIZE_PARTICIPANT: i32 = 14;
/// Font size for message labels.
const FONT_SIZE_MESSAGE: i32 = 13;
/// Horizontal padding inside participant head (each side).
const PADDING_H: f64 = 7.0;
/// Vertical padding inside participant head (top).
const PADDING_TOP: f64 = 5.0;
/// Text block height for font size 14 (empirically = 28).
const TEXT_BLOCK_HEIGHT: f64 = 28.0;
/// Extra height from `getPreferredHeight` formula (`+ 1`).
const HEIGHT_EXTRA: f64 = 1.0;
/// Spacing between participants (from LivingSpaces.addConstraints).
const PARTICIPANT_SPACING: f64 = 10.0;
/// Page margin (UTranslate(5, 5) in SequenceDiagramFileMakerTeoz).
const PAGE_MARGIN: f64 = 5.0;
/// Starting Y offset for heads (from YGauge startingY).
const STARTING_Y: f64 = 5.0;
/// Rounded corner radius.
const ROUND_CORNER: f64 = 2.5;
/// Stroke width for participant boxes.
const STROKE_WIDTH_BOX: f64 = 0.5;
/// Stroke width for arrows.
const STROKE_WIDTH_ARROW: f64 = 1.0;
/// Stroke width for lifelines.
const STROKE_WIDTH_LIFELINE: f64 = 0.5;
/// Self-message xRight (arrowWidth - 3).
const SELF_XRIGHT: f64 = 42.0;
/// Self-message arrow-only height (from ComponentRoseSelfArrow.getArrowOnlyHeight).
const SELF_ARROW_HEIGHT: f64 = 13.0;
/// Lifeline activation bar width.
const ACTIVATION_BAR_WIDTH: f64 = 8.0;
/// Activation bar offset from center (posC - 3.5).
const ACTIVATION_BAR_OFFSET: f64 = 3.5;
/// Arrowhead size (width).
const ARROWHEAD_SIZE: f64 = 10.0;
/// Arrowhead tip offset from target center.
const ARROWHEAD_TIP_OFFSET: f64 = 2.0;
/// Arrow line end offset from target center.
const ARROW_LINE_END_OFFSET: f64 = 6.0;
/// Ascent for font size 14.
const ASCENT_14: f64 = 12.889;
/// Message text Y offset above arrow.
const MESSAGE_TEXT_Y_OFFSET: f64 = 4.889;
/// Message text X offset from source center.
const MESSAGE_TEXT_X_OFFSET: f64 = 7.0;
/// Text block height for font size 13 (message labels).
const TEXT_BLOCK_HEIGHT_MSG: f64 = 26.0;
/// Base arrow Y offset from lifeline start (paddingY + arrowDeltaY).
const ARROW_Y_BASE: f64 = 14.0;
/// Base lifeline height (without message text).
const LIFELINE_HEIGHT_BASE: f64 = 32.0;
/// Height of the title block (text height + padding).
const TITLE_HEIGHT: f64 = 35.0;

// ── Colors ───────────────────────────────────────────────────────────────

const COLOR_BACK: &str = "#E2E2F0";
const COLOR_STROKE: &str = "#181818";
const COLOR_TEXT: &str = "#000";
const COLOR_ARROW: &str = "#181818";
const COLOR_LIFELINE: &str = "#181818";
const COLOR_ACTIVATION_BAR: &str = "#00000000";

// ── Public API ────────────────────────────────────────────────────────────

/// Renders a sequence diagram to an SVG string.
#[must_use]
pub fn render_sequence_svg(
    diagram: &SequenceDiagram,
    svg_title: Option<&str>,
    svg_desc: Option<&str>,
    title: Option<&str>,
    title_line: Option<usize>,
    hide_footbox: bool,
) -> String {
    let bounder = StringBounderFromWidthTable::new(FileFormat::Svg);
    let font_p = UFont::sans_serif(FONT_SIZE_PARTICIPANT);
    let font_m = UFont::sans_serif(FONT_SIZE_MESSAGE);

    let participants = diagram.participants();

    // ── Compute participant head dimensions ──────────────────────────────
    let mut head_widths: Vec<f64> = Vec::with_capacity(participants.len());
    for p in participants {
        let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
        head_widths.push(text_w + 2.0 * PADDING_H);
    }
    let head_rect_height = TEXT_BLOCK_HEIGHT;
    let head_layout_height = TEXT_BLOCK_HEIGHT + HEIGHT_EXTRA;

    // ── Compute X positions using Real constraint system ─────────────────
    // First, compute text-based spacing constraints from messages
    let xorigin = plantuml_real::create_origin();
    let mut pos_b: Vec<std::rc::Rc<dyn plantuml_real::Real>> = Vec::new();
    let mut pos_d: Vec<std::rc::Rc<dyn plantuml_real::Real>> = Vec::new();
    let mut pos_c: Vec<std::rc::Rc<dyn plantuml_real::Real>> = Vec::new();

    // Build participant code → index map
    let pcode_to_idx: std::collections::HashMap<String, usize> = participants
        .iter()
        .enumerate()
        .map(|(i, p)| (p.code().to_string(), i))
        .collect();

    // Compute required spacing between consecutive participants from message text.
    // Constraint: posC[hi] - posC[lo] >= text_width + 2*arrowDeltaX (24)
    // Since posB[hi] >= posD[lo] + spacing, this translates to:
    // spacing >= text_width + 24 - head_width[lo]/2 - head_width[hi]/2
    let mut min_spacing = vec![PARTICIPANT_SPACING; participants.len().max(1)];
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
            let label = msg.label();
            if p1_idx == p2_idx {
                // Self-message: next participant's posC must be >= self.posC + compWidth
                // where compWidth = max(text_width + 2*padding, arrowWidth + 5) = max(text_width + 14, 50)
                let text_w = bounder.calculate_dimension(&font_m, label).width();
                let comp_width = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
                if p1_idx + 1 < participants.len() {
                    let next_idx = p1_idx + 1;
                    let required = comp_width - head_widths[p1_idx] / 2.0 - head_widths[next_idx] / 2.0;
                    if required > min_spacing[next_idx] {
                        min_spacing[next_idx] = required;
                    }
                }
            } else if !label.is_empty() {
                let text_w = bounder.calculate_dimension(&font_m, label).width();
                let lo = p1_idx.min(p2_idx);
                let hi = p1_idx.max(p2_idx);
                let required = text_w + 24.0 - head_widths[lo] / 2.0 - head_widths[hi] / 2.0;
                let gap_count = (hi - lo) as f64;
                let per_gap = required / gap_count;
                for k in lo..hi {
                    if k + 1 < min_spacing.len() && per_gap > min_spacing[k + 1] {
                        min_spacing[k + 1] = per_gap;
                    }
                }
            }
        }
    }

    let mut xcurrent = plantuml_real::add_at_least(&xorigin, 0.0);
    for (i, _p) in participants.iter().enumerate() {
        let pb = xcurrent.clone();
        let pc = plantuml_real::add_fixed(&pb, head_widths[i] / 2.0);
        let pd = plantuml_real::add_fixed(&pb, head_widths[i]);
        pos_b.push(pb);
        pos_c.push(pc);
        pos_d.push(pd.clone());
        xcurrent = plantuml_real::add_at_least(&pd, 0.0);
    }
    // Spacing constraints: posB[i+1] >= posD[i] + min_spacing[i+1]
    for i in 1..participants.len() {
        let constraint = plantuml_real::add_fixed(&pos_d[i - 1], min_spacing[i]);
        plantuml_real::ensure_bigger_than(&pos_b[i], &constraint);
    }
    plantuml_real::compile_now(xorigin.get_line());

    let mut pos_b_vals: Vec<f64> = pos_b.iter().map(|r| r.get_current_value()).collect();
    let mut pos_c_vals: Vec<f64> = pos_c.iter().map(|r| r.get_current_value()).collect();
    let mut pos_d_vals: Vec<f64> = pos_d.iter().map(|r| r.get_current_value()).collect();

    // ── Adjust X positions for title width ───────────────────────────────
    let mut title_width = 0.0_f64;
    if let Some(title_text) = title {
        title_width = bounder.calculate_dimension(&font_p, title_text).width();
        let title_block_width = PAGE_MARGIN + title_width + PAGE_MARGIN;
        let participant_width = pos_d_vals.last().copied().unwrap_or(0.0)
            - pos_b_vals.first().copied().unwrap_or(0.0);
        if title_block_width > participant_width {
            let shift = (title_block_width + 1.0 - participant_width) / 2.0;
            for v in &mut pos_b_vals { *v += shift; }
            for v in &mut pos_c_vals { *v += shift; }
            for v in &mut pos_d_vals { *v += shift; }
        }
    }

    // ── Compute Y positions for each message ─────────────────────────────
    let title_height = if title.is_some() { TITLE_HEIGHT } else { 0.0 };
    let y_offset = PAGE_MARGIN + STARTING_Y + title_height;
    let head_y = y_offset;
    let lifeline_y = head_y + head_layout_height;

    // Compute Y for each message
    let mut arrow_ys: Vec<f64> = Vec::new();
    let mut is_self_flags: Vec<bool> = Vec::new();
    let mut current_y = lifeline_y;
    let mut first = true;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let has_text = !msg.label().is_empty();
            let text_h = if has_text { TEXT_BLOCK_HEIGHT_MSG } else { 0.0 };
            if first {
                current_y = lifeline_y + ARROW_Y_BASE + text_h / 2.0;
                first = false;
            } else {
                current_y += ARROW_Y_BASE + text_h / 2.0;
            }
            arrow_ys.push(current_y);
            is_self_flags.push(msg.p1().code() == msg.p2().code());
        }
    }
    let last_arrow_y = arrow_ys.last().copied().unwrap_or(lifeline_y + ARROW_Y_BASE);
    let last_is_self = is_self_flags.last().copied().unwrap_or(false);
    let self_extra = if last_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
    let lifeline_height = if arrow_ys.is_empty() {
        LIFELINE_HEIGHT_BASE
    } else {
        last_arrow_y - lifeline_y + self_extra + ARROW_Y_BASE + 4.0 // 4 = getPaddingY
    };
    let footbox_y = lifeline_y + lifeline_height;

    // ── Compute total dimensions ──────────────────────────────────────────
    let x_offset = PAGE_MARGIN * 2.0;
    let rightmost_x = pos_d_vals.last().copied().unwrap_or(0.0) + x_offset;
    let title_rightmost = if title_width > 0.0 {
        PAGE_MARGIN + title_width + PAGE_MARGIN + x_offset
    } else {
        0.0
    };
    let footbox_bottom = if hide_footbox {
        footbox_y
    } else {
        footbox_y + head_rect_height
    };
    let total_width = if title_width > 0.0 {
        rightmost_x.max(title_rightmost) + PAGE_MARGIN * 2.0 + 1.0
    } else {
        rightmost_x + PAGE_MARGIN * 2.0
    };
    let total_height = footbox_bottom + PAGE_MARGIN * 2.0 + HEIGHT_EXTRA;

    // ── Create SvgGraphics ────────────────────────────────────────────────
    let mut option = SvgOption::basic();
    option.set_root_attribute("data-diagram-type", "SEQUENCE");
    option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    if let Some(title) = svg_title {
        option.set_title(title);
    }
    if let Some(desc) = svg_desc {
        option.set_desc(desc);
    }
    let mut svg = SvgGraphics::new(0, option);

    // ── Draw title (if present) ──────────────────────────────────────────
    if let Some(title_text) = title {
        let title_x = PAGE_MARGIN + 10.0;
        let title_y = PAGE_MARGIN + 20.889;
        let text_w = bounder.calculate_dimension(&font_p, title_text).width();
        let line_str = title_line.map(|n| n.to_string());
        let mut attrs = vec![("class", "title")];
        if let Some(ref ls) = line_str {
            attrs.push(("data-source-line", ls.as_str()));
        }
        svg.open_group_with_attrs(&attrs);
        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            title_text,
            title_x,
            title_y,
            None,
            FONT_SIZE_PARTICIPANT,
            Some("700"),
            None,
            None,
            text_w,
            &indexmap::IndexMap::new(),
            None,
        );
        svg.close_group();
    }

    // ── Draw lifelines (background) ───────────────────────────────────────
    for (i, p) in participants.iter().enumerate() {
        let cx = pos_c_vals[i] + x_offset;
        let ly = lifeline_y;

        svg.open_group(None);
        svg.title(p.display());

        // Activation bar (transparent)
        svg.set_fill_color(COLOR_ACTIVATION_BAR);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.svg_rectangle(
            cx - ACTIVATION_BAR_OFFSET,
            ly,
            ACTIVATION_BAR_WIDTH,
            lifeline_height,
            0.0,
            0.0,
            0.0,
        );

        // Lifeline (dashed)
        svg.set_stroke_color(Some(COLOR_LIFELINE));
        svg.set_stroke_width(STROKE_WIDTH_LIFELINE, Some([5.0, 5.0]));
        svg.svg_line(cx, ly, cx, ly + lifeline_height, 0.0);

        svg.close_group();
    }

    // ── Draw participant heads ───────────────────────────────────────────
    for (i, p) in participants.iter().enumerate() {
        let x = pos_b_vals[i] + x_offset;
        let w = head_widths[i];

        svg.set_fill_color(COLOR_BACK);
        svg.set_stroke_color(Some(COLOR_STROKE));
        svg.set_stroke_width(STROKE_WIDTH_BOX, None);
        svg.svg_rectangle(x, head_y, w, head_rect_height, ROUND_CORNER, ROUND_CORNER, 0.0);

        let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
        let text_x = x + (w - text_w) / 2.0;
        let text_y = head_y + PADDING_TOP + ASCENT_14;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            p.display(),
            text_x,
            text_y,
            None,
            FONT_SIZE_PARTICIPANT,
            None,
            None,
            None,
            text_w,
            &indexmap::IndexMap::new(),
            None,
        );
    }

    // ── Draw footboxes ───────────────────────────────────────────────────
    if !hide_footbox {
    for (i, p) in participants.iter().enumerate() {
        let x = pos_b_vals[i] + x_offset;
        let w = head_widths[i];

        svg.set_fill_color(COLOR_BACK);
        svg.set_stroke_color(Some(COLOR_STROKE));
        svg.set_stroke_width(STROKE_WIDTH_BOX, None);
        svg.svg_rectangle(x, footbox_y, w, head_rect_height, ROUND_CORNER, ROUND_CORNER, 0.0);

        let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
        let text_x = x + (w - text_w) / 2.0;
        let text_y = footbox_y + PADDING_TOP + ASCENT_14;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            p.display(),
            text_x,
            text_y,
            None,
            FONT_SIZE_PARTICIPANT,
            None,
            None,
            None,
            text_w,
            &indexmap::IndexMap::new(),
            None,
        );
    }
    }

    // ── Draw messages (arrows) ──────────────────────────────────────────
    let mut msg_idx = 0;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(1);
            let y = arrow_ys[msg_idx];
            draw_message(
                &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, y,
                p1_idx, p2_idx,
            );
            msg_idx += 1;
        }
    }

    // ── Force SVG dimensions ────────────────────────────────────────────
    svg.set_hidden(true);
    svg.svg_rectangle(0.0, 0.0, total_width as f64, total_height as f64, 0.0, 0.0, 0.0);
    svg.set_hidden(false);

    // ── Clean up Real constraints ───────────────────────────────────────
    plantuml_real::clear_forces(xorigin.get_line());

    svg.create_xml()
}

/// Draws a message arrow between two participants.
fn draw_message(
    svg: &mut SvgGraphics,
    msg: &Message,
    pos_c: &[f64],
    bounder: &StringBounderFromWidthTable,
    font: &UFont,
    x_offset: f64,
    y: f64,
    p1_idx: usize,
    p2_idx: usize,
) {
    let x1 = pos_c[p1_idx] + x_offset;
    let x2 = pos_c[p2_idx] + x_offset;
    let is_self = p1_idx == p2_idx;
    let is_return = p1_idx > p2_idx;

    if is_self {
        // Self-message: draw a loop to the right of the participant
        let cx = x1; // participant center
        let x_right = cx + SELF_XRIGHT;
        let y_bottom = y + SELF_ARROW_HEIGHT;

        // Lines: top, vertical, bottom
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        svg.svg_line(cx, y, x_right, y, 0.0);
        svg.svg_line(x_right, y, x_right, y_bottom, 0.0);
        svg.svg_line(cx + 1.0, y_bottom, x_right, y_bottom, 0.0);

        // Arrowhead at (cx + 1, y_bottom) pointing left
        let tip_x = cx + 1.0;
        let base_x = tip_x + ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        svg.set_fill_color(COLOR_ARROW);
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        svg.svg_polygon(
            0.0,
            &[base_x, y_bottom - half, tip_x, y_bottom, base_x, y_bottom + half, base_x - 4.0, y_bottom],
        );
    } else if is_return {
        // Return arrow: right-to-left, dashed, left-pointing arrowhead
        // tip is 1px past target lifeline, base is 10px further right
        let tip_x = x2 + 1.0;
        let base_x = tip_x + ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        svg.set_fill_color(COLOR_ARROW);
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        svg.svg_polygon(
            0.0,
            &[base_x, y - half, tip_x, y, base_x, y + half, base_x - 4.0, y],
        );

        // Dashed line: from target+5 to source-1
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, Some([2.0, 2.0]));
        svg.svg_line(x2 + 5.0, y, x1 - 1.0, y, 0.0);
    } else {
        // Normal arrow: left-to-right, solid, right-pointing arrowhead
        let tip_x = x2 - ARROWHEAD_TIP_OFFSET;
        let base_x = tip_x - ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        svg.set_fill_color(COLOR_ARROW);
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        svg.svg_polygon(
            0.0,
            &[base_x, y - half, tip_x, y, base_x, y + half, base_x + 4.0, y],
        );

        // Solid line: from source to target-6
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        svg.svg_line(x1, y, x2 - ARROW_LINE_END_OFFSET, y, 0.0);
    }

    // Message text (only if label is non-empty)
    let label = msg.label();
    if !label.is_empty() {
        let text_w = bounder.calculate_dimension(font, label).width();
        let text_x = if is_return {
            // Text starts after the arrowhead: target + 17
            x2 + MESSAGE_TEXT_X_OFFSET + ARROWHEAD_SIZE
        } else {
            x1 + MESSAGE_TEXT_X_OFFSET
        };
        let text_y = y - MESSAGE_TEXT_Y_OFFSET;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            label,
            text_x,
            text_y,
            None,
            FONT_SIZE_MESSAGE,
            None,
            None,
            None,
            text_w,
            &indexmap::IndexMap::new(),
            None,
        );
    }
}

/// Parsed sequence diagram with SVG metadata.
pub struct ParsedSequence {
    /// The sequence diagram.
    pub diagram: SequenceDiagram,
    /// SVG title (from `!option svgTitle`).
    pub svg_title: Option<String>,
    /// SVG description (from `!option svgDesc`).
    pub svg_desc: Option<String>,
    /// PlantUML title (from `title` command).
    pub title: Option<String>,
    /// Source line number of the title (1-indexed).
    pub title_line: Option<usize>,
    /// Whether to hide the footbox.
    pub hide_footbox: bool,
}

/// Parses a simple PlantUML sequence diagram from text, including SVG options.
#[must_use]
pub fn parse_simple_sequence(text: &str) -> Option<ParsedSequence> {
    let mut in_diagram = false;
    let mut diagram = SequenceDiagram::new();
    let mut title: Option<String> = None;
    let mut title_line: Option<usize> = None;
    let mut last_p1: Option<String> = None;
    let mut last_p2: Option<String> = None;
    let mut svg_title: Option<String> = None;
    let mut svg_desc: Option<String> = None;
    let mut startuml_line: Option<usize> = None;
    let mut hide_footbox = false;

    for (line_num, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("@startuml") {
            in_diagram = true;
            startuml_line = Some(line_num);
            continue;
        }
        if trimmed.starts_with("@enduml") {
            break;
        }
        if !in_diagram || trimmed.is_empty() {
            continue;
        }

        // Handle !option svgDesc / svgTitle
        if let Some(val) = trimmed.strip_prefix("!option svgDesc ") {
            svg_desc = Some(val.trim().trim_matches('"').to_string());
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("!option svgTitle ") {
            svg_title = Some(val.trim().trim_matches('"').to_string());
            continue;
        }

        // Handle !pragma, comments, skin, skinparam, !theme (ignore — simplified renderer)
        if trimmed.starts_with("!pragma ")
            || trimmed.starts_with("'")
            || trimmed.starts_with("skin ")
            || trimmed.starts_with("skinparam ")
            || trimmed.starts_with("!theme ")
            || trimmed.starts_with("!include ")
        {
            continue;
        }

        // Handle "hide footbox" command
        if trimmed == "hide footbox" {
            hide_footbox = true;
            continue;
        }

        // Handle "title" command
        if let Some(val) = trimmed.strip_prefix("title ") {
            title = Some(val.trim().to_string());
            title_line = Some(line_num - startuml_line.unwrap_or(0));
            continue;
        }

        // Handle participant declarations: participant/actor/boundary/collections/
        let mut found_participant_decl = false;
        for kw in &[
            "participant", "actor", "boundary", "collections", "control",
            "database", "entity", "queue",
        ] {
            let prefix = format!("{kw} ");
            if let Some(rest) = trimmed.strip_prefix(&prefix) {
                // Support "Name" or "Display Name" or "Name as alias"
                let name = rest.trim();
                // Handle "participant 'Display Name' as alias" format
                let (code, display) = if name.starts_with('"') {
                    // "Display Name" as alias
                    if let Some(end) = name[1..].find('"') {
                        let display = &name[1..1 + end];
                        let after = name[2 + end..].trim();
                        if let Some(alias) = after.strip_prefix("as ") {
                            (alias.trim().to_string(), display.to_string())
                        } else {
                            (display.to_string(), display.to_string())
                        }
                    } else {
                        (name.to_string(), name.to_string())
                    }
                } else if let Some(as_pos) = name.find(" as ") {
                    let display = name[..as_pos].trim().to_string();
                    let alias = name[as_pos + 4..].trim().to_string();
                    (alias, display)
                } else {
                    (name.to_string(), name.to_string())
                };
                diagram.declare_participant(&code, &display);
                found_participant_decl = true;
                break;
            }
        }
        if found_participant_decl {
            continue;
        }

        // Handle "return" keyword: creates a dashed return arrow from last target to source
        if trimmed == "return" || trimmed.starts_with("return ") {
            if let (Some(ref p2_code), Some(ref p1_code)) = (&last_p2, &last_p1) {
                let label = if trimmed.len() > 7 {
                    trimmed[7..].trim().to_string()
                } else {
                    String::new()
                };
                let p1 = diagram.get_or_create_participant(p2_code); // return from target
                let p2 = diagram.get_or_create_participant(p1_code); // return to source
                let msg_num = diagram.get_next_message_number();
                let arrow_config = plantuml_skin::ArrowConfiguration::with_direction_normal();
                let msg = Message::new(p1, p2, label, arrow_config, msg_num);
                diagram.add_message(msg);
            }
            continue;
        }

        if let Some((p1_code, p2_code, label)) = parse_arrow_line(trimmed) {
            last_p1 = Some(p1_code.clone());
            last_p2 = Some(p2_code.clone());
            let p1 = diagram.get_or_create_participant(&p1_code);
            let p2 = diagram.get_or_create_participant(&p2_code);
            let msg_num = diagram.get_next_message_number();
            let arrow_config = plantuml_skin::ArrowConfiguration::with_direction_normal();
            let msg = Message::new(p1, p2, label, arrow_config, msg_num);
            diagram.add_message(msg);
        }
    }

    if diagram.participants().is_empty() {
        None
    } else {
        Some(ParsedSequence {
            diagram,
            svg_title,
            svg_desc,
            title,
            title_line,
            hide_footbox,
        })
    }
}

/// Parses an arrow line like "Alice -> Bob : hello".
fn parse_arrow_line(line: &str) -> Option<(String, String, String)> {
    let arrow_pos = line.find("->")?;
    let p1_code = line[..arrow_pos].trim().to_string();
    let rest = &line[arrow_pos + 2..];

    let (p2_part, label) = if let Some(colon_pos) = rest.find(':') {
        (
            rest[..colon_pos].trim().to_string(),
            rest[colon_pos + 1..].trim().to_string(),
        )
    } else {
        (rest.trim().to_string(), String::new())
    };

    if p1_code.is_empty() || p2_part.is_empty() {
        return None;
    }
    Some((p1_code, p2_part, label))
}

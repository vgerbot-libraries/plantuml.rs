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
/// Base arrow Y offset from lifeline start (paddingY + arrowDeltaY).
const ARROW_Y_BASE: f64 = 14.0;
/// Base lifeline height (without message text).
const LIFELINE_HEIGHT_BASE: f64 = 32.0;
/// Height of the title block (text height + padding).
const TITLE_HEIGHT: f64 = 35.0;

/// Note vertical margin (from `Opale.marginY`).
const NOTE_MARGIN_Y: f64 = 5.0;
/// Note fold corner size (from `Opale.cornersize`).
const NOTE_CORNERSIZE: f64 = 10.0;
/// Note horizontal padding (from `Rose.paddingX`).
const NOTE_PADDING_X: f64 = 5.0;
/// Note old padding X1 (from `ClockwiseTopRightBottomLeft` left, non-CENTER alignment).
const NOTE_OLD_PADDING_X1: f64 = 6.0;
/// Note old padding X2 (from `ClockwiseTopRightBottomLeft` right, non-CENTER alignment).
const NOTE_OLD_PADDING_X2: f64 = 15.0;

// ── Colors ───────────────────────────────────────────────────────────────

const COLOR_BACK: &str = "#E2E2F0";
const COLOR_STROKE: &str = "#181818";
const COLOR_TEXT: &str = "#000";
const COLOR_ARROW: &str = "#181818";
const COLOR_LIFELINE: &str = "#181818";
const COLOR_ACTIVATION_BAR: &str = "#00000000";
const COLOR_NOTE_BACK: &str = "#FEFFDD";

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
    notes: &[NoteInfo],
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
                let text_w = max_line_width(&bounder, &font_m, label);
                let comp_width = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
                if p1_idx + 1 < participants.len() {
                    let next_idx = p1_idx + 1;
                    let required = comp_width - head_widths[p1_idx] / 2.0 - head_widths[next_idx] / 2.0;
                    if required > min_spacing[next_idx] {
                        min_spacing[next_idx] = required;
                    }
                }
            } else if !label.is_empty() {
                let text_w = max_line_width(&bounder, &font_m, label);
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
    let mut msg_text_heights: Vec<f64> = Vec::new();
    let mut current_y = lifeline_y;
    let mut first = true;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let has_text = !msg.label().is_empty();
            let line_count = if has_text {
                msg.label().split("\\n").count()
            } else {
                0
            };
            // Text block height: 13px per line + 13px base
            let text_h = (line_count as f64) * 13.0 + 13.0;
            msg_text_heights.push(text_h);
            // Increment depends on whether the CURRENT message has a note
            let curr_has_note = notes.iter().any(|n| n.msg_index == arrow_ys.len());
            if first {
                // First arrow: arrow_y = lifeline_y + 1 + text_h
                current_y = lifeline_y + 1.0 + text_h;
                first = false;
            } else if curr_has_note {
                // With note: increment = ARROW_Y_BASE + text_h
                current_y += ARROW_Y_BASE + text_h;
            } else {
                // Without note: increment = 1 + text_h
                current_y += 1.0 + text_h;
            }
            arrow_ys.push(current_y);
            is_self_flags.push(msg.p1().code() == msg.p2().code());
        }
    }

    // Compute note heights for each message
    let mut note_heights: Vec<f64> = vec![0.0; arrow_ys.len()];
    for note in notes {
        if note.msg_index < note_heights.len() {
            let note_lines = note.text.split("\\n").count();
            let note_h = (note_lines as f64) * 13.0 + 2.0 * NOTE_MARGIN_Y;
            if note_h > note_heights[note.msg_index] {
                note_heights[note.msg_index] = note_h;
            }
        }
    }

    let last_arrow_y = arrow_ys.last().copied().unwrap_or(lifeline_y + ARROW_Y_BASE);
    let last_is_self = is_self_flags.last().copied().unwrap_or(false);
    let self_extra = if last_is_self { SELF_ARROW_HEIGHT } else { 0.0 };

    // Compute the max note bottom across all messages
    let mut max_note_bottom = 0.0_f64;
    for (i, &note_h) in note_heights.iter().enumerate() {
        if note_h > 0.0 && i < arrow_ys.len() {
            let msg_lines = msg_text_heights.get(i).map_or(1, |&h| ((h - 13.0) / 13.0) as usize);
            let note_y_top = arrow_ys[i] - ARROW_Y_BASE - (msg_lines.saturating_sub(1) as f64) * 13.0;
            let note_bottom = note_y_top + note_h;
            if note_bottom > max_note_bottom {
                max_note_bottom = note_bottom;
            }
        }
    }

    // Lifeline extends to max of: normal arrow height, or note bottom + 15
    let normal_lifeline_bottom = last_arrow_y + self_extra + ARROW_Y_BASE + 4.0; // 4 = getPaddingY
    let note_lifeline_bottom = if max_note_bottom > 0.0 {
        max_note_bottom + 15.0
    } else {
        0.0
    };
    let lifeline_bottom = normal_lifeline_bottom.max(note_lifeline_bottom);

    let lifeline_height = if arrow_ys.is_empty() {
        LIFELINE_HEIGHT_BASE
    } else {
        lifeline_bottom - lifeline_y
    };
    let footbox_y = lifeline_y + lifeline_height;

    // ── Compute total dimensions ──────────────────────────────────────────
    // First pass: compute min_layout_left using solver values (without x_offset)
    // to determine how far left notes extend, then set x_offset accordingly.
    let mut min_layout_left = 0.0_f64;
    for note in notes {
        if note.msg_index >= arrow_ys.len() {
            continue;
        }
        if note.position != NotePosition::Left {
            continue;
        }
        let msg_idx = note.msg_index;
        let mut msg_count2 = 0;
        let mut p_idx = 0;
        let mut is_self_msg = false;
        let mut is_reverse = false;
        let mut msg_label = String::new();
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                if msg_count2 == msg_idx {
                    p_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    is_self_msg = msg.is_self_message();
                    is_reverse = msg.arrow_config().is_reverse_define();
                    msg_label = msg.label().to_string();
                    break;
                }
                msg_count2 += 1;
            }
        }
        let p_center_solver = pos_c_vals.get(p_idx).copied().unwrap_or(0.0);
        let note_text_w = max_line_width(&bounder, &font_m, &note.text);
        let layout_w = note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2 + 2.0 * NOTE_PADDING_X;
        let layout_left = if is_self_msg && is_reverse {
            // LEFT note on <-- self-message: layout_left = posC - comp_width - layout_w
            let label_w = max_line_width(&bounder, &font_m, &msg_label);
            let comp_width = (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
            p_center_solver - comp_width - layout_w
        } else {
            // LEFT note on --> self-message or normal message: layout_left = posC - layout_w
            p_center_solver - layout_w
        };
        if layout_left < min_layout_left {
            min_layout_left = layout_left;
        }
    }
    let x_offset = PAGE_MARGIN * 2.0 - min_layout_left.min(0.0);
    let rightmost_x = pos_d_vals.last().copied().unwrap_or(0.0) + x_offset;
    let title_rightmost = if title_width > 0.0 {
        PAGE_MARGIN + title_width + PAGE_MARGIN + x_offset
    } else {
        0.0
    };

    // Second pass: compute max_note_right with the correct x_offset
    let mut max_note_right = 0.0_f64;
    for note in notes {
        if note.msg_index >= arrow_ys.len() {
            continue;
        }
        if note.position != NotePosition::Right {
            continue;
        }
        let msg_idx = note.msg_index;
        let mut msg_count2 = 0;
        let mut p_idx = 0;
        let mut is_self_msg = false;
        let mut is_reverse = false;
        let mut msg_label = String::new();
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                if msg_count2 == msg_idx {
                    p_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    is_self_msg = msg.is_self_message();
                    is_reverse = msg.arrow_config().is_reverse_define();
                    msg_label = msg.label().to_string();
                    break;
                }
                msg_count2 += 1;
            }
        }
        let p_center = pos_c_vals.get(p_idx).copied().unwrap_or(0.0) + x_offset;
        let note_text_w = max_line_width(&bounder, &font_m, &note.text);
        let layout_w = note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2 + 2.0 * NOTE_PADDING_X;
        let layout_right = if is_self_msg && !is_reverse {
            // RIGHT note on --> self-message: layout_right = posC + comp_width + layout_w
            let label_w = max_line_width(&bounder, &font_m, &msg_label);
            let comp_width = (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
            p_center + comp_width + layout_w
        } else {
            // RIGHT note on <-- self-message or normal message: layout_right = posC + layout_w
            p_center + layout_w
        };
        if layout_right > max_note_right {
            max_note_right = layout_right;
        }
    }

    let footbox_bottom = if hide_footbox {
        footbox_y
    } else {
        footbox_y + head_rect_height
    };
    let total_width = if title_width > 0.0 {
        rightmost_x.max(title_rightmost).max(max_note_right) + PAGE_MARGIN * 2.0 + 1.0
    } else {
        rightmost_x.max(max_note_right) + PAGE_MARGIN * 2.0
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

    // ── Force SVG dimensions ────────────────────────────────────────────
    svg.set_hidden(true);
    svg.svg_rectangle(0.0, 0.0, total_width as f64, total_height as f64, 0.0, 0.0, 0.0);
    svg.set_hidden(false);

    // ── Draw messages and notes (interleaved) ───────────────────────────
    let mut msg_idx = 0;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(1);
            let y = arrow_ys[msg_idx];

            // Draw arrow first, then note(s) for this message
            draw_message(
                &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, y,
                p1_idx, p2_idx,
            );
            for note in notes {
                if note.msg_index != msg_idx {
                    continue;
                }
                draw_note(
                    &mut svg, note, msg, &pcode_to_idx, &pos_c_vals, x_offset,
                    y, msg_idx, &msg_text_heights, &bounder, &font_m,
                );
            }
            msg_idx += 1;
        }
    }


    // ── Clean up Real constraints ───────────────────────────────────────
    plantuml_real::clear_forces(xorigin.get_line());

    svg.create_xml()
}

/// Draws a note attached to a message.
fn draw_note(
    svg: &mut SvgGraphics,
    note: &NoteInfo,
    msg: &Message,
    pcode_to_idx: &std::collections::HashMap<String, usize>,
    pos_c_vals: &[f64],
    x_offset: f64,
    y: f64,
    msg_idx: usize,
    msg_text_heights: &[f64],
    bounder: &StringBounderFromWidthTable,
    font_m: &UFont,
) {
    let p_idx = match note.position {
        NotePosition::Right => pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0),
        NotePosition::Left => pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0),
    };
    let is_self_msg = msg.is_self_message();
    let is_reverse = msg.arrow_config().is_reverse_define();
    let msg_label = msg.label().to_string();

    let p_center = pos_c_vals[p_idx] + x_offset;
    // Number of message lines for this message (for note Y offset)
    let msg_line_count = msg_text_heights.get(msg_idx).map_or(1, |&h| ((h - 13.0) / 13.0) as usize);

    // Compute note dimensions
    let lines: Vec<&str> = note.text.split("\\n").collect();
    let mut max_line_w = 0.0_f64;
    for line in &lines {
        let w = bounder.calculate_dimension(font_m, line).width();
        if w > max_line_w {
            max_line_w = w;
        }
    }
    // Note polygon width = (int)(text_width + oldPaddingX1 + oldPaddingX2)
    let text_width_total = max_line_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2;
    let polygon_w = text_width_total.trunc() as f64;
    // Note layout width = text_width + oldPaddingX1 + oldPaddingX2 + 2*paddingX
    let layout_w = text_width_total + 2.0 * NOTE_PADDING_X;
    let note_h = (lines.len() as f64) * 13.0 + 2.0 * NOTE_MARGIN_Y;

    // Note Y position: note_y_top = arrow_y - ARROW_Y_BASE - (msg_lines - 1) * 13
    let note_y_top = y - ARROW_Y_BASE - (msg_line_count.saturating_sub(1) as f64) * 13.0;

    // Compute self-message comp width if needed
    let comp_width = if is_self_msg {
        let label_w = max_line_width(bounder, font_m, &msg_label);
        (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0)
    } else {
        0.0
    };

    // Note X position: polygon is drawn at layout_position + paddingX
    // For self-messages, the layout position is offset by comp_width
    let note_x = match note.position {
        NotePosition::Right => {
            if is_self_msg && !is_reverse {
                // RIGHT on --> : polygon at posC + comp_width + paddingX
                p_center + comp_width + NOTE_PADDING_X
            } else {
                // RIGHT on <-- or normal: polygon at posC + paddingX
                p_center + NOTE_PADDING_X
            }
        }
        NotePosition::Left => {
            if is_self_msg && is_reverse {
                // LEFT on <-- : polygon at posC - comp_width - layout_w + paddingX
                p_center - comp_width - layout_w + NOTE_PADDING_X
            } else {
                // LEFT on --> or normal: polygon at posC - layout_w + paddingX
                p_center - layout_w + NOTE_PADDING_X
            }
        }
    };

    // Draw note shape (folded corner rectangle) with absolute coordinates
    svg.set_fill_color(COLOR_NOTE_BACK);
    svg.set_stroke_color(Some(COLOR_STROKE));
    svg.set_stroke_width(STROKE_WIDTH_BOX, None);
    let path_d = format_note_path_abs(note_x, note_y_top, polygon_w, note_h, NOTE_CORNERSIZE);
    svg.svg_path(&path_d, 0.0);

    // Draw fold corner
    let corner_d = format_note_corner_abs(note_x, note_y_top, polygon_w, NOTE_CORNERSIZE);
    svg.svg_path(&corner_d, 0.0);

    // Draw note text
    svg.set_fill_color(COLOR_TEXT);
    svg.set_stroke_color(None);
    svg.set_stroke_width(0.0, None);
    for (line_idx, line) in lines.iter().enumerate() {
        let text_w = bounder.calculate_dimension(font_m, line).width();
        let text_x = note_x + NOTE_OLD_PADDING_X1;
        let text_y = note_y_top + 15.111 + (line_idx as f64) * 13.0;
        svg.text(
            line,
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
    let is_reverse = msg.arrow_config().is_reverse_define();
    let is_dashed = msg.arrow_config().is_dotted();

    if is_self {
        // Self-message: draw a loop to the right (--> ) or left (<--) of the participant
        let cx = x1; // participant center
        let (loop_x, tip_dir) = if is_reverse {
            // <-- : loop to the left, arrowhead points right
            (cx - SELF_XRIGHT, -1.0)
        } else {
            // --> : loop to the right, arrowhead points left
            (cx + SELF_XRIGHT, 1.0)
        };
        let y_bottom = y + SELF_ARROW_HEIGHT;

        // Lines: top, vertical, bottom (dashed for -->/<--)
        // For --> : top at cx, bottom at cx+1
        // For <-- : top at cx-1, bottom at cx-2
        let top_near = cx + (is_reverse as i32 as f64) * tip_dir;
        let bottom_near = cx + (if is_reverse { 2.0 } else { 1.0 }) * tip_dir;
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
        if is_reverse {
            svg.svg_line(loop_x, y, top_near, y, 0.0);
            svg.svg_line(loop_x, y, loop_x, y_bottom, 0.0);
            svg.svg_line(loop_x, y_bottom, bottom_near, y_bottom, 0.0);
        } else {
            svg.svg_line(top_near, y, loop_x, y, 0.0);
            svg.svg_line(loop_x, y, loop_x, y_bottom, 0.0);
            svg.svg_line(bottom_near, y_bottom, loop_x, y_bottom, 0.0);
        }

        // Arrowhead at bottom_near pointing towards cx
        let tip_x = bottom_near;
        let base_x = tip_x + tip_dir * ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        svg.set_fill_color(COLOR_ARROW);
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        svg.svg_polygon(
            0.0,
            &[base_x, y_bottom - half, tip_x, y_bottom, base_x, y_bottom + half, base_x - 4.0 * tip_dir, y_bottom],
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
        let text_x = if is_self {
            if is_reverse {
                // <-- : text at left side of loop
                x1 - SELF_XRIGHT - 1.0
            } else {
                // --> : text at right side of participant
                x1 + MESSAGE_TEXT_X_OFFSET
            }
        } else if is_return {
            // Text starts after the arrowhead: target + 17
            x2 + MESSAGE_TEXT_X_OFFSET + ARROWHEAD_SIZE
        } else {
            x1 + MESSAGE_TEXT_X_OFFSET
        };
        // Handle multi-line text: split on literal \n
        let lines: Vec<&str> = label.split("\\n").collect();
        let text_y = y - MESSAGE_TEXT_Y_OFFSET - ((lines.len().max(1) - 1) as f64) * 13.0;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        // Handle multi-line text: split on literal \n (already split above)
        for (line_idx, line) in lines.iter().enumerate() {
            let text_w = bounder.calculate_dimension(font, line).width();
            let line_y = text_y + (line_idx as f64) * 13.0;
            svg.text(
                line,
                text_x,
                line_y,
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
}

fn format_note_path_abs(x: f64, y: f64, width: f64, height: f64, cornersize: f64) -> String {
    // The note has a folded corner at top-right:
    // (x,y) → (x,y+h) → (x+w,y+h) → (x+w,y+cs) → (x+w-cs,y) → (x,y)
    let xs = format_number_path(x);
    let ys = format_number_path(y);
    let ws = format_number_path(x + width);
    let hs = format_number_path(y + height);
    let cs = format_number_path(y + cornersize);
    let x_cs = format_number_path(x + width - cornersize);
    format!("M{xs},{ys} L{xs},{hs} L{ws},{hs} L{ws},{cs} L{x_cs},{ys} L{xs},{ys}")
}

/// Formats an SVG path for the note fold corner triangle, using absolute coordinates.
/// Ported from: `net.sourceforge.plantuml.svek.image.Opale.getCorner`.
fn format_note_corner_abs(x: f64, y: f64, width: f64, cornersize: f64) -> String {
    let x_cs = format_number_path(x + width - cornersize);
    let y_cs = format_number_path(y + cornersize);
    let w = format_number_path(x + width);
    let y0 = format_number_path(y);
    format!("M{x_cs},{y0} L{x_cs},{y_cs} L{w},{y_cs} L{x_cs},{y0}")
}

/// Formats a number for use in SVG path data (matches Java's number formatting).
/// Uses HALF_UP rounding via shortest-representation to match Java's BigDecimal behavior.
fn format_number_path(n: f64) -> String {
    let shortest = format!("{n}");
    let rounded = round_half_up_path(&shortest, 3);
    // Strip trailing zeros but keep at least one decimal
    let s = rounded.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Rounds a decimal string to `decimal` fractional digits using HALF_UP.
fn round_half_up_path(s: &str, decimal: usize) -> String {
    let neg = s.starts_with('-');
    let s = s.trim_start_matches('-');
    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    if frac_part.len() <= decimal {
        let padded = if decimal == 0 {
            int_part.to_string()
        } else {
            format!("{int_part}.{frac_part:0<decimal$}")
        };
        return if neg { format!("-{padded}") } else { padded };
    }
    let round_digit = frac_part.as_bytes()[decimal] - b'0';
    if round_digit < 5 {
        let truncated = &frac_part[..decimal];
        let result = if decimal == 0 {
            int_part.to_string()
        } else {
            format!("{int_part}.{truncated}")
        };
        if neg { format!("-{result}") } else { result }
    } else {
        let mut digits: Vec<u8> = int_part
            .bytes()
            .chain(frac_part[..decimal].bytes())
            .map(|b| b - b'0')
            .collect();
        let mut i = digits.len();
        loop {
            if i == 0 {
                digits.insert(0, 1);
                break;
            }
            i -= 1;
            digits[i] += 1;
            if digits[i] < 10 {
                break;
            }
            digits[i] = 0;
        }
        let all_digits: String = digits.iter().map(|d| (d + b'0') as char).collect();
        let result = if decimal == 0 {
            all_digits
        } else {
            let int_len = all_digits.len() - decimal;
            format!("{}.{}", &all_digits[..int_len], &all_digits[int_len..])
        };
        if neg { format!("-{result}") } else { result }
    }
}

/// Note position relative to participant.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NotePosition {
    /// Note to the left of participant.
    Left,
    /// Note to the right of participant.
    Right,
}

/// Parsed note attached to a message.
#[derive(Clone)]
pub struct NoteInfo {
    /// Note position (left or right of participant).
    pub position: NotePosition,
    /// Note text (may contain `\n` for multi-line).
    pub text: String,
    /// Index of the message this note is attached to.
    pub msg_index: usize,
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
    /// Notes attached to messages.
    pub notes: Vec<NoteInfo>,
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
    let mut notes: Vec<NoteInfo> = Vec::new();
    let mut msg_count = 0usize;

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

        // Handle "note right:" and "note left:" (single-line notes)
        if let Some(val) = trimmed.strip_prefix("note right:") {
            if msg_count > 0 {
                notes.push(NoteInfo {
                    position: NotePosition::Right,
                    text: val.trim().to_string(),
                    msg_index: msg_count - 1,
                });
            }
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("note left:") {
            if msg_count > 0 {
                notes.push(NoteInfo {
                    position: NotePosition::Left,
                    text: val.trim().to_string(),
                    msg_index: msg_count - 1,
                });
            }
            continue;
        }

        if let Some((p1_code, p2_code, label, arrow)) = parse_arrow_line(trimmed) {
            last_p1 = Some(p1_code.clone());
            last_p2 = Some(p2_code.clone());
            let p1 = diagram.get_or_create_participant(&p1_code);
            let p2 = diagram.get_or_create_participant(&p2_code);
            let msg_num = diagram.get_next_message_number();
            let arrow_config = match arrow {
                "-->" => plantuml_skin::ArrowConfiguration::with_direction_self(false)
                    .with_body(plantuml_skin::ArrowBody::Dotted),
                "<--" => plantuml_skin::ArrowConfiguration::with_direction_self(true)
                    .with_body(plantuml_skin::ArrowBody::Dotted),
                "<-" => plantuml_skin::ArrowConfiguration::with_direction_normal().reverse_define(),
                _ => plantuml_skin::ArrowConfiguration::with_direction_normal(),
            };
            let msg = Message::new(p1, p2, label, arrow_config, msg_num);
            diagram.add_message(msg);
            msg_count += 1;
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
            notes,
        })
    }
}

/// Parses an arrow line like "Alice -> Bob : hello".
/// Parses an arrow line like "Alice -> Bob : hello" or "Test --> Test: Text".
/// Handles common arrow types: ->, -->, <-, <--, ->>, <<-.
/// Returns (p1_code, p2_code, label, arrow_string).
fn parse_arrow_line(line: &str) -> Option<(String, String, String, &'static str)> {
    // Try each arrow pattern, longest first to avoid partial matches
    let arrows: [&str; 6] = ["-->", "<--", "->>", "<<-", "->", "<-"];
    for arrow in &arrows {
        if let Some(pos) = line.find(arrow) {
            let p1_code = line[..pos].trim().to_string();
            let rest = &line[pos + arrow.len()..];
            let (p2_part, label) = if let Some(colon_pos) = rest.find(':') {
                (
                    rest[..colon_pos].trim().to_string(),
                    rest[colon_pos + 1..].trim().to_string(),
                )
            } else {
                (rest.trim().to_string(), String::new())
            };
            if p1_code.is_empty() || p2_part.is_empty() {
                continue;
            }
            return Some((p1_code, p2_part, label, arrow));
        }
    }
    None
}

/// Computes the maximum line width for a (potentially multi-line) label.
/// Splits on literal `\n` and returns the width of the widest line.
fn max_line_width(bounder: &StringBounderFromWidthTable, font: &UFont, text: &str) -> f64 {
    text.split("\\n")
        .map(|line| bounder.calculate_dimension(font, line).width())
        .fold(0.0_f64, f64::max)
}

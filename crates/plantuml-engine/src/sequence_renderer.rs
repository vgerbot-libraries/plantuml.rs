//! Sequence diagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/teoz/` package
//!
//! Minimal renderer for simple `A -> B : msg` sequence diagrams.
//! Full Teoz layout engine will be ported incrementally.

use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::{FontStyle, UFont};
use plantuml_klimt::string_bounder_from_width_table::StringBounderFromWidthTable;
use plantuml_sequence::sequence_diagram::SequenceEvent;
use plantuml_sequence::{LifeEventType, Message, ParticipantType, SequenceDiagram};
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
/// Activation bar width for explicit activate/deactivate (wider than transparent bar).
const ACTIVATION_BAR_EXPLICIT_WIDTH: f64 = 10.0;
/// Activation bar offset from center for explicit activate (posC - 5).
const ACTIVATION_BAR_EXPLICIT_OFFSET: f64 = 5.0;
/// Destroy X mark half-size (extends ±9px from center).
const DESTROY_X_HALF_SIZE: f64 = 9.0;
/// Destroy X color.
const COLOR_DESTROY: &str = "#A80036";
/// Destroy X stroke width.
const DESTROY_X_STROKE_WIDTH: f64 = 2.0;
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

// ── Group frame constants (from GroupingTile.java) ────────────────────────

/// Horizontal margin inside group frame (MARGINX in GroupingTile).
const GROUP_MARGIN_X: f64 = 16.0;
/// Vertical margin around group frame (EXTERNAL_MARGINY in GroupingTile).
const GROUP_MARGIN_Y: f64 = 4.0;
/// Magic vertical margin (MARGINY_MAGIC in GroupingTile).
const GROUP_MARGIN_Y_MAGIC: f64 = 20.0;
/// Group header tab height (from ComponentGroupingHeaderTeoz preferred height).
const GROUP_HEADER_HEIGHT: f64 = 15.0;
/// Group header offset added to first message Y inside a group.
/// = header_height + MARGINY_MAGIC/2 + EXTERNAL_MARGINY = 15 + 10 + 4.
const GROUP_HEADER_OFFSET: f64 = 29.0;
/// Extra header height for partitions: TITLE_VPAD*2 - (GROUP_HEADER_HEIGHT - font_height) = 4*2 - (15-13) = 6.
/// Partition titles are drawn directly on the frame (no tab), with 4px padding above and below.
const PARTITION_HEADER_EXTRA: f64 = 6.0;
/// Gap between consecutive group frames = 2*EXTERNAL_MARGINY + MARGINY_MAGIC/2.
const GROUP_GAP: f64 = 18.0;
/// Else tile height (from ComponentRoseGroupingElse.getPreferredHeight in teoz mode).
/// = getTextHeight(13) + 4 = 17.
const ELSE_TILE_HEIGHT: f64 = 17.0;
/// Group frame stroke width.
const GROUP_STROKE_WIDTH: f64 = 1.5;
/// Group header tab corner cut size.
const GROUP_TAB_CORNER: f64 = 10.0;
/// External margin X1 (left side of group frame, from GroupingTile.EXTERNAL_MARGINX1).
const GROUP_EXTERNAL_MARGIN_X1: f64 = 3.0;
/// External margin X2 (right side of group frame, from GroupingTile.EXTERNAL_MARGINX2).
const GROUP_EXTERNAL_MARGIN_X2: f64 = 9.0;
/// Group header text left padding from frame x.
const GROUP_TEXT_PADDING: f64 = 15.0;
/// Group header text Y offset from frame y (ascent for 13px bold).
const GROUP_TEXT_Y_OFFSET: f64 = 11.111;
/// Group header fill color.
const COLOR_GROUP_HEADER: &str = "#EEE";
/// Group frame stroke color.
const COLOR_GROUP_STROKE: &str = "#000";

/// Wraps message text to fit within max_width, splitting by words.
/// Ported from Java's StringBounder word-wrap logic for MaxMessageSize.
fn wrap_message_text(
    bounder: &StringBounderFromWidthTable,
    font: &UFont,
    label: &str,
    max_width: f64,
) -> Vec<String> {
    let mut result = Vec::new();
    for explicit_line in label.split("\\n") {
        let words: Vec<&str> = explicit_line.split_whitespace().collect();
        if words.is_empty() {
            result.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in &words {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            let candidate_w = bounder.calculate_dimension(font, &candidate).width();
            if candidate_w <= max_width || current.is_empty() {
                current = candidate;
            } else {
                result.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            result.push(current);
        }
    }
    result
}
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
    groups: &[GroupInfo],
    msg_activates: &[Vec<String>],
    msg_deactivates: &[Vec<String>],
    msg_parallel: &[bool],
    max_message_size: Option<f64>,
    msg_exo: &[Option<ExoType>],
) -> String {
    let bounder = StringBounderFromWidthTable::new(FileFormat::Svg);
    let font_p = UFont::sans_serif(FONT_SIZE_PARTICIPANT);
    let font_m = UFont::sans_serif(FONT_SIZE_MESSAGE);

    let participants = diagram.participants();

    // ── Compute participant head dimensions ──────────────────────────────
    // Actor stickman dimensions (from ActorStickMan.java)
    const STICKMAN_HEAD_DIAM: f64 = 16.0;
    const STICKMAN_BODY_LEN: f64 = 27.0;
    const STICKMAN_LEGS_Y: f64 = 15.0;
    const STICKMAN_ARMS_LEN: f64 = 13.0;
    const STICKMAN_LEGS_X: f64 = 13.0;
    const STICKMAN_THICKNESS: f64 = 0.5;
    const STICKMAN_HEIGHT: f64 =
        STICKMAN_HEAD_DIAM + STICKMAN_BODY_LEN + STICKMAN_LEGS_Y + 2.0 * STICKMAN_THICKNESS + 1.0; // 60
    const STICKMAN_WIDTH: f64 = STICKMAN_ARMS_LEN.max(STICKMAN_LEGS_X) * 2.0 + 2.0 * STICKMAN_THICKNESS; // 27
    const ACTOR_PADDING_H: f64 = 3.0; // Actor horizontal padding (from ComponentRoseActor)
    // Actor text height: textBlock height + padding(0,0) = same as regular text block height
    // Actor head height = stickman(60) + text_block(19) = 79 in Java
    // head_layout_height = headHeight - STARTING_Y = 79 - 5 = 74
    const ACTOR_HEAD_LAYOUT_HEIGHT: f64 = 74.0; // headHeight(79) - STARTING_Y(5)

    let mut head_widths: Vec<f64> = Vec::with_capacity(participants.len());
    let mut is_actor: Vec<bool> = Vec::with_capacity(participants.len());
    for p in participants {
        let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
        let actor = p.ptype() == ParticipantType::Actor;
        is_actor.push(actor);
        if actor {
            // Actor: max(stickman_width, text_width + ACTOR_PADDING_H + ACTOR_PADDING_H)
            // Java: getTextWidth = text_width + padding.left + padding.right; padding=(0,3,0,3)
            head_widths.push(STICKMAN_WIDTH.max(text_w + ACTOR_PADDING_H + ACTOR_PADDING_H));
        } else {
            // Java: getPreferredWidth = getTextWidth + margin.left + margin.right
            // getTextWidth = text_width + padding.left + padding.right; padding=(5,7,5,7)
            // Must add padding.left and padding.right separately to match Java's float addition order
            head_widths.push(text_w + PADDING_H + PADDING_H);
        }
    }
    // head_layout_height = max of all participants' head layout heights
    let has_actor = is_actor.iter().any(|&a| a);
    let head_layout_height = if has_actor {
        ACTOR_HEAD_LAYOUT_HEIGHT.max(TEXT_BLOCK_HEIGHT + HEIGHT_EXTRA)
    } else {
        TEXT_BLOCK_HEIGHT + HEIGHT_EXTRA
    };
    // head_rect_height for footbox_bottom computation: the max head component height
    let head_rect_height = if has_actor {
        (ACTOR_HEAD_LAYOUT_HEIGHT).max(TEXT_BLOCK_HEIGHT) // 74 for actors
    } else {
        TEXT_BLOCK_HEIGHT
    };

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
    // Pre-pass: compute activation levels per message for spacing and frame dimensions
    let mut pre_participant_levels: Vec<i32> = vec![0; participants.len()];
    let mut max_participant_levels: Vec<i32> = vec![0; participants.len()];
    let mut pre_msg_self_levels: Vec<(i32, i32)> = Vec::new();
    // Track activation levels at each message for spacing constraints
    let mut msg_p1_levels: Vec<i32> = Vec::new();
    let mut msg_p2_levels: Vec<i32> = Vec::new();
    {
        let mut mi = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                let is_self = msg_exo.get(mi).copied().flatten().is_none() && msg.p1().code() == msg.p2().code();
                if is_self {
                    let p_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let level_ignore = pre_participant_levels.get(p_idx).copied().unwrap_or(0);
                    let future_acts = msg_activates.get(mi).map(|v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32).unwrap_or(0);
                    let future_deacts = msg_deactivates.get(mi).map(|v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32).unwrap_or(0);
                    let level_considere = (level_ignore + future_acts - future_deacts).max(0);
                    pre_msg_self_levels.push((level_ignore, level_considere));
                } else {
                    pre_msg_self_levels.push((0, 0));
                }
                // Record activation levels at this message for IGNORE_FUTURE_DEACTIVATE mode.
                // Java getLevelAt() with IGNORE_FUTURE_DEACTIVATE:
                // - Includes activations up to and including this message (lookahead for same-message activates)
                // - Ignores future deactivations (does NOT subtract inline deactivations)
                let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                let p1_level = pre_participant_levels.get(p1_idx).copied().unwrap_or(0);
                let p2_level = pre_participant_levels.get(p2_idx).copied().unwrap_or(0);
                // Add inline activations attached to this message (lookahead in Java)
                let inline_acts_p1 = msg_activates.get(mi).map(|v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32).unwrap_or(0);
                let inline_acts_p2 = msg_activates.get(mi).map(|v| v.iter().filter(|c| *c == msg.p2().code()).count() as i32).unwrap_or(0);
                let p1_level_for_constraint = p1_level + inline_acts_p1;
                let p2_level_for_constraint = p2_level + inline_acts_p2;
                msg_p1_levels.push(p1_level_for_constraint);
                msg_p2_levels.push(p2_level_for_constraint);
                mi += 1;
            } else if let SequenceEvent::LifeEvent(le) = event {
                let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                if p_idx < pre_participant_levels.len() {
                    if le.is_activate() {
                        pre_participant_levels[p_idx] += 1;
                        max_participant_levels[p_idx] = max_participant_levels[p_idx].max(pre_participant_levels[p_idx]);
                    } else if le.is_deactivate() || le.is_destroy() {
                        pre_participant_levels[p_idx] = (pre_participant_levels[p_idx] - 1).max(0);
                    }
                }
            }
        }
    }
    let msg_self_levels = pre_msg_self_levels;
    // Precompute wrapped lines and their max widths for all messages
    let mut pre_wrapped_lines: Vec<Vec<String>> = Vec::new();
    let mut pre_wrapped_widths: Vec<f64> = Vec::new();
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let label = msg.label();
            let wrapped = if !label.is_empty() {
                if let Some(max_w) = max_message_size {
                    wrap_message_text(&bounder, &font_m, label, max_w)
                } else {
                    label.split("\\n").map(String::from).collect()
                }
            } else {
                Vec::new()
            };
            let max_w = wrapped.iter()
                .map(|l| bounder.calculate_dimension(&font_m, l).width())
                .fold(0.0_f64, f64::max);
            pre_wrapped_lines.push(wrapped);
            pre_wrapped_widths.push(max_w);
        }
    }
    let mut min_spacing = vec![PARTICIPANT_SPACING; participants.len().max(1)];
    // Each constraint stores: (lo, hi, point1_offset, point2_pre_offset, arrow_width)
    // Matching Java CommunicationTile.addConstraints() exactly:
    // Forward: point2.ensureBiggerThan(point1.addFixed(width))
    //   where point2 = posC[hi] + (-5 if level2>0 else 0), point1 = posC[lo]
    // Reverse: point1.ensureBiggerThan(point2.addFixed(width))
    //   where point1 = posC[hi] + (-5 if level1>0 else 0), point2 = posC[lo] + level2*5
    let mut nonadjacent_constraints: Vec<(usize, usize, f64, f64, f64)> = Vec::new();
    // Exo arrow constraints: (participant_idx, arrow_width) — posC[p] >= xOrigin + width
    let mut exo_constraints: Vec<(usize, f64)> = Vec::new();

    let mut spacing_msg_idx = 0usize;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
            let label = msg.label();
            let text_w = pre_wrapped_widths[spacing_msg_idx];
            let exo = msg_exo.get(spacing_msg_idx).copied().flatten();
            if let Some(exo_type) = exo {
                // Exo arrows: skip regular constraint handling
                // TO_RIGHT: no constraint (arrow extends right from participant)
                // FROM_LEFT: posC[p] >= xOrigin + width (participant far enough from left border)
                if exo_type == ExoType::FromLeft {
                    exo_constraints.push((p2_idx, text_w + 24.0));
                }
                spacing_msg_idx += 1;
                continue;
            }
            if p1_idx == p2_idx {
                // Self-message: posC2 = posC + getMaxPosition() (global max activation level)
                let max_act = *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                let comp_width = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0) + ACTIVATION_BAR_EXPLICIT_OFFSET * max_act;
                if p1_idx + 1 < participants.len() {
                    let next_idx = p1_idx + 1;
                    let required = comp_width - head_widths[p1_idx] / 2.0 - head_widths[next_idx] / 2.0;
                    if required > min_spacing[next_idx] {
                        min_spacing[next_idx] = required;
                    }
                }
            } else if !label.is_empty() {
                let lo = p1_idx.min(p2_idx);
                let hi = p1_idx.max(p2_idx);
                let is_reverse = p1_idx > p2_idx;
                let level_p1 = msg_p1_levels.get(spacing_msg_idx).copied().unwrap_or(0);
                let level_p2 = msg_p2_levels.get(spacing_msg_idx).copied().unwrap_or(0);
                // Match Java CommunicationTile.addConstraints() exactly.
                // For messages without activation, use posB constraint (matching original behavior).
                // For messages with activation, use posC constraint with exact Java addFixed chain.
                let has_activation = if is_reverse {
                    level_p1 > 0 || level_p2 > 0
                } else {
                    level_p2 > 0
                };
                if !has_activation {
                    // No activation: use posB constraint (posB[hi] >= posD[lo] + required)
                    let required = text_w + 24.0 - head_widths[lo] / 2.0 - head_widths[hi] / 2.0;
                    if hi - lo == 1 {
                        if required > min_spacing[hi] {
                            min_spacing[hi] = required;
                        }
                    } else {
                        nonadjacent_constraints.push((lo, hi, 0.0, 0.0, text_w + 24.0));
                    }
                } else {
                    // With activation: use posC constraint matching Java's exact addFixed chain
                    let point1_offset = if is_reverse {
                        if level_p1 > 0 { -ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 }
                    } else {
                        if level_p2 > 0 { -ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 }
                    };
                    let point2_pre_offset = if is_reverse {
                        level_p2 as f64 * ACTIVATION_BAR_EXPLICIT_OFFSET
                    } else {
                        0.0
                    };
                    nonadjacent_constraints.push((lo, hi, point1_offset, point2_pre_offset, text_w + 24.0));
                }
            }
            spacing_msg_idx += 1;
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
    // Message constraints matching Java CommunicationTile.addConstraints() exactly.
    // Java creates point1/point2 Real objects with addFixed, then ensureBiggerThan.
    // The exact chain of addFixed operations matters for floating point precision.
    for &(lo, hi, p1_off, p2_pre, arrow_w) in &nonadjacent_constraints {
        // point1 = posC[hi] + p1_off  (p1_off = -5 if activation, else 0)
        // point2 = posC[lo] + p2_pre  (p2_pre = level2*5 for reverse, else 0)
        // constraint = point2 + arrow_w
        // ensure_bigger_than(point1, constraint)
        let mut point1 = pos_c[hi].clone();
        if p1_off != 0.0 {
            point1 = plantuml_real::add_fixed(&point1, p1_off);
        }
        let mut point2 = pos_c[lo].clone();
        if p2_pre != 0.0 {
            point2 = plantuml_real::add_fixed(&point2, p2_pre);
        }
        let constraint = plantuml_real::add_fixed(&point2, arrow_w);
        plantuml_real::ensure_bigger_than(&point1, &constraint);
    }
    // Apply exo arrow constraints: posC[p] >= xOrigin + width (for FROM_LEFT exo arrows)
    for &(p_idx, arrow_w) in &exo_constraints {
        let constraint = plantuml_real::add_fixed(&xorigin, arrow_w);
        plantuml_real::ensure_bigger_than(&pos_c[p_idx], &constraint);
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

    // ── Compute Y positions for each message ─────────────────────────────
    //
    // The increment depends on whether the PREVIOUS message had a note
    // (the note extends below the arrow, pushing the next message down).
    // Group header offsets are added to the first message in each group.

    // Build msg_index → group index map
    let n_msgs: usize = diagram.events().iter().filter(|e| matches!(e, SequenceEvent::Message(_))).count();
    let mut msg_group: Vec<Option<usize>> = vec![None; n_msgs];
    for (gi, group) in groups.iter().enumerate() {
        for mi in group.msg_start..group.msg_end {
            if mi < msg_group.len() {
                // Prefer the innermost (highest nesting level) group
                let prev_level = msg_group[mi]
                    .and_then(|prev_gi| groups.get(prev_gi))
                    .map(|g| g.nesting_level)
                    .unwrap_or(0);
                if group.nesting_level >= prev_level {
                    msg_group[mi] = Some(gi);
                }
            }
        }
    }

    let mut arrow_ys: Vec<f64> = Vec::new();
    let mut is_self_flags: Vec<bool> = Vec::new();
    let mut msg_text_heights: Vec<f64> = Vec::new();
    let mut msg_wrapped_lines: Vec<Vec<String>> = Vec::new();
    let mut current_y = lifeline_y;
    let mut prev_is_self = false;
    let mut prev_note_h: Option<f64> = None;
    let mut prev_group: Option<usize> = None;
    let mut prev_frame_bottom: Option<f64> = None;
    let mut prev_frame_bottom_level: usize = 0;
    let mut msg_idx = 0usize;
    let mut current_position = lifeline_y;
    // Activation tracking: (participant_idx, start_y, end_y)
    let mut activations: Vec<(usize, f64, f64)> = Vec::new();
    // Destroy tracking: (participant_idx, destroy_y)
    let mut destroys: Vec<(usize, f64)> = Vec::new();
    // Pending activation: (participant_idx, start_y)
    let mut pending_activations: Vec<(usize, f64)> = Vec::new();
    // Per-participant activation level (running count of activates minus deactivates)
    let mut participant_levels: Vec<i32> = vec![0; participants.len()];
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let has_text = !msg.label().is_empty();
            // Use precomputed wrapped lines
            let wrapped_lines = pre_wrapped_lines[msg_idx].clone();
            let line_count = wrapped_lines.len();
            msg_wrapped_lines.push(wrapped_lines);
            let text_h = (line_count as f64) * 13.0 + 13.0;
            msg_text_heights.push(text_h);
            let exo = msg_exo.get(msg_idx).copied().flatten();
            let is_self = exo.is_none() && msg.p1().code() == msg.p2().code();
            is_self_flags.push(is_self);
            let curr_group = msg_group.get(msg_idx).copied().flatten();
            let is_first_in_group = curr_group.is_some() && curr_group != prev_group
                && curr_group.map(|gi| msg_idx == groups[gi].msg_start).unwrap_or(false);
            let is_else = curr_group
                .map(|gi| groups[gi].group_type == "else")
                .unwrap_or(false);
            let is_partition = curr_group
                .map(|gi| groups[gi].group_type == "partition")
                .unwrap_or(false);
            let header_extra = if is_partition { PARTITION_HEADER_EXTRA } else { 0.0 };
            let is_parallel = msg_parallel.get(msg_idx).copied().unwrap_or(false);

            if is_parallel && msg_idx > 0 {
                // Parallel message: starts at the previous tile's min + contactRelative.
                // For a group (contact=null), falls back to group's min = initial Y.
                // In SVG coords: lifeline_y + startingY(8) + contactRelative(19) = lifeline_y + 1 + text_h
                current_y = lifeline_y + 1.0 + text_h;
            } else if msg_idx == 0 {
                // First message overall
                current_y = lifeline_y + 1.0 + text_h;
                if is_first_in_group {
                    // Add GROUP_HEADER_OFFSET for each nesting level (outer + inner groups)
                    let curr_level = curr_group
                        .and_then(|gi| groups.get(gi))
                        .map(|g| g.nesting_level)
                        .unwrap_or(0);
                    current_y += GROUP_HEADER_OFFSET * (curr_level as f64 + 1.0) + header_extra;
                }
            } else if is_first_in_group && is_else {
                // Else section: normal increment + else tile height (no group gap/header)
                // Ported from ElseTile YGauge: min = prev_tile_max, height = ELSE_TILE_HEIGHT
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                current_y += inc + ELSE_TILE_HEIGHT;
            } else if is_first_in_group && curr_group.map(|gi| groups[gi].parallel).unwrap_or(false) {
                // Parallel group: top-aligned with the previous tile's chaining point.
                // Same Y as the first group's first message (not stacked below).
                let curr_level = curr_group
                    .and_then(|gi| groups.get(gi))
                    .map(|g| g.nesting_level)
                    .unwrap_or(0);
                current_y = lifeline_y + 1.0 + text_h + GROUP_HEADER_OFFSET * (curr_level as f64 + 1.0) + header_extra;
            } else if is_first_in_group {
                // First message in a subsequent group (non-else)
                let curr_level = curr_group
                    .and_then(|gi| groups.get(gi))
                    .map(|g| g.nesting_level)
                    .unwrap_or(0);
                let prev_level = prev_group
                    .and_then(|gi| groups.get(gi))
                    .map(|g| g.nesting_level)
                    .unwrap_or(0);
                if curr_level > prev_level {
                    // Nested group within an else/parent: normal increment + header offset
                    let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                    let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                    current_y += inc + GROUP_HEADER_OFFSET + header_extra;
                } else if let Some(fb) = prev_frame_bottom {
                    current_y = fb + GROUP_GAP + GROUP_HEADER_OFFSET + GROUP_HEADER_HEIGHT + header_extra;
                } else {
                    // First group after non-grouped messages
                    let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                    let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                    current_y += inc + GROUP_HEADER_OFFSET + header_extra;
                }
            } else if let Some(nh) = prev_note_h {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                current_y += nh.max(1.0 + text_h + prev_self_extra);
            } else {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                current_y += 1.0 + text_h + prev_self_extra;
            }


            // After a group ends, ensure next message/else clears the frame bottom
            if !is_parallel {
                if let Some(fb) = prev_frame_bottom {
                    let fb_level = prev_frame_bottom_level;
                    let curr_level = curr_group
                        .and_then(|gi| groups.get(gi))
                        .map(|g| g.nesting_level)
                        .unwrap_or(0);
                    if is_first_in_group && is_else {
                        if curr_level < fb_level {
                            let min_y = fb + GROUP_GAP + ARROW_Y_BASE + ELSE_TILE_HEIGHT + 1.0;
                            if current_y < min_y {
                                current_y = min_y;
                            }
                        }
                    } else if !is_first_in_group {
                        let min_y = fb + GROUP_GAP + ARROW_Y_BASE + 1.0;
                        if current_y < min_y {
                            current_y = min_y;
                        }
                    }
                    prev_frame_bottom = None;
                }
            }
            arrow_ys.push(current_y);
            prev_note_h = notes.iter().find(|n| n.msg_index == msg_idx).map(|note| {
                let note_lines = note.text.split("\\n").count();
                (note_lines as f64) * 13.0 + 2.0 * NOTE_MARGIN_Y + NOTE_CORNERSIZE
            });
            prev_is_self = is_self;
            // Update current_position for LifeEvent Y tracking
            current_position = current_y + if is_self { SELF_ARROW_HEIGHT } else { 0.0 };

            // If this is the last message in a group, compute frame bottom
            if let Some(gi) = curr_group {
                let group = &groups[gi];
                if msg_idx + 1 >= group.msg_end {
                    // Compute bodyHeight from actual arrow Y positions (accounts for sub-group spacing)
                    let last_mi = group.msg_end - 1;
                    let last_text_h = msg_text_heights[last_mi];
                    let last_is_self = is_self_flags[last_mi];
                    let last_msg_h = if last_is_self {
                        1.0 + last_text_h + SELF_ARROW_HEIGHT
                    } else {
                        1.0 + last_text_h
                    };
                    let last_note = notes.iter().find(|n| n.msg_index == last_mi);
                    let body_height = if let Some(note) = last_note {
                        let note_lines = note.text.split("\\n").count();
                        let note_h = (note_lines as f64) * 13.0
                            + 2.0 * NOTE_MARGIN_Y
                            + NOTE_CORNERSIZE;
                        last_msg_h.max(note_h)
                    } else {
                        last_msg_h
                    };
                    let body_height = arrow_ys[last_mi] - arrow_ys[group.msg_start] + body_height;
                    let p_extra = if group.group_type == "partition" { PARTITION_HEADER_EXTRA } else { 0.0 };
                    let (frame_y, frame_height) = if group.group_type == "else" {
                        let first_text_h = msg_text_heights[group.msg_start];
                        let fy = arrow_ys[group.msg_start] - ELSE_TILE_HEIGHT - 2.0 - first_text_h + GROUP_MARGIN_Y_MAGIC / 2.0;
                        let fh = body_height + GROUP_MARGIN_Y_MAGIC / 2.0;
                        (fy, fh)
                    } else {
                        let fy = arrow_ys[group.msg_start] - GROUP_HEADER_OFFSET - GROUP_HEADER_HEIGHT - p_extra;
                        let fh = body_height + GROUP_HEADER_HEIGHT + GROUP_MARGIN_Y_MAGIC / 2.0 + p_extra;
                        (fy, fh)
                    };
                    let fb = frame_y + frame_height;
                    // For parallel groups, take max with existing prev_frame_bottom
                    // (parallel siblings should not reduce the overall bottom)
                    if group.parallel {
                        prev_frame_bottom = Some(prev_frame_bottom.map(|existing| existing.max(fb)).unwrap_or(fb));
                    } else {
                        prev_frame_bottom = Some(fb);
                    }
                    prev_frame_bottom_level = group.nesting_level;
                }
            }

            prev_group = curr_group;

            msg_idx += 1;
        } else if let SequenceEvent::LifeEvent(le) = event {
            let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
            if le.is_activate() {
                pending_activations.push((p_idx, current_position));
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] += 1;
                }
            } else if le.is_deactivate() {
                if let Some((pi, start_y)) = pending_activations.pop() {
                    activations.push((pi, start_y, current_position));
                }
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] = (participant_levels[p_idx] - 1).max(0);
                }
            } else if le.is_destroy() {
                if let Some((pi, start_y)) = pending_activations.pop() {
                    activations.push((pi, start_y, current_position));
                }
                destroys.push((p_idx, current_position));
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] = (participant_levels[p_idx] - 1).max(0);
                }
            }
        }
    }

    // ── Compute group frame dimensions ──────────────────────────────────
    // Precompute innermost nesting level at each msg_start
    let mut msg_start_innermost: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for group in groups.iter() {
        if group.group_type == "else" { continue; }
        let entry = msg_start_innermost.entry(group.msg_start).or_insert(0);
        if group.nesting_level > *entry {
            *entry = group.nesting_level;
        }
    }
    let mut group_frames: Vec<(f64, f64, f64, f64)> = Vec::new(); // (x, y, w, h)
    for (gi, group) in groups.iter().enumerate() {
        // Compute bodyHeight from actual arrow Y positions (accounts for sub-group spacing)
        let last_mi = group.msg_end - 1;
        let last_text_h = msg_text_heights[last_mi];
        let last_is_self = is_self_flags[last_mi];
        let last_msg_h = if last_is_self {
            1.0 + last_text_h + SELF_ARROW_HEIGHT
        } else {
            1.0 + last_text_h
        };
        let last_note = notes.iter().find(|n| n.msg_index == last_mi);
        let last_h = if let Some(note) = last_note {
            let note_lines = note.text.split("\\n").count();
            let note_h = (note_lines as f64) * 13.0
                + 2.0 * NOTE_MARGIN_Y
                + NOTE_CORNERSIZE;
            last_msg_h.max(note_h)
        } else {
            last_msg_h
        };
        let body_height = arrow_ys[last_mi] - arrow_ys[group.msg_start] + last_h;
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut msg_iter_idx = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                if msg_iter_idx >= group.msg_start && msg_iter_idx < group.msg_end {
                    let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    let exo = msg_exo.get(msg_iter_idx).copied().flatten();
                    let is_self = exo.is_none() && msg.p1().code() == msg.p2().code();
                    let is_reverse = if is_self {
                        msg.arrow_config().is_reverse_define()
                    } else if exo.is_some() {
                        false
                    } else {
                        pos_c_vals[p1_idx] > pos_c_vals[p2_idx]
                    };
                    let text_h = msg_text_heights[msg_iter_idx];
                    let _ = text_h; // Used in X range computation below
                    let mi_note = notes.iter().find(|n| n.msg_index == msg_iter_idx);
                    // Drawn X range
                    let p1_c = pos_c_vals[p1_idx];
                    let p2_c = pos_c_vals[p2_idx];
                    let (self_drawn_w, mut d_min, mut d_max) = if let Some(exo_type) = exo {
                        // Exo arrow: drawn range is [posC - width, posC] or [posC, posC + width]
                        let label_w = pre_wrapped_widths[msg_iter_idx];
                        let exo_w = label_w + 24.0;
                        match exo_type {
                            ExoType::ToRight => (0.0, p1_c, p1_c + exo_w),
                            ExoType::FromLeft => (0.0, p2_c - exo_w, p2_c),
                        }
                    } else {
                        let self_drawn_w = if is_self {
                            let label_w = pre_wrapped_widths[msg_iter_idx];
                            let max_act = *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                            SELF_XRIGHT.max(MESSAGE_TEXT_X_OFFSET + label_w) + ACTIVATION_BAR_EXPLICIT_OFFSET * max_act
                        } else {
                            0.0
                        };
                        let (d_min, d_max) = if is_self {
                            if is_reverse {
                                (p1_c - self_drawn_w, p1_c)
                            } else {
                                (p1_c, p1_c + self_drawn_w)
                            }
                        } else {
                            (p1_c.min(p2_c), p1_c.max(p2_c))
                        };
                        (self_drawn_w, d_min, d_max)
                    };

                    // Note X range — for self-messages, the note is drawn at the
                    // edge of the self-message's drawn extent (not at posC), so the
                    // note's drawn extent is ADDED to the self-message's drawn extent.
                    // Java: CommunicationTileSelfNoteLeft.getDrawnMinX() = tile.getDrawnMinX() - noteWidth
                    //        CommunicationTileSelfNoteRight.getDrawnMaxX() = tile.getDrawnMaxX() + noteWidth
                    if let Some(note) = mi_note {
                        let note_text_w = max_line_width(&bounder, &font_m, &note.text);
                        let note_comp_w = note_text_w
                            + NOTE_OLD_PADDING_X1
                            + NOTE_OLD_PADDING_X2
                            + 2.0 * NOTE_PADDING_X;
                        if is_self {
                            // For self-messages, the note's drawn extent depends on
                            // which side of the self-message loop the note is on:
                            // - LEFT on reverse (<--): note is LEFT of drawn left edge
                            //   Java: getDrawnMinX() = tile.getDrawnMinX() - noteWidth
                            // - RIGHT on forward (-->): note is RIGHT of drawn right edge
                            //   Java: getDrawnMaxX() = tile.getDrawnMaxX() + noteWidth
                            // - RIGHT on reverse (<--): note is RIGHT of posC (tile.getMaxX = posC)
                            // - LEFT on forward (-->): note is LEFT of posC (tile.getMinX = posC)
                            match note.position {
                                NotePosition::Left if is_reverse => {
                                    d_min = d_min.min((p1_c - self_drawn_w) - note_comp_w);
                                }
                                NotePosition::Right if !is_reverse => {
                                    d_max = d_max.max((p1_c + self_drawn_w) + note_comp_w);
                                }
                                NotePosition::Right => {
                                    // RIGHT on reverse: note at posC, extends right
                                    d_max = d_max.max(p1_c + note_comp_w);
                                }
                                NotePosition::Left => {
                                    // LEFT on forward: note at posC, extends left
                                    d_min = d_min.min(p1_c - note_comp_w);
                                }
                            }
                        } else {
                            match note.position {
                                NotePosition::Right => {
                                    let note_p_idx = if is_reverse { p1_idx } else { p2_idx };
                                    let note_max = pos_c_vals[note_p_idx] + note_comp_w;
                                    d_max = d_max.max(note_max);
                                }
                                NotePosition::Left => {
                                    let note_p_idx = if is_reverse { p2_idx } else { p1_idx };
                                    let note_min = pos_c_vals[note_p_idx] - note_comp_w;
                                    d_min = d_min.min(note_min);
                                }
                            }
                        }
                    }

                    min_x = min_x.min(d_min);
                    max_x = max_x.max(d_max);
                }
                msg_iter_idx += 1;
            }
        }


        let p_extra = if group.group_type == "partition" { PARTITION_HEADER_EXTRA } else { 0.0 };
        let (frame_y, frame_height) = if group.group_type == "else" {
            // Else divider Y = arrow_y - ELSE_TILE_HEIGHT - 2 - text_h + MARGINY_MAGIC/2
            // (extra -1 accounts for text_h being 26 vs Java's preferredHeight 25)
            let first_text_h = msg_text_heights[group.msg_start];
            let fy = arrow_ys[group.msg_start] - ELSE_TILE_HEIGHT - 2.0 - first_text_h + GROUP_MARGIN_Y_MAGIC / 2.0;
            // Else frame height = body_height (from first to last msg) + MARGINY_MAGIC/2
            let fh = body_height + GROUP_MARGIN_Y_MAGIC / 2.0;
            (fy, fh)
        } else {
            // Frame Y: subtract GROUP_HEADER_OFFSET for each nesting level from this group
            // to the innermost group at the same msg_start.
            // E.g., opt (nesting=0) with alt (nesting=1) at same msg_start:
            //   opt subtracts 2 headers, alt subtracts 1 header.
            let innermost = *msg_start_innermost.get(&group.msg_start).unwrap_or(&group.nesting_level);
            let headers_to_subtract = (innermost + 1) - group.nesting_level;
            // For wrapped text, subtract the extra text height so the frame Y
            // stays at the group's chaining point (independent of text wrapping).
            // Single-line text_h = 26; extra = text_h - 26.
            let first_text_h = msg_text_heights[group.msg_start];
            let extra_text_h = (first_text_h - 26.0).max(0.0);
            let fy = arrow_ys[group.msg_start] - extra_text_h - GROUP_HEADER_OFFSET * headers_to_subtract as f64 - GROUP_HEADER_HEIGHT - p_extra;
            let fh = body_height + GROUP_HEADER_HEIGHT + GROUP_MARGIN_Y_MAGIC / 2.0 + p_extra;
            (fy, fh)
        };
        let frame_x = min_x - GROUP_MARGIN_X;
        let content_span = max_x - min_x + 2.0 * GROUP_MARGIN_X;
        let frame_width = content_span; // Header width added after nesting extension
        group_frames.push((frame_x, frame_y, frame_width, frame_height));
        let _ = gi;
    }

    // Compute header widths for all non-else/non-partition groups
    let mut header_widths: Vec<f64> = vec![0.0; groups.len()];
    for (gi, group) in groups.iter().enumerate() {
        if group.group_type == "else" || group.group_type == "partition" {
            continue;
        }
        let tab_label = if group.group_type == "group" {
            if group.comment.is_empty() { "group".to_string() } else { group.comment.clone() }
        } else {
            group.group_type.clone()
        };
        let tab_label_w = max_line_width(&bounder, &font_m, &tab_label);
        let tab_width = tab_label_w + 3.0 * GROUP_TEXT_PADDING;
        header_widths[gi] = if group.group_type != "group" && !group.comment.is_empty() {
            let cond_label = format!("[{}]", group.comment);
            let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
            let cond_w = max_line_width(&bounder, &font_small, &cond_label);
            tab_width + GROUP_TEXT_PADDING + cond_w
        } else {
            tab_width
        };
    }
    // Track nesting Y extension for each group (to adjust lifeline bottom)
    let mut nesting_y_ext: Vec<f64> = vec![0.0; groups.len()];
    // Else extension: extend parent (non-else) groups to cover else sections' content bounds.
    // Must run BEFORE nesting extension so nesting sees the extended parent frames.
    for gi in 0..groups.len() {
        if groups[gi].group_type != "else" {
            continue;
        }
        let level = groups[gi].nesting_level;
        let mut parent_gi: Option<usize> = None;
        for pj in (0..gi).rev() {
            if groups[pj].nesting_level == level && groups[pj].group_type != "else" {
                parent_gi = Some(pj);
                break;
            }
        }
        if let Some(pgi) = parent_gi {
            let (pfx, pfy, pfw, pfh) = group_frames[pgi];
            let (efx, efy, efw, efh) = group_frames[gi];
            let new_left = pfx.min(efx);
            let new_right = (pfx + pfw).max(efx + efw);
            let new_width = new_right - new_left;
            let else_bottom = efy + efh + GROUP_MARGIN_Y_MAGIC / 2.0 - GROUP_MARGIN_Y;
            let new_height = pfh.max(else_bottom - pfy);
            group_frames[pgi] = (new_left, pfy, new_width, new_height);
        }
    }
    // Nesting extension pass: process groups from innermost (highest nesting) to outermost.
    // Each parent's frame must include its children's drawn bounds plus MARGINX.
    // Also extends parent's height to cover child's frame bottom.
    let mut nesting_order: Vec<usize> = (0..groups.len()).collect();
    nesting_order.sort_by_key(|&i| std::cmp::Reverse(groups[i].nesting_level));
    for &ci in &nesting_order {
        let child_level = groups[ci].nesting_level;
        if child_level == 0 {
            continue;
        }
        let cs = groups[ci].msg_start;
        let ce = groups[ci].msg_end;
        let mut parent_gi: Option<usize> = None;
        for pj in 0..groups.len() {
            if pj == ci {
                continue;
            }
            if groups[pj].nesting_level == child_level - 1
                && groups[pj].msg_start <= cs
                && groups[pj].msg_end >= ce
            {
                if parent_gi.is_none()
                    || (groups[pj].msg_end - groups[pj].msg_start)
                        < (groups[parent_gi.unwrap()].msg_end - groups[parent_gi.unwrap()].msg_start)
                {
                    parent_gi = Some(pj);
                }
            }
        }
        if let Some(pgi) = parent_gi {
            let (cfx, cfy, cfw, cfh) = group_frames[ci];
            let cframe_min = cfx;
            let cframe_max = cfx + cfw;
            let header_max = cfx + header_widths[ci] + GROUP_MARGIN_X;
            let cdrawn_min = cframe_min - GROUP_EXTERNAL_MARGIN_X1;
            let cdrawn_max = cframe_max.max(header_max) + GROUP_EXTERNAL_MARGIN_X2;
            let (pfx, pfy, pfw, pfh) = group_frames[pgi];
            let pframe_min = pfx;
            let pframe_max = pfx + pfw;
            let new_min = pframe_min.min(cdrawn_min - GROUP_MARGIN_X);
            let new_max = pframe_max.max(cdrawn_max + GROUP_MARGIN_X);
            // Extend parent height to cover child frame bottom.
            // Parent frame bottom = child frame bottom + (GROUP_HEADER_OFFSET - GROUP_HEADER_HEIGHT)
            // for each nesting level difference.
            let cframe_bottom = cfy + cfh;
            let pframe_bottom = pfy + pfh;
            let level_diff = child_level - groups[pgi].nesting_level;
            let new_bottom = pframe_bottom.max(cframe_bottom + (GROUP_HEADER_OFFSET - GROUP_HEADER_HEIGHT) * level_diff as f64);
            let new_height = new_bottom - pfy;
            // Track how much the parent's bottom was extended by nesting.
            // Use child_frame_bottom as the baseline (not pframe_bottom) because
            // the parent's frame may not yet cover the child's else-extended frame.
            let baseline = pframe_bottom.max(cframe_bottom);
            if new_bottom > baseline {
                nesting_y_ext[pgi] += new_bottom - baseline;
            }
            group_frames[pgi] = (new_min, pfy, new_max - new_min, new_height);
        }
    }
    // Apply header width after nesting extension: frame_max >= frame_min + header_width + MARGIN_X
    for (gi, group) in groups.iter().enumerate() {
        if group.group_type == "else" || group.group_type == "partition" {
            continue;
        }
        let (fx, fy, fw, fh) = group_frames[gi];
        let frame_max = fx + fw;
        let header_max = fx + header_widths[gi] + GROUP_MARGIN_X;
        if header_max > frame_max {
            group_frames[gi] = (fx, fy, header_max - fx, fh);
        }
    }
    // Second else extension: re-extend parents to cover else groups' nesting-extended frames
    for gi in 0..groups.len() {
        if groups[gi].group_type != "else" {
            continue;
        }
        let level = groups[gi].nesting_level;
        let mut parent_gi: Option<usize> = None;
        for pj in (0..gi).rev() {
            if groups[pj].nesting_level == level && groups[pj].group_type != "else" {
                parent_gi = Some(pj);
                break;
            }
        }
        if let Some(pgi) = parent_gi {
            let (pfx, pfy, pfw, pfh) = group_frames[pgi];
            let (efx, efy, efw, efh) = group_frames[gi];
            let new_left = pfx.min(efx);
            let new_right = (pfx + pfw).max(efx + efw);
            let new_width = new_right - new_left;
            let else_bottom = efy + efh + GROUP_MARGIN_Y_MAGIC / 2.0 - GROUP_MARGIN_Y;
            let new_height = pfh.max(else_bottom - pfy);
            group_frames[pgi] = (new_left, pfy, new_width, new_height);
        }
    }
    // Match else groups' X/width to their parent's final X/width
    for gi in 0..groups.len() {
        if groups[gi].group_type != "else" {
            continue;
        }
        let level = groups[gi].nesting_level;
        let mut parent_gi: Option<usize> = None;
        for pj in (0..gi).rev() {
            if groups[pj].nesting_level == level && groups[pj].group_type != "else" {
                parent_gi = Some(pj);
                break;
            }
        }
        if let Some(pgi) = parent_gi {
            let (pfx, _, pfw, _) = group_frames[pgi];
            let (_, efy, _, efh) = group_frames[gi];
            group_frames[gi] = (pfx, efy, pfw, efh);
        }
    }
    for (gi, gf) in group_frames.iter().enumerate() {
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

    // Use max arrow Y (parallel messages can have Y values out of order)
    let (last_arrow_y, last_is_self) = arrow_ys
        .iter()
        .zip(is_self_flags.iter())
        .max_by(|(ya, _), (yb, _)| ya.partial_cmp(yb).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(&y, &s)| (y, s))
        .unwrap_or((lifeline_y + ARROW_Y_BASE, false));

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
    let self_extra = if last_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
    let normal_lifeline_bottom = last_arrow_y + self_extra + ARROW_Y_BASE + 4.0; // 4 = getPaddingY
    let note_lifeline_bottom = if max_note_bottom > 0.0 {
        max_note_bottom + 15.0
    } else {
        0.0
    };
    // Compute group_lifeline_bottom based on nesting and parallel messages.
    // - Groups with nesting extension: frame_bottom - nesting_y_ext + MARGINY_MAGIC + MARGINY (24)
    // - Groups without nesting, no parallel msgs: frame_bottom + 24
    let has_parallel = msg_parallel.iter().any(|&p| p);
    let has_self_msgs = is_self_flags.iter().any(|&s| s);
    let java_preferred_height = 25.0; // Java's message preferredHeight (textHeight=13 + arrowDeltaY=4 + 2*paddingY=8)
    let msg_ygauge_max = arrow_ys.iter().map(|&y| y + java_preferred_height).fold(0.0_f64, f64::max);

    let group_lifeline_bottom = group_frames
        .iter()
        .enumerate()
        .map(|(i, &(_, fy, _, fh))| {
            let frame_bottom = fy + fh;
            // Check if this group or any of its else children have nesting Y extension.
            let g = &groups[i];
            let has_nesting = nesting_y_ext[i] > 0.0 || {
                (0..groups.len()).any(|j| {
                    if groups[j].group_type != "else" || groups[j].nesting_level != g.nesting_level {
                        return false;
                    }
                    let parent = (0..j).rev()
                        .find(|&pj| groups[pj].nesting_level == groups[j].nesting_level
                            && groups[pj].group_type != "else");
                    parent == Some(i) && nesting_y_ext[j] > 0.0
                })
            };
            let val = if has_nesting {
                // -1 correction for text_h=26 vs Java's 25, only when self-messages present
                frame_bottom - nesting_y_ext[i] + GROUP_MARGIN_Y_MAGIC + GROUP_MARGIN_Y - if has_self_msgs { 1.0 } else { 0.0 }
            } else if has_parallel {
                msg_ygauge_max + PAGE_MARGIN
            } else {
                frame_bottom + GROUP_MARGIN_Y_MAGIC + GROUP_MARGIN_Y
            };
            val
        })
        .fold(0.0_f64, f64::max);
    // When parallel messages are present, cap normal_lifeline_bottom at msg_ygauge_max + PAGE_MARGIN
    // to avoid 1px overshoot from text_h=26 vs Java's 25 in self-messages.
    let normal_lifeline_bottom = if has_parallel {
        normal_lifeline_bottom.min(msg_ygauge_max + PAGE_MARGIN)
    } else {
        normal_lifeline_bottom
    };
    let lifeline_bottom = normal_lifeline_bottom.max(note_lifeline_bottom).max(group_lifeline_bottom);

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
                    let p1_idx_n = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let p2_idx_n = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    is_self_msg = msg.is_self_message();
                    is_reverse = if is_self_msg {
                        msg.arrow_config().is_reverse_define()
                    } else {
                        pos_c_vals[p1_idx_n] > pos_c_vals[p2_idx_n]
                    };
                    p_idx = if is_reverse { p2_idx_n } else { p1_idx_n };
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
            let label_w = pre_wrapped_widths.get(msg_idx).copied().unwrap_or_else(|| max_line_width(&bounder, &font_m, &msg_label));
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
    // Also account for group frame left edges (frame_x - EXTERNAL_MARGINX1)
    let mut min_left = min_layout_left;
    for &(fx, _, _, _) in &group_frames {
        let frame_left = fx - GROUP_EXTERNAL_MARGIN_X1;
        if frame_left < min_left {
            min_left = frame_left;
        }
    }
    let x_offset = PAGE_MARGIN * 2.0 - min_left.min(0.0);
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
                    let p1_idx_n = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let p2_idx_n = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    is_self_msg = msg.is_self_message();
                    is_reverse = if is_self_msg {
                        msg.arrow_config().is_reverse_define()
                    } else {
                        pos_c_vals[p1_idx_n] > pos_c_vals[p2_idx_n]
                    };
                    p_idx = if is_reverse { p1_idx_n } else { p2_idx_n };
                    msg_label = msg.label().to_string();
                    break;
                }
                msg_count2 += 1;
            }
        }
        let p_center = pos_c_vals.get(p_idx).copied().unwrap_or(0.0) + x_offset;
        let note_text_w = max_line_width(&bounder, &font_m, &note.text);
        let layout_w = note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2 + 2.0 * NOTE_PADDING_X;
        // Java CommunicationTileNoteRight.getNotePosition = posC + level * LIVE_DELTA_SIZE
        // getMaxX = getNotePosition + getPreferredWidth = posC + level*5 + layout_w
        let note_level = if is_reverse { msg_p1_levels.get(msg_idx).copied().unwrap_or(0) } else { msg_p2_levels.get(msg_idx).copied().unwrap_or(0) };
        let level_dx = (note_level as f64) * ACTIVATION_BAR_EXPLICIT_OFFSET;
        let layout_right = if is_self_msg && !is_reverse {
            // RIGHT note on --> self-message: layout_right = posC + comp_width + level_dx + layout_w
            let label_w = pre_wrapped_widths.get(msg_idx).copied().unwrap_or_else(|| max_line_width(&bounder, &font_m, &msg_label));
            let comp_width = (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
            p_center + comp_width + level_dx + layout_w
        } else {
            // RIGHT note on <-- self-message or normal message: layout_right = posC + level_dx + layout_w
            p_center + level_dx + layout_w
        };
        if layout_right > max_note_right {
            max_note_right = layout_right;
        }
    }
    let max_frame_right: f64 = group_frames
        .iter()
        .map(|&(fx, _, fw, _)| fx + fw + GROUP_EXTERNAL_MARGIN_X2 + x_offset)
        .fold(0.0_f64, f64::max);

    let footbox_bottom = if hide_footbox {
        footbox_y
    } else {
        footbox_y + head_rect_height
    };
    let total_width = if title_width > 0.0 {
        rightmost_x.max(title_rightmost).max(max_note_right).max(max_frame_right) + PAGE_MARGIN * 2.0 + 1.0
    } else {
        rightmost_x.max(max_note_right).max(max_frame_right) + PAGE_MARGIN * 2.0
    };

    // Override partition frame dimensions to span the full diagram width
    let full_right = rightmost_x.max(max_note_right).max(max_frame_right);
    for (gi, group) in groups.iter().enumerate() {
        if group.group_type == "partition" {
            let (_, fy, _, fh) = group_frames[gi];
            let frame_x = PAGE_MARGIN * 2.0 - x_offset;
            let frame_width = full_right - PAGE_MARGIN * 2.0;
            group_frames[gi] = (frame_x, fy, frame_width, fh);
        }
    }
    // HEIGHT_EXTRA (1px) is only needed for show_footbox: the +1 from ensure_visible
    // already provides the extra pixel for hide_footbox.
    let height_extra = if hide_footbox { 0.0 } else { HEIGHT_EXTRA };
    let total_height = footbox_bottom + PAGE_MARGIN * 2.0 + height_extra;

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

    // ── Draw group frame backgrounds (sorted by msg_start for correct nesting order) ──
    let mut bg_order: Vec<usize> = (0..groups.len()).collect();
    bg_order.sort_by_key(|&i| (groups[i].msg_start, groups[i].nesting_level));
    for &gi in &bg_order {
        // Skip else sections — they don't have their own frame
        if groups[gi].group_type == "else" {
            continue;
        }
        let (fx, fy, fw, fh) = group_frames[gi];
        svg.set_fill_color("none");
        if groups[gi].group_type == "partition" {
            // Partition: background rect (stroke:none) + label + border, all in background pass
            svg.set_stroke_color(None);
            svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
            svg.svg_rectangle(fx + x_offset, fy, fw, fh, 0.0, 0.0, 0.0);

            let tab_label = if groups[gi].comment.is_empty() {
                "partition".to_string()
            } else {
                groups[gi].comment.clone()
            };
            let label_w = max_line_width(&bounder, &font_m, &tab_label);
            svg.set_fill_color(COLOR_TEXT);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            let label_x = fx + x_offset + (fw - label_w) / 2.0;
            svg.text(
                &tab_label,
                label_x,
                fy + GROUP_TEXT_Y_OFFSET + 3.0,
                None,
                FONT_SIZE_MESSAGE,
                Some("700"),
                None,
                None,
                label_w,
                &indexmap::IndexMap::new(),
                None,
            );

            svg.set_fill_color("none");
            svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
            svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
            svg.svg_rectangle(fx + x_offset, fy, fw, fh, 0.0, 0.0, 0.0);
        } else {
            svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
            svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
            svg.svg_rectangle(fx + x_offset, fy, fw, fh, 0.0, 0.0, 0.0);
        }
    }

    // ── Draw lifelines (background) ───────────────────────────────────────
    // ── Draw lifelines + activation bars + destroy marks (per participant) ──
    for (i, p) in participants.iter().enumerate() {
        let cx = pos_c_vals[i] + x_offset;
        let ly = lifeline_y;

        // Check if this participant is destroyed
        let destroy_y = destroys.iter().find(|(pi, _)| *pi == i).map(|(_, y)| *y);
        let ll_height = if let Some(dy) = destroy_y {
            dy + DESTROY_X_HALF_SIZE - 1.0 - ly
        } else {
            lifeline_height
        };

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
            ll_height,
            0.0,
            0.0,
            0.0,
        );

        // Lifeline (dashed)
        svg.set_stroke_color(Some(COLOR_LIFELINE));
        svg.set_stroke_width(STROKE_WIDTH_LIFELINE, Some([5.0, 5.0]));
        svg.svg_line(cx, ly, cx, ly + ll_height, 0.0);

        svg.close_group();

        // Draw activation bars for this participant
        for &(pi, start_y, end_y) in &activations {
            if pi != i {
                continue;
            }
            svg.open_group(None);
            svg.title("");
            svg.set_fill_color("#FFF");
            svg.set_stroke_color(Some(COLOR_LIFELINE));
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(
                cx - ACTIVATION_BAR_EXPLICIT_OFFSET,
                start_y,
                ACTIVATION_BAR_EXPLICIT_WIDTH,
                end_y - start_y,
                0.0,
                0.0,
                0.0,
            );
            svg.close_group();
        }

        // Draw destroy X marks for this participant
        for &(pi, dy) in &destroys {
            if pi != i {
                continue;
            }
            svg.set_fill_color("none");
            svg.set_stroke_color(Some(COLOR_DESTROY));
            svg.set_stroke_width(DESTROY_X_STROKE_WIDTH, None);
            svg.svg_line(cx - DESTROY_X_HALF_SIZE, dy - DESTROY_X_HALF_SIZE, cx + DESTROY_X_HALF_SIZE, dy + DESTROY_X_HALF_SIZE, 0.0);
            svg.svg_line(cx - DESTROY_X_HALF_SIZE, dy + DESTROY_X_HALF_SIZE, cx + DESTROY_X_HALF_SIZE, dy - DESTROY_X_HALF_SIZE, 0.0);
        }
    }

    // ── Draw participant heads ───────────────────────────────────────────
    for (i, p) in participants.iter().enumerate() {
        let x = pos_b_vals[i] + x_offset;
        let w = head_widths[i];
        let cx = pos_c_vals[i] + x_offset;

        if is_actor[i] {
            // Actor: draw stickman + text below
            // Stickman head center: (cx, head_y + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM/2)
            let head_cy = head_y + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
            svg.set_fill_color(COLOR_BACK);
            svg.set_stroke_color(Some(COLOR_STROKE));
            svg.set_stroke_width(STICKMAN_THICKNESS * 2.0, None);
            svg.svg_ellipse(cx, head_cy, STICKMAN_HEAD_DIAM / 2.0, STICKMAN_HEAD_DIAM / 2.0, 0.0);

            // Body + arms + legs as path
            let body_top = head_cy + STICKMAN_HEAD_DIAM / 2.0; // head_y + 0.5 + 16 = head_y + 16.5
            let body_bottom = body_top + STICKMAN_BODY_LEN; // head_y + 43.5
            let arms_y = body_top + 8.0; // head_y + 24.5 (armsY = 8)
            let legs_bottom = body_bottom + STICKMAN_LEGS_Y; // head_y + 58.5

            svg.set_fill_color("none");
            svg.set_stroke_color(Some(COLOR_STROKE));
            svg.set_stroke_width(STICKMAN_THICKNESS * 2.0, None);
            let path_d = format!(
                "M{cx},{body_top} L{cx},{body_bottom} M{arms_l},{arms_y} L{arms_r},{arms_y} M{cx},{body_bottom} L{leg_lx},{legs_bottom} M{cx},{body_bottom} L{leg_rx},{legs_bottom}",
                cx = format_number_path(cx),
                body_top = format_number_path(body_top),
                body_bottom = format_number_path(body_bottom),
                arms_y = format_number_path(arms_y),
                arms_l = format_number_path(cx - STICKMAN_ARMS_LEN),
                arms_r = format_number_path(cx + STICKMAN_ARMS_LEN),
                legs_bottom = format_number_path(legs_bottom),
                leg_lx = format_number_path(cx - STICKMAN_LEGS_X),
                leg_rx = format_number_path(cx + STICKMAN_LEGS_X),
            );
            svg.svg_path(&path_d, 0.0);

            // Text below stickman
            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = cx - text_w / 2.0;
            let text_y = head_y + STICKMAN_HEIGHT + ASCENT_14 - 2.0; // Adjust for actor text position

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
        } else {
            // Regular participant: draw rectangle + text inside
            svg.set_fill_color(COLOR_BACK);
            svg.set_stroke_color(Some(COLOR_STROKE));
            svg.set_stroke_width(STROKE_WIDTH_BOX, None);
            svg.svg_rectangle(x, head_y, w, TEXT_BLOCK_HEIGHT, ROUND_CORNER, ROUND_CORNER, 0.0);

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
    }

    // ── Draw footboxes ───────────────────────────────────────────────────
    if !hide_footbox {
    for (i, p) in participants.iter().enumerate() {
        let x = pos_b_vals[i] + x_offset;
        let w = head_widths[i];
        let cx = pos_c_vals[i] + x_offset;

        if is_actor[i] {
            // Actor footbox: text above + stickman below (reversed from head)
            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = cx - text_w / 2.0;
            let text_y = footbox_y + ASCENT_14 - 2.0;

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

            // Stickman below text
            let stickman_top = footbox_y + TEXT_BLOCK_HEIGHT; // text area height
            let head_cy = stickman_top + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
            svg.set_fill_color(COLOR_BACK);
            svg.set_stroke_color(Some(COLOR_STROKE));
            svg.set_stroke_width(STICKMAN_THICKNESS * 2.0, None);
            svg.svg_ellipse(cx, head_cy, STICKMAN_HEAD_DIAM / 2.0, STICKMAN_HEAD_DIAM / 2.0, 0.0);

            let body_top = head_cy + STICKMAN_HEAD_DIAM / 2.0;
            let body_bottom = body_top + STICKMAN_BODY_LEN;
            let arms_y = body_top + 8.0;
            let legs_bottom = body_bottom + STICKMAN_LEGS_Y;

            svg.set_fill_color("none");
            svg.set_stroke_color(Some(COLOR_STROKE));
            svg.set_stroke_width(STICKMAN_THICKNESS * 2.0, None);
            let path_d = format!(
                "M{cx},{body_top} L{cx},{body_bottom} M{arms_l},{arms_y} L{arms_r},{arms_y} M{cx},{body_bottom} L{leg_lx},{legs_bottom} M{cx},{body_bottom} L{leg_rx},{legs_bottom}",
                cx = format_number_path(cx),
                body_top = format_number_path(body_top),
                body_bottom = format_number_path(body_bottom),
                arms_y = format_number_path(arms_y),
                arms_l = format_number_path(cx - STICKMAN_ARMS_LEN),
                arms_r = format_number_path(cx + STICKMAN_ARMS_LEN),
                legs_bottom = format_number_path(legs_bottom),
                leg_lx = format_number_path(cx - STICKMAN_LEGS_X),
                leg_rx = format_number_path(cx + STICKMAN_LEGS_X),
            );
            svg.svg_path(&path_d, 0.0);
        } else {
            // Regular participant: draw rectangle + text inside
            svg.set_fill_color(COLOR_BACK);
            svg.set_stroke_color(Some(COLOR_STROKE));
            svg.set_stroke_width(STROKE_WIDTH_BOX, None);
            svg.svg_rectangle(x, footbox_y, w, TEXT_BLOCK_HEIGHT, ROUND_CORNER, ROUND_CORNER, 0.0);

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
    }

    // ── Force SVG dimensions ────────────────────────────────────────────
    svg.set_hidden(true);
    svg.svg_rectangle(0.0, 0.0, total_width as f64, total_height as f64, 0.0, 0.0, 0.0);
    svg.set_hidden(false);

    // ── Draw group headers + messages + notes (interleaved) ──────────────
    let mut msg_idx = 0;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            // Draw ALL group headers that start at this message index,
            // from outermost (lowest nesting) to innermost (highest nesting).
            let mut groups_at_msg: Vec<usize> = (0..groups.len())
                .filter(|&gi| groups[gi].msg_start == msg_idx && groups[gi].group_type != "else")
                .collect();
            groups_at_msg.sort_by_key(|&gi| groups[gi].nesting_level);
            for gi in groups_at_msg {
                let (fx, fy, fw, fh) = group_frames[gi];
                let fx_off = fx + x_offset;
                let group = &groups[gi];

                // For non-else, non-partition groups: draw header tab + label + condition
                if group.group_type == "partition" {
                        // Partition: centered label + separate border rect
                        let tab_label = if group.comment.is_empty() {
                            "partition".to_string()
                        } else {
                            group.comment.clone()
                        };
                        let label_w = max_line_width(&bounder, &font_m, &tab_label);

                        // Label text (13px bold, centered in frame)
                        svg.set_fill_color(COLOR_TEXT);
                        svg.set_stroke_color(None);
                        svg.set_stroke_width(0.0, None);
                        let label_x = fx_off + (fw - label_w) / 2.0;
                        svg.text(
                            &tab_label,
                            label_x,
                            fy + GROUP_TEXT_Y_OFFSET + 3.0,
                            None,
                            FONT_SIZE_MESSAGE,
                            Some("700"),
                            None,
                            None,
                            label_w,
                            &indexmap::IndexMap::new(),
                            None,
                        );

                        // Border rect (drawn after label, with stroke)
                        svg.set_fill_color("none");
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                        svg.svg_rectangle(fx_off, fy, fw, fh, 0.0, 0.0, 0.0);
                    } else {
                        // Determine tab label: for "group" use comment (or "group"),
                        // for other types use the keyword itself
                        let tab_label = if group.group_type == "group" {
                            if group.comment.is_empty() {
                                "group".to_string()
                            } else {
                                group.comment.clone()
                            }
                        } else {
                            group.group_type.clone()
                        };
                        let label_w = max_line_width(&bounder, &font_m, &tab_label);
                        let tab_w = label_w + GROUP_TEXT_PADDING + 2.0 * GROUP_TEXT_PADDING;

                        // Header tab path (folded corner)
                        svg.set_fill_color(COLOR_GROUP_HEADER);
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                        let path = format!(
                            "M{x},{y} L{x2},{y} L{x2},{y2} L{x3},{y3} L{x},{y3} L{x},{y}",
                            x = format_number_path(fx_off),
                            y = format_number_path(fy),
                            x2 = format_number_path(fx_off + tab_w),
                            y2 = format_number_path(fy + 5.0),
                            x3 = format_number_path(fx_off + tab_w - GROUP_TAB_CORNER),
                            y3 = format_number_path(fy + GROUP_HEADER_HEIGHT),
                        );
                        svg.svg_path(&path, 0.0);

                        // Frame rect (foreground, same as background)
                        svg.set_fill_color("none");
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                        svg.svg_rectangle(fx_off, fy, fw, fh, 0.0, 0.0, 0.0);

                        // Tab label text (13px bold)
                        svg.set_fill_color(COLOR_TEXT);
                        svg.set_stroke_color(None);
                        svg.set_stroke_width(0.0, None);
                        svg.text(
                            &tab_label,
                            fx_off + GROUP_TEXT_PADDING,
                            fy + GROUP_TEXT_Y_OFFSET,
                            None,
                            FONT_SIZE_MESSAGE,
                            Some("700"),
                            None,
                            None,
                            label_w,
                            &indexmap::IndexMap::new(),
                            None,
                        );

                        // Condition text (11px bold, in brackets) for non-group types
                        if group.group_type != "group" && !group.comment.is_empty() {
                            let cond_label = format!("[{}]", group.comment);
                            let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
                            let cond_w = max_line_width(&bounder, &font_small, &cond_label);
                            svg.set_fill_color(COLOR_TEXT);
                            svg.set_stroke_color(None);
                            svg.set_stroke_width(0.0, None);
                            svg.text(
                                &cond_label,
                                fx_off + tab_w + GROUP_TEXT_PADDING,
                                fy + 10.556,
                                None,
                                11,
                                Some("700"),
                                None,
                                None,
                                cond_w,
                                &indexmap::IndexMap::new(),
                                None,
                            );
                        }
                    }
            }
            // Draw else dividers for else groups starting at this message
            for gi in 0..groups.len() {
                if groups[gi].group_type != "else" || groups[gi].msg_start != msg_idx {
                    continue;
                }
                let (fx, fy, fw, _fh) = group_frames[gi];
                let fx_off = fx + x_offset;
                let group = &groups[gi];
                // Dashed divider line across the parent frame
                svg.set_fill_color("none");
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(1.0, Some([2.0, 2.0]));
                svg.svg_line(fx_off, fy, fx_off + fw, fy, 0.0);
                // Else label text (11px bold, in brackets)
                if !group.comment.is_empty() {
                    let else_label = format!("[{}]", group.comment);
                    let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
                    let label_w = max_line_width(&bounder, &font_small, &else_label);
                    svg.set_fill_color(COLOR_TEXT);
                    svg.set_stroke_color(None);
                    svg.set_stroke_width(0.0, None);
                    svg.text(
                        &else_label,
                        fx_off + 5.0,
                        fy + 10.556,
                        None,
                        11,
                        Some("700"),
                        None,
                        None,
                        label_w,
                        &indexmap::IndexMap::new(),
                        None,
                    );
                }
            }

            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(1);
            let y = arrow_ys[msg_idx];

            // Draw arrow first, then note(s) for this message
            let exo = msg_exo.get(msg_idx).copied().flatten();
            if let Some(exo_type) = exo {
                draw_exo_message(
                    &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, y,
                    p1_idx, exo_type,
                    msg_wrapped_lines.get(msg_idx).map(|v| v.as_slice()).unwrap_or(&[]),
                );
            } else {
                draw_message(
                    &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, y,
                    p1_idx, p2_idx,
                    msg_self_levels.get(msg_idx).map(|&(li, lc)| li).unwrap_or(0),
                    msg_self_levels.get(msg_idx).map(|&(li, lc)| lc).unwrap_or(0),
                    msg_wrapped_lines.get(msg_idx).map(|v| v.as_slice()).unwrap_or(&[]),
                    msg_p1_levels.get(msg_idx).copied().unwrap_or(0),
                    msg_p2_levels.get(msg_idx).copied().unwrap_or(0),
                );
            }
            for note in notes {
                if note.msg_index != msg_idx {
                    continue;
                }
                draw_note(
                    &mut svg, note, msg, &pcode_to_idx, &pos_c_vals, x_offset,
                    y, msg_idx, &msg_text_heights, &bounder, &font_m,
                    pre_wrapped_widths.get(msg_idx).copied().unwrap_or(0.0),
                    msg_p1_levels.get(msg_idx).copied().unwrap_or(0),
                    msg_p2_levels.get(msg_idx).copied().unwrap_or(0),
                );
            }
            msg_idx += 1;
        }
    }


    // ── Clean up Real constraints ───────────────────────────────────────
    plantuml_real::clear_forces(xorigin.get_line());

    svg.create_xml()
}

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
    pre_wrapped_width: f64,
    msg_p1_level: i32,
    msg_p2_level: i32,
) {
    let is_self_msg = msg.is_self_message();
    let is_reverse_syntax = msg.arrow_config().is_reverse_define();
    let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);

    let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
    // For self-messages, use arrow-syntax isReverse (CommunicationTileSelf.isReverseDefine).
    // For non-self messages, use position-based isReverse (posC[p1] > posC[p2]).
    let is_reverse = if is_self_msg {
        is_reverse_syntax
    } else {
        pos_c_vals[p1_idx] > pos_c_vals[p2_idx]
    };
    let p_idx = match note.position {
        NotePosition::Right => if is_reverse { p1_idx } else { p2_idx },
        NotePosition::Left => if is_reverse { p2_idx } else { p1_idx },
    };

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
        (pre_wrapped_width + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0)
    } else {
        0.0
    };

    // Note X position: polygon is drawn at getNotePosition + getPaddingX()
    // For RIGHT notes: getNotePosition = posC + level * LIVE_DELTA_SIZE
    // For LEFT notes: getNotePosition = posC - getPreferredWidth (no level offset)
    // For self-messages, the layout position is offset by comp_width
    let note_level = match note.position {
        NotePosition::Right => if is_reverse { msg_p1_level } else { msg_p2_level },
        NotePosition::Left => 0, // LEFT notes don't have level offset in Java
    };
    let level_dx = (note_level as f64) * ACTIVATION_BAR_EXPLICIT_OFFSET;
    let note_x = match note.position {
        NotePosition::Right => {
            if is_self_msg && !is_reverse {
                // RIGHT on --> : polygon at posC + comp_width + level_dx + paddingX
                p_center + comp_width + level_dx + NOTE_PADDING_X
            } else {
                // RIGHT on <-- or normal: polygon at posC + level_dx + paddingX
                p_center + level_dx + NOTE_PADDING_X
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
    level_ignore: i32,
    level_considere: i32,
    wrapped_lines: &[String],
    msg_p1_level: i32,
    msg_p2_level: i32,
) {
    let x1_raw = pos_c[p1_idx] + x_offset;
    let x2_raw = pos_c[p2_idx] + x_offset;
    let is_self = p1_idx == p2_idx;
    let is_return = p1_idx > p2_idx;
    let is_reverse = msg.arrow_config().is_reverse_define();
    let is_dashed = msg.arrow_config().is_dotted();
    let ld = ACTIVATION_BAR_EXPLICIT_OFFSET; // LIVE_DELTA_SIZE = 5
    let max_level = level_ignore.max(level_considere) as f64;

    // Java CommunicationTile.drawU(): adjust x1/x2 based on activation levels
    // For non-reverse: x1 += ld * level1; x2 += ld * (level2 > 0 ? level2 - 2 : level2)
    // For reverse: x1 -= ld if level1==1; x1 += ld*(level1-2) if level1>2; x2 += ld * level2
    // Self-messages handle activation levels separately (CommunicationTileSelf.drawU)
    let (x1, x2) = if is_self {
        (x1_raw, x2_raw)
    } else if is_return {
        let mut x1 = x1_raw;
        let mut x2 = x2_raw;
        if msg_p1_level == 1 {
            x1 -= ld;
        } else if msg_p1_level > 2 {
            x1 += ld * (msg_p1_level as f64 - 2.0);
        }
        x2 += ld * (msg_p2_level as f64);
        (x1, x2)
    } else {
        let x1 = x1_raw + ld * (msg_p1_level as f64);
        let level2_adj = if msg_p2_level > 0 { msg_p2_level - 2 } else { msg_p2_level };
        let x2 = x2_raw + ld * (level2_adj as f64);
        (x1, x2)
    };

    if is_self {
        // Self-message: draw a loop to the right (--> ) or left (<--) of the participant
        // Activation levels adjust X positions (from CommunicationTileSelf.drawU + ComponentRoseSelfArrow.drawRightSide)
        let cx = x1; // participant center (posC)
        let delta_x1 = (level_ignore - level_considere) as f64 * ld;

        let (loop_x, tip_dir) = if is_reverse {
            // <-- : loop to the left, arrowhead points right
            (cx - ld * max_level - SELF_XRIGHT, -1.0)
        } else {
            // --> : loop to the right, arrowhead points left
            (cx + ld * max_level + SELF_XRIGHT, 1.0)
        };
        let y_bottom = y + SELF_ARROW_HEIGHT;

        // Compute top and bottom near points based on activation levels
        // For --> (non-reverse):
        //   top_start = posC + LD * levelIgnore
        //   bottom_start = posC + LD * levelConsidere + (1 if deltaX1 <= 0 else 0)
        // For <-- (reverse): mirror with -tip_dir
        let top_near = if is_reverse {
            cx - ld * level_ignore as f64 - 1.0
        } else {
            cx + ld * level_ignore as f64
        };
        let bottom_near = if is_reverse {
            cx - ld * level_considere as f64 - (if delta_x1 >= 0.0 { 2.0 } else { 1.0 })
        } else {
            cx + ld * level_considere as f64 + (if delta_x1 <= 0.0 { 1.0 } else { 0.0 })
        };

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

        // Line: from target+5 to source-1 (dashed only if arrow is dotted)
        svg.set_stroke_color(Some(COLOR_ARROW));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
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
        // Use wrapped lines if provided, otherwise split by \\n
        let lines: Vec<&str> = if !wrapped_lines.is_empty() {
            wrapped_lines.iter().map(String::as_str).collect()
        } else {
            label.split("\\n").collect()
        };
        let text_x = if is_self {
            if is_reverse {
                // <-- : text at left edge of component
                // comp_width = max(SELF_XRIGHT, MESSAGE_TEXT_X_OFFSET + label_width)
                // When comp_width == SELF_XRIGHT (loop wider than text), text is 1px left of loop
                let label_w = lines.iter().map(|l| bounder.calculate_dimension(font, l).width()).fold(0.0_f64, f64::max);
                let comp_width = SELF_XRIGHT.max(MESSAGE_TEXT_X_OFFSET + label_w);
                x1 - ld * max_level - comp_width.max(SELF_XRIGHT + 1.0)
            } else {
                // --> : text at right side of participant, offset by activation level
                x1 + ld * max_level + MESSAGE_TEXT_X_OFFSET
            }
        } else if is_return {
            // Text starts after the arrowhead: target + 17
            x2 + MESSAGE_TEXT_X_OFFSET + ARROWHEAD_SIZE
        } else {
            x1 + MESSAGE_TEXT_X_OFFSET
        };
        let text_y = y - MESSAGE_TEXT_Y_OFFSET - ((lines.len().max(1) - 1) as f64) * 13.0;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        // When text is wrapped (MaxMessageSize active), render each word as a separate
        // <text> element with textLength, matching Java PlantUML's text block rendering.
        // Otherwise, render each line as a single <text> element (existing behavior).
        let is_wrapped = !wrapped_lines.is_empty() && wrapped_lines.len() > label.split("\\n").count();
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = text_y + (line_idx as f64) * 13.0;
            if is_wrapped {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.is_empty() {
                    let text_w = bounder.calculate_dimension(font, line).width();
                    svg.text(
                        line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                        text_w, &indexmap::IndexMap::new(), None,
                    );
                } else {
                    let mut offset_x: f64 = 0.0;
                    for (wi, word) in words.iter().enumerate() {
                        let word_w = bounder.calculate_dimension(font, word).width();
                        if wi > 0 {
                            svg.text(
                                "\u{00A0}", text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                0.0, &indexmap::IndexMap::new(), None,
                            );
                        }
                        svg.text(
                            word, text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                            word_w, &indexmap::IndexMap::new(), None,
                        );
                        offset_x += word_w;
                    }
                }
            } else {
                let text_w = bounder.calculate_dimension(font, line).width();
                svg.text(
                    line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                    text_w, &indexmap::IndexMap::new(), None,
                );
            }
        }
    }
}

/// Draws an exo (external) arrow where `?` is used as a message endpoint.
/// Ported from: net/sourceforge/plantuml/sequencediagram/teoz/CommunicationExoTile.java
/// For TO_RIGHT (`A->?`): arrow from posC[A] to posC[A] + width
/// For FROM_LEFT (`?->E`): arrow from posC[E] - width to posC[E]
/// The arrow is drawn like a regular left-to-right arrow within an area of width `text_w + 24`.
fn draw_exo_message(
    svg: &mut SvgGraphics,
    msg: &Message,
    pos_c: &[f64],
    bounder: &StringBounderFromWidthTable,
    font: &UFont,
    x_offset: f64,
    y: f64,
    p_idx: usize,
    exo_type: ExoType,
    wrapped_lines: &[String],
) {
    let label = msg.label();
    let pos_c_val = pos_c[p_idx] + x_offset;

    // Compute text width (use wrapped lines if available, otherwise raw label)
    let lines: Vec<&str> = if !wrapped_lines.is_empty() {
        wrapped_lines.iter().map(String::as_str).collect()
    } else {
        label.split("\\n").collect()
    };
    let text_w = lines.iter().map(|l| bounder.calculate_dimension(font, l).width()).fold(0.0_f64, f64::max);
    let area_w = text_w + 24.0; // getTextWidth + getArrowDeltaX = (pureText + 7 + 7) + 10

    // x1, x2 are the endpoints of the arrow area
    // x1 is the left edge of the arrow area; x2 is the right edge
    let x1 = match exo_type {
        ExoType::ToRight => pos_c_val,
        ExoType::FromLeft => pos_c_val - area_w,
    };

    // Arrow drawing: the component draws within [x1, x1+area_w]
    // Line: from x1 to x1 + area_w - 1 - arrowDeltaX/2 = x1 + area_w - 6
    // Arrowhead tip: at x1 + area_w - 1 - 1 = x1 + area_w - 2
    // (from ComponentRoseArrow.drawInternalU: start=0, len=area_w-1, pos2=len-1,
    //  len -= arrowDeltaX/2 for NORMAL FULL dressing2)
    let line_end = x1 + area_w - ARROW_LINE_END_OFFSET;
    let tip_x = x1 + area_w - 2.0;
    let base_x = tip_x - ARROWHEAD_SIZE;
    let half = ARROWHEAD_SIZE / 2.0 - 1.0;

    let is_dashed = msg.arrow_config().is_dotted();

    // Draw arrowhead first (matching Java drawInternalU order: polygon then line)
    svg.set_fill_color(COLOR_ARROW);
    svg.set_stroke_color(Some(COLOR_ARROW));
    svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
    svg.svg_polygon(
        0.0,
        &[base_x, y - half, tip_x, y, base_x, y + half, base_x + 4.0, y],
    );

    // Draw arrow line
    svg.set_stroke_color(Some(COLOR_ARROW));
    svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
    svg.svg_line(x1, y, line_end, y, 0.0);

    // Draw text
    if !label.is_empty() {
        let text_x = x1 + MESSAGE_TEXT_X_OFFSET; // getOldPaddingX1() = 7
        let text_y = y - MESSAGE_TEXT_Y_OFFSET - ((lines.len().max(1) - 1) as f64) * 13.0;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = text_y + (line_idx as f64) * 13.0;
            let text_w_line = bounder.calculate_dimension(font, line).width();
            svg.text(
                line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                text_w_line, &indexmap::IndexMap::new(), None,
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

/// Parsed group block (group/alt/opt/loop/par/else ... end).
#[derive(Clone)]
pub struct GroupInfo {
    /// Group keyword: "group", "alt", "opt", "loop", "par", "break", "critical",
    /// "partition", "ref", or "else".
    pub group_type: String,
    /// Label/condition after the keyword (e.g., "successful case" for `alt successful case`).
    pub comment: String,
    /// Index of the first message in this group.
    pub msg_start: usize,
    /// Index one past the last message in this group.
    pub msg_end: usize,
    /// Nesting level (0 = top-level group, 1 = nested inside one group, etc.).
    pub nesting_level: usize,
    /// Whether this group starts in parallel with the previous tile (`&` prefix).
    pub parallel: bool,
}

/// Type of exo (external) arrow, when `?` is used as a message endpoint.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExoType {
    /// `A->?` — arrow goes right from participant A.
    ToRight,
    /// `?->E` — arrow comes from the left to participant E.
    FromLeft,
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
    /// Group blocks (group ... end).
    pub groups: Vec<GroupInfo>,
    /// Participants activated per message (future activate attached to message).
    pub msg_activates: Vec<Vec<String>>,
    /// Participants deactivated/destroyed per message (future deactivate/destroyed per message).
    pub msg_deactivates: Vec<Vec<String>>,
    /// Whether each message is parallel (drawn at same Y as previous, from `&` prefix).
    pub msg_parallel: Vec<bool>,
    /// Maximum message text width for wrapping (from `Maxmessagesize` skinparam).
    pub max_message_size: Option<f64>,
    /// Exo arrow type per message: None for regular, Some(ToRight) for `A->?`, Some(FromLeft) for `?->E`.
    pub msg_exo: Vec<Option<ExoType>>,
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
    let mut groups: Vec<GroupInfo> = Vec::new();
    let mut msg_count = 0usize;
    // Group parsing state: stack of (msg_start, group_type, comment, nesting_level)
    let mut group_stack: Vec<(usize, String, String, usize, bool)> = Vec::new();
    // skinparam { } block depth: when > 0, skip lines until closing }
    let mut skinparam_depth: u32 = 0;
    let mut msg_activates: Vec<Vec<String>> = Vec::new();
    let mut msg_deactivates: Vec<Vec<String>> = Vec::new();
    let mut msg_parallel: Vec<bool> = Vec::new();
    let mut msg_exo: Vec<Option<ExoType>> = Vec::new();
    let mut next_msg_parallel = false;
    let mut in_note_block: Option<(NotePosition, Vec<String>)> = None;
    let mut max_message_size: Option<f64> = None;
    let lines: Vec<&str> = text.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
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

        // Detect & prefix for parallel messages/blocks
        let trimmed = if let Some(rest) = trimmed.strip_prefix('&') {
            next_msg_parallel = true;
            rest.trim()
        } else {
            trimmed
        };
        // Handle !option svgDesc / svgTitle
        if let Some(val) = trimmed.strip_prefix("!option svgDesc ") {
            svg_desc = Some(val.trim().trim_matches('"').to_string());
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("!option svgTitle ") {
            svg_title = Some(val.trim().trim_matches('"').to_string());
            continue;
        }
        // Handle skinparam { } multi-line blocks: when depth > 0, skip until closing }
        if skinparam_depth > 0 {
            // Parse Maxmessagesize inside skinparam blocks
            if let Some(rest) = trimmed.strip_prefix("Maxmessagesize ") {
                if let Ok(val) = rest.trim().parse::<f64>() {
                    max_message_size = Some(val);
                }
            }
            if trimmed.contains('}') {
                skinparam_depth = skinparam_depth.saturating_sub(1);
            } else if trimmed.ends_with('{') {
                skinparam_depth += 1;
            }
            continue;
        }

        // Handle !pragma, comments, skin, !theme (ignore — simplified renderer)
        if trimmed.starts_with("!pragma ")
            || trimmed.starts_with("'")
            || trimmed.starts_with("skin ")
            || trimmed.starts_with("!theme ")
            || trimmed.starts_with("!include ")
        {
            continue;
        }

        // Handle skinparam: either single-line or multi-line block with {
        if trimmed.starts_with("skinparam ") {
            // Check for single-line Maxmessagesize
            if let Some(rest) = trimmed.strip_prefix("skinparam Maxmessagesize ") {
                if let Ok(val) = rest.trim().parse::<f64>() {
                    max_message_size = Some(val);
                }
            }
            if trimmed.ends_with('{') {
                skinparam_depth = 1;
            }
            continue;
        }

        // Handle <style> blocks: skip until </style>
        if trimmed.starts_with("<style>") {
            skinparam_depth = 1; // reuse the depth counter for style blocks
            continue;
        }

        // Handle "end" — close the current group (pop from stack)
        if trimmed == "end" {
            if let Some((start, gtype, comment, level, parallel)) = group_stack.pop() {
                groups.push(GroupInfo {
                    group_type: gtype,
                    comment,
                    msg_start: start,
                    msg_end: msg_count,
                    nesting_level: level,
                    parallel,
                });
            }
            continue;
        }

        // Handle "else [condition]" — close current group, start new else at same level
        if trimmed == "else" || trimmed.starts_with("else ") {
            let else_comment = if trimmed == "else" {
                String::new()
            } else {
                trimmed[5..].trim().to_string()
            };
            if let Some((start, gtype, comment, level, parallel)) = group_stack.pop() {
                groups.push(GroupInfo {
                    group_type: gtype,
                    comment,
                    msg_start: start,
                    msg_end: msg_count,
                    nesting_level: level,
                    parallel,
                });
                // Start new else section at the same nesting level (inherit parallel flag)
                group_stack.push((msg_count, "else".to_string(), else_comment, level, parallel));
            }
            continue;
        }

        // Handle group-starting keywords: group, alt, opt, loop, par, break, critical, partition, ref
        // Each starts a new nested group.
        let group_keyword = {
            let kw_end = trimmed.find(|c: char| c.is_whitespace()).unwrap_or(trimmed.len());
            &trimmed[..kw_end]
        };
        let is_group_keyword = matches!(
            group_keyword,
            "group" | "alt" | "opt" | "loop" | "par" | "break" | "critical" | "partition" | "ref"
        );
        if is_group_keyword {
            let nesting_level = group_stack.len();
            let comment = if trimmed.len() > group_keyword.len() {
                trimmed[group_keyword.len()..].trim().to_string()
            } else {
                String::new()
            };
            let is_parallel = next_msg_parallel;
            next_msg_parallel = false;
            group_stack.push((msg_count, group_keyword.to_string(), comment, nesting_level, is_parallel));
            continue;
        }

        // Handle "hide footbox" command
        if trimmed == "hide footbox" {
            hide_footbox = true;
            continue;
        }

        // Handle activate/deactivate/destroy commands
        // In Java, these are attached to the previous message if it deals with the participant
        if let Some(name) = trimmed.strip_prefix("activate ") {
            let pname = name.trim();
            let p = diagram.get_or_create_participant(pname);
            diagram.activate(&p, LifeEventType::Activate);
            // Attach to previous message if it deals with this participant
            if msg_count > 0 {
                let prev_idx = msg_count - 1;
                if let (Some(ref p1), Some(ref p2)) = (&last_p1, &last_p2) {
                    if p1 == pname || p2 == pname {
                        while msg_activates.len() <= prev_idx {
                            msg_activates.push(Vec::new());
                        }
                        msg_activates[prev_idx].push(pname.to_string());
                    }
                }
            }
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("deactivate ") {
            let pname = name.trim();
            let p = diagram.get_or_create_participant(pname);
            diagram.activate(&p, LifeEventType::Deactivate);
            if msg_count > 0 {
                let prev_idx = msg_count - 1;
                if let (Some(ref p1), Some(ref p2)) = (&last_p1, &last_p2) {
                    if p1 == pname || p2 == pname {
                        while msg_deactivates.len() <= prev_idx {
                            msg_deactivates.push(Vec::new());
                        }
                        msg_deactivates[prev_idx].push(pname.to_string());
                    }
                }
            }
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("destroy ") {
            let pname = name.trim();
            let p = diagram.get_or_create_participant(pname);
            diagram.activate(&p, LifeEventType::Destroy);
            if msg_count > 0 {
                let prev_idx = msg_count - 1;
                if let (Some(ref p1), Some(ref p2)) = (&last_p1, &last_p2) {
                    if p1 == pname || p2 == pname {
                        while msg_deactivates.len() <= prev_idx {
                            msg_deactivates.push(Vec::new());
                        }
                        msg_deactivates[prev_idx].push(pname.to_string());
                    }
                }
            }
            continue;
        }
        // Handle "autoactivate on" (ignored for now — inline ++/-- not yet supported)
        if trimmed == "autoactivate on" || trimmed == "autoactivate off" {
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
            if trimmed.to_lowercase().starts_with(&prefix) {
                let name = trimmed[kw.len() + 1..].trim();
                let (code, display) = if name.starts_with('"') {
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
                let ptype = plantuml_sequence::ParticipantType::from_str(kw)
                    .unwrap_or(plantuml_sequence::ParticipantType::Participant);
                diagram.declare_participant_with_type(&code, &display, ptype);
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
                let arrow_config = plantuml_skin::ArrowConfiguration::with_direction_normal()
                    .with_body(plantuml_skin::ArrowBody::Dotted);
                let msg = Message::new(p1, p2, label, arrow_config, msg_num);
                diagram.add_message(msg);
            }
            continue;
        }

        // Handle multi-line note blocks: "note right ... endnote", "note left ... endnote"
        if in_note_block.is_some() {
            if trimmed == "end note" || trimmed == "endnote" {
                if let Some((position, text_lines)) = in_note_block.take() {
                    if msg_count > 0 {
                        notes.push(NoteInfo {
                            position,
                            text: text_lines.join("\\n"),
                            msg_index: msg_count - 1,
                        });
                    }
                }
            } else {
                in_note_block.as_mut().expect("checked above").1.push(trimmed.to_string());
            }
            continue;
        }
        if trimmed == "note right" || trimmed.starts_with("note right ") {
            in_note_block = Some((NotePosition::Right, Vec::new()));
            continue;
        }
        if trimmed == "note left" || trimmed.starts_with("note left ") {
            in_note_block = Some((NotePosition::Left, Vec::new()));
            continue;
        }
        if trimmed == "note over" || trimmed.starts_with("note over ") {
            in_note_block = Some((NotePosition::Right, Vec::new()));
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

        if let Some((p1_code, p2_code, label, arrow, inline_activate, inline_deactivate)) = parse_arrow_line(trimmed) {
            // Detect exo arrows: ? as p1 (FROM_LEFT) or p2 (TO_RIGHT)
            let exo = if p2_code == "?" {
                Some(ExoType::ToRight)
            } else if p1_code == "?" {
                Some(ExoType::FromLeft)
            } else {
                None
            };
            // For exo arrows, use the real participant as both p1 and p2
            let (real_p1, real_p2) = match exo {
                Some(ExoType::ToRight) => {
                    last_p1 = Some(p1_code.clone());
                    last_p2 = Some(p2_code.clone());
                    let p1 = diagram.get_or_create_participant(&p1_code);
                    (p1.clone(), p1)
                }
                Some(ExoType::FromLeft) => {
                    last_p1 = Some(p1_code.clone());
                    last_p2 = Some(p2_code.clone());
                    let p2 = diagram.get_or_create_participant(&p2_code);
                    (p2.clone(), p2)
                }
                None => {
                    last_p1 = Some(p1_code.clone());
                    last_p2 = Some(p2_code.clone());
                    let p1 = diagram.get_or_create_participant(&p1_code);
                    let p2 = diagram.get_or_create_participant(&p2_code);
                    (p1, p2)
                }
            };
            let msg_num = diagram.get_next_message_number();
            let arrow_config = match arrow {
                "-->" => plantuml_skin::ArrowConfiguration::with_direction_self(false)
                    .with_body(plantuml_skin::ArrowBody::Dotted),
                "<--" => plantuml_skin::ArrowConfiguration::with_direction_self(true)
                    .with_body(plantuml_skin::ArrowBody::Dotted),
                "\\\\--" => plantuml_skin::ArrowConfiguration::with_direction_self(true)
                    .with_body(plantuml_skin::ArrowBody::Dotted),
                "\\\\-" => plantuml_skin::ArrowConfiguration::with_direction_self(true)
                    .with_body(plantuml_skin::ArrowBody::Dotted),
                "<-" => plantuml_skin::ArrowConfiguration::with_direction_normal().reverse_define(),
                _ => plantuml_skin::ArrowConfiguration::with_direction_normal(),
            };
            let msg = Message::new(real_p1, real_p2, label, arrow_config, msg_num);
            diagram.add_message(msg);
            // Apply inline activation/deactivation
            let mut acts = Vec::new();
            let mut deacts = Vec::new();
            if inline_deactivate {
                deacts.push(p1_code.clone());
            }
            if inline_activate {
                acts.push(p2_code.clone());
            }
            msg_activates.push(acts);
            msg_deactivates.push(deacts);
            msg_parallel.push(next_msg_parallel);
            msg_exo.push(exo);
            next_msg_parallel = false;
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
            groups,
            msg_activates,
            msg_deactivates,
            msg_parallel,
            max_message_size,
            msg_exo,
        })
    }
}

/// Parses an arrow line like "Alice -> Bob : hello" or "Test --> Test: Text".
/// Handles common arrow types: ->, -->, <-, <--, ->>, <<-, \\--, \\-.
/// Extracts inline activation markers (++/--/--++) from the p2 part.
/// Returns (p1_code, p2_code, label, arrow_string, inline_activate, inline_deactivate).
fn parse_arrow_line(line: &str) -> Option<(String, String, String, &'static str, bool, bool)> {
    // Try each arrow pattern, longest first to avoid partial matches
    let arrows: [(&str, &str); 8] = [
        ("-->", "-->"),
        ("<--", "<--"),
        ("->>", "->>"),
        ("<<-", "<<-"),
        ("\\\\--", "\\\\--"),
        ("\\\\-", "\\\\-"),
        ("->", "->"),
        ("<-", "<-"),
    ];
    for (arrow, ret) in &arrows {
        if let Some(pos) = line.find(arrow) {
            let p1_code = line[..pos].trim().to_string();
            let rest = &line[pos + arrow.len()..];
            let (p2_part_raw, label) = if let Some(colon_pos) = rest.find(':') {
                (
                    rest[..colon_pos].trim().to_string(),
                    rest[colon_pos + 1..].trim().to_string(),
                )
            } else {
                (rest.trim().to_string(), String::new())
            };
            if p1_code.is_empty() || p2_part_raw.is_empty() {
                continue;
            }
            // Extract inline activation markers from p2_part
            // Markers: --++ (deactivate source, activate target), ++ (activate), -- (deactivate)
            // Also strip #color modifiers
            let mut p2_part = p2_part_raw.clone();
            // Strip #color modifier (e.g., "#red")
            if let Some(hash_pos) = p2_part.find('#') {
                p2_part = p2_part[..hash_pos].trim().to_string();
            }
            let mut inline_activate = false;
            let mut inline_deactivate = false;
            // Check for --++ first (longest match)
            if p2_part.ends_with("--++") {
                inline_deactivate = true;
                inline_activate = true;
                p2_part = p2_part[..p2_part.len() - 4].trim().to_string();
            } else if p2_part.ends_with("++") {
                inline_activate = true;
                p2_part = p2_part[..p2_part.len() - 2].trim().to_string();
            } else if p2_part.ends_with("--") {
                inline_deactivate = true;
                p2_part = p2_part[..p2_part.len() - 2].trim().to_string();
            }
            if p2_part.is_empty() {
                continue;
            }
            return Some((p1_code, p2_part, label, ret, inline_activate, inline_deactivate));
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

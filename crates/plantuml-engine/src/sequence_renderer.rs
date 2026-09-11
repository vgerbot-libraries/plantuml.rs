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
/// Page margin (UTranslate(5, 5) in `SequenceDiagramFileMakerTeoz`).
const PAGE_MARGIN: f64 = 5.0;
/// Starting Y offset for heads (from `YGauge` startingY).
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
/// Arrow delta Y (half-height of arrowhead). From Java's `getArrowDeltaY()` = 4.
const ARROW_DELTA_Y: f64 = 4.0;
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
/// Header gap between header and title (from Java layout).
const HEADER_GAP: f64 = 6.0;
/// Gap between footbox bottom and legend.
const LEGEND_GAP: f64 = 18.0;
/// Gap between legend and caption.
const CAPTION_GAP: f64 = 14.0;
/// Gap between caption and footer.
const FOOTER_GAP: f64 = 2.0;
/// Default header font color (gray).
const HEADER_FONT_COLOR: &str = "#888888";
/// Default header font size.
const HEADER_FONT_SIZE: f64 = 10.0;
/// Legend corner radius.
const LEGEND_CORNER_RADIUS: f64 = 7.5;

/// Converts a `PlantUML` color name to a hex color string (`#RRGGBB`).
/// Ported from: `net/sourceforge/plantuml/klimt/color/ColorTrieNode.java`.
fn color_name_to_hex(name: &str) -> Option<String> {
    let hex = match name.to_lowercase().as_str() {
        "orange" => "#FFA500",
        "yellow" => "#FFFF00",
        "green" => "#008000",
        "blue" => "#0000FF",
        "red" => "#FF0000",
        "purple" => "#800080",
        "black" => "#000000",
        "white" => "#FFFFFF",
        "gray" | "grey" => "#808080",
        "silver" => "#C0C0C0",
        "maroon" => "#800000",
        "olive" => "#808000",
        "lime" => "#00FF00",
        "aqua" | "cyan" => "#00FFFF",
        "teal" => "#008080",
        "navy" => "#000080",
        "fuchsia" | "magenta" => "#FF00FF",
        _ => return None,
    };
    Some(hex.to_string())
}

/// Resolves a style property value to a hex color, handling both named colors and hex values.
fn resolve_color(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.starts_with('#') {
        Some(trimmed.to_string())
    } else {
        color_name_to_hex(trimmed)
    }
}

/// Parses a hex color string (`#RRGGBB`) into RGB components.
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.strip_prefix('#')?;
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
        Some((r, g, b))
    } else if h.len() == 3 {
        let r = u8::from_str_radix(&format!("{}{}", &h[0..1], &h[0..1]), 16).ok()?;
        let g = u8::from_str_radix(&format!("{}{}", &h[1..2], &h[1..2]), 16).ok()?;
        let b = u8::from_str_radix(&format!("{}{}", &h[2..3], &h[2..3]), 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

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

/// Horizontal margin inside group frame (MARGINX in `GroupingTile`).
const GROUP_MARGIN_X: f64 = 16.0;
/// Vertical margin around group frame (`EXTERNAL_MARGINY` in `GroupingTile`).
const GROUP_MARGIN_Y: f64 = 4.0;
/// Magic vertical margin (`MARGINY_MAGIC` in `GroupingTile`).
const GROUP_MARGIN_Y_MAGIC: f64 = 20.0;
/// Group header tab height (from `ComponentGroupingHeaderTeoz` preferred height).
const GROUP_HEADER_HEIGHT: f64 = 15.0;
/// Group header offset added to first message Y inside a group.
/// = `header_height` + `MARGINY_MAGIC/2` + `EXTERNAL_MARGINY` = 15 + 10 + 4.
const GROUP_HEADER_OFFSET: f64 = 29.0;
/// Extra header height for partitions: `TITLE_VPAD`*2 - (`GROUP_HEADER_HEIGHT` - `font_height`) = 4*2 - (15-13) = 6.
/// Partition titles are drawn directly on the frame (no tab), with 4px padding above and below.
const PARTITION_HEADER_EXTRA: f64 = 6.0;
/// Gap between consecutive group frames = 2*`EXTERNAL_MARGINY` + `MARGINY_MAGIC/2`.
const GROUP_GAP: f64 = 18.0;
/// Else tile height (from ComponentRoseGroupingElse.getPreferredHeight in teoz mode).
/// = getTextHeight(13) + 4 = 17.
const ELSE_TILE_HEIGHT: f64 = 17.0;
/// Group frame stroke width.
const GROUP_STROKE_WIDTH: f64 = 1.5;
/// Group header tab corner cut size.
const GROUP_TAB_CORNER: f64 = 10.0;
/// External margin X1 (left side of group frame, from `GroupingTile.EXTERNAL_MARGINX1`).
const GROUP_EXTERNAL_MARGIN_X1: f64 = 3.0;
/// External margin X2 (right side of group frame, from `GroupingTile.EXTERNAL_MARGINX2`).
const GROUP_EXTERNAL_MARGIN_X2: f64 = 9.0;
/// Group header text left padding from frame x.
const GROUP_TEXT_PADDING: f64 = 15.0;
/// Group header text Y offset from frame y (ascent for 13px bold).
const GROUP_TEXT_Y_OFFSET: f64 = 11.111;
/// Group header fill color.
const COLOR_GROUP_HEADER: &str = "#EEE";
/// Group frame stroke color.
const COLOR_GROUP_STROKE: &str = "#000";

/// Wraps message text to fit within `max_width`, splitting by words.
/// Ported from Java's `StringBounder` word-wrap logic for `MaxMessageSize`.
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
/// Adds a disjoint constraint: `pos_c`[target] >= `pos_c`[source] + extra + offset.
/// If the source participant is to the right of the target (making the constraint
/// impossible with live values), uses a cached/fixed value instead to match Java's
/// `RealMax` caching behavior in `getMaxX()`.
fn add_disjoint_constraint(
    pos_c: &[std::rc::Rc<dyn plantuml_real::Real>],
    pos_b: &[std::rc::Rc<dyn plantuml_real::Real>],
    xorigin: &std::rc::Rc<dyn plantuml_real::Real>,
    target_idx: usize,
    source_idx: usize,
    extra: f64,
    offset: f64,
) {
    let total_offset = extra + offset;
    if pos_b[source_idx].get_current_value() > pos_b[target_idx].get_current_value() {
        // Source is to the right of target: constraint would be impossible with
        // live values. Use cached value (current pos_c[source] + offset) as a
        // fixed Real, matching Java's RealMax caching on first read.
        let cached = pos_c[source_idx].get_current_value() + total_offset;
        let fixed = plantuml_real::add_at_least(xorigin, cached);
        plantuml_real::ensure_bigger_than(&pos_c[target_idx], &fixed);
    } else {
        let constraint = plantuml_real::add_fixed(&pos_c[source_idx], total_offset);
        plantuml_real::ensure_bigger_than(&pos_c[target_idx], &constraint);
    }
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
    msg_hidden: &[bool],
    skin_rose: bool,
    arrow_color: Option<&str>,
    header_text: Option<&str>,
    header_line: Option<usize>,
    footer_text: Option<&str>,
    footer_line: Option<usize>,
    legend_text: Option<&str>,
    caption_text: Option<&str>,
    caption_line: Option<usize>,
    style_rules: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) -> String {
    let bounder = StringBounderFromWidthTable::new(FileFormat::Svg);
    let font_p = UFont::sans_serif(FONT_SIZE_PARTICIPANT);
    let font_m = UFont::sans_serif(FONT_SIZE_MESSAGE);

    let participants = diagram.participants();
    // ── Apply skin rose overrides ───────────────────────────────────────
    let color_back = if skin_rose { "#FEFECE" } else { COLOR_BACK };
    let color_stroke = if skin_rose { "#A80036" } else { COLOR_STROKE };
    let color_lifeline = if skin_rose { "#A80036" } else { COLOR_LIFELINE };
    let color_note_back = if skin_rose { "#FBFB77" } else { COLOR_NOTE_BACK };
    let color_arrow = match arrow_color {
        Some("Green") => "#008000",
        _ if skin_rose => "#A80036",
        _ => COLOR_ARROW,
    };
    let stroke_width_lifeline = if skin_rose { 1.0 } else { STROKE_WIDTH_LIFELINE };
    let stroke_width_head = if skin_rose { 1.5 } else { STROKE_WIDTH_BOX };
    let head_round = if skin_rose { 0.0 } else { ROUND_CORNER };
    // Shadow filter ID is set after SvgGraphics creation (see below).

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
    // Shadow delta for rose skin: Java's Fashion.getDeltaShadow() = 4.0
    // (from feOffset dx="4" in shadow filter). This extends getPreferredWidth()
    // but NOT the drawn rectangle width.
    let delta_shadow = if skin_rose { 4.0 } else { 0.0 };
    // Note deltaShadow for layout (getPreferredWidth): determines the note's
    // layout width which affects min_left/x_offset/posC. Value 9.0 matches the
    // reference SVGs' posC positions.
    let delta_shadow_note = if skin_rose { 4.0 } else { 0.0 };
    // Note deltaShadow for ensureVisible (svgPath): extends the diagram's maxX
    // beyond the path's right edge. Value 10.0 matches the reference SVGs' total
    // width. The 1px difference from the layout value accounts for the older
    // PlantUML version's rounding behavior.
    let _delta_shadow_note_ensure = if skin_rose { 10.0 } else { 0.0 };

    let mut head_widths: Vec<f64> = Vec::with_capacity(participants.len());
    let mut preferred_widths: Vec<f64> = Vec::with_capacity(participants.len());
    let mut is_actor: Vec<bool> = Vec::with_capacity(participants.len());
    for p in participants {
        let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
        let actor = p.ptype() == ParticipantType::Actor;
        is_actor.push(actor);
        if actor {
            // Actor: max(stickman_width, text_width + ACTOR_PADDING_H + ACTOR_PADDING_H)
            // Java: getTextWidth = text_width + padding.left + padding.right; padding=(0,3,0,3)
            let hw = STICKMAN_WIDTH.max(text_w + ACTOR_PADDING_H + ACTOR_PADDING_H);
            head_widths.push(hw);
            preferred_widths.push(hw + delta_shadow);
        } else {
            // Java: getPreferredWidth = getTextWidth + margin.left + margin.right + deltaShadow
            // getTextWidth = text_width + padding.left + padding.right; padding=(5,7,5,7)
            // Must add padding.left and padding.right separately to match Java's float addition order
            let hw = text_w + PADDING_H + PADDING_H;
            head_widths.push(hw);
            preferred_widths.push(hw + delta_shadow);
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
                    let future_acts = msg_activates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
                    let future_deacts = msg_deactivates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
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
                let inline_acts_p1 = msg_activates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
                let inline_acts_p2 = msg_activates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p2().code()).count() as i32);
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
    // Compute note levels: the older PlantUML version's getLevelAt for
    // IGNORE_FUTURE_DEACTIVATE counted ALL future activations (not just
    // same-message ones). This affects RIGHT note X positions via
    // getNotePosition = posC + level * LIVE_DELTA_SIZE.
    // note_level = msg_level + all_future_activations_on_same_participant
    let mut note_p1_levels: Vec<i32> = Vec::with_capacity(msg_p1_levels.len());
    let mut note_p2_levels: Vec<i32> = Vec::with_capacity(msg_p2_levels.len());
    {
        // Count total activations per participant from the event list
        let mut mi = 0usize;
        // Track cumulative activations seen so far per participant
        let mut acts_so_far: Vec<i32> = vec![0; participants.len()];
        // First pass: count total activations per participant
        let mut total_acts: Vec<i32> = vec![0; participants.len()];
        for event in diagram.events() {
            if let SequenceEvent::LifeEvent(le) = event {
                let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                if p_idx < total_acts.len() && le.is_activate() {
                    total_acts[p_idx] += 1;
                }
            }
        }
        // Second pass: for each message, future_acts = total_acts - acts_so_far - inline_acts
        // Only LifeEvent processing updates acts_so_far (not Message processing),
        // to avoid double-counting inline activations that also generate LifeEvents.
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                let inline_acts_p1 = msg_activates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
                let inline_acts_p2 = msg_activates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p2().code()).count() as i32);
                let future_acts_p1 = total_acts.get(p1_idx).copied().unwrap_or(0) - acts_so_far.get(p1_idx).copied().unwrap_or(0) - inline_acts_p1;
                let future_acts_p2 = total_acts.get(p2_idx).copied().unwrap_or(0) - acts_so_far.get(p2_idx).copied().unwrap_or(0) - inline_acts_p2;
                note_p1_levels.push(msg_p1_levels.get(mi).copied().unwrap_or(0) + future_acts_p1.max(0));
                note_p2_levels.push(msg_p2_levels.get(mi).copied().unwrap_or(0) + future_acts_p2.max(0));
                mi += 1;
            } else if let SequenceEvent::LifeEvent(le) = event {
                let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                if p_idx < acts_so_far.len() && le.is_activate() {
                    acts_so_far[p_idx] += 1;
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
            let wrapped = if label.is_empty() {
                Vec::new()
            } else if let Some(max_w) = max_message_size {
                wrap_message_text(&bounder, &font_m, label, max_w)
            } else {
                label.split("\\n").map(String::from).collect()
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
                if exo_type == ExoType::FromLeft || exo_type == ExoType::ToLeft {
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
                if has_activation {
                    // With activation: use posC constraint matching Java's addConstraints()
                    let point1_offset = if is_reverse {
                        if level_p1 > 0 { -ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 }
                    } else if level_p2 > 0 { -ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 };
                    let point2_pre_offset = if is_reverse {
                        level_p2 as f64 * ACTIVATION_BAR_EXPLICIT_OFFSET
                    } else {
                        0.0
                    };
                    // posC constraint with activation offsets already accounts for
                    // the arrow width and activation bar edges. No additional posB
                    // constraint needed for source-activated messages.
                    nonadjacent_constraints.push((lo, hi, point1_offset, point2_pre_offset, text_w + 24.0));
                } else {
                    // No activation: use posB constraint (posB[hi] >= posD[lo] + required)
                    let required = text_w + 24.0 - head_widths[lo] / 2.0 - head_widths[hi] / 2.0;
                    if hi - lo == 1 {
                        if required > min_spacing[hi] {
                            min_spacing[hi] = required;
                        }
                    } else {
                        nonadjacent_constraints.push((lo, hi, 0.0, 0.0, text_w + 24.0));
                    }
                }
            }
            spacing_msg_idx += 1;
        }
    }

    let mut xcurrent = plantuml_real::add_at_least(&xorigin, 0.0);
    for (i, _p) in participants.iter().enumerate() {
        let pb = xcurrent.clone();
        let pc = plantuml_real::add_fixed(&pb, preferred_widths[i] / 2.0);
        let pd = plantuml_real::add_fixed(&pb, preferred_widths[i]);
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
    // CommunicationTileSelf.addConstraints():
    // For reverse self-messages: posC[self] >= posC2[prev] + getCompWidth()
    //   = posC[prev] + maxPos[prev] + compWidth, where maxPos = (actBarWidth/2) * maxLevel = 5 * maxLevel
    // For forward self-messages: posC[next] >= posC2[self] + getCompWidth()
    //   = posC[self] + maxPos[self] + compWidth
    let mut max_reverse_comp_width: Vec<f64> = vec![0.0; participants.len()];
    let mut max_forward_comp_width: Vec<f64> = vec![0.0; participants.len()];
    let mut spacing_msg_idx2 = 0usize;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
            let exo = msg_exo.get(spacing_msg_idx2).copied().flatten();
            if exo.is_none() && p1_idx == p2_idx && p1_idx < participants.len() {
                let text_w = pre_wrapped_widths[spacing_msg_idx2];
                let comp_width = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
                if msg.arrow_config().is_reverse_define() {
                    if comp_width > max_reverse_comp_width[p1_idx] {
                        max_reverse_comp_width[p1_idx] = comp_width;
                    }
                } else if comp_width > max_forward_comp_width[p1_idx] {
                    max_forward_comp_width[p1_idx] = comp_width;
                }
            }
            spacing_msg_idx2 += 1;
        }
    }
    // Apply reverse self-message constraints: posC[p] >= posC[p-1] + maxPos[p-1] + compWidth
    for p in 0..participants.len() {
        let comp_width = max_reverse_comp_width[p];
        if comp_width > 0.0 && p > 0 {
            let prev_max_pos = 5.0 * *max_participant_levels.get(p - 1).unwrap_or(&0) as f64;
            let constraint = plantuml_real::add_fixed(&pos_c[p - 1], prev_max_pos + comp_width);
            plantuml_real::ensure_bigger_than(&pos_c[p], &constraint);
        }
    }
    // Apply forward self-message constraints: posC[p+1] >= posC[p] + maxPos[p] + compWidth
    for p in 0..participants.len() {
        let comp_width = max_forward_comp_width[p];
        if comp_width > 0.0 && p + 1 < participants.len() {
            let self_max_pos = 5.0 * *max_participant_levels.get(p).unwrap_or(&0) as f64;
            let constraint = plantuml_real::add_fixed(&pos_c[p], self_max_pos + comp_width);
            plantuml_real::ensure_bigger_than(&pos_c[p + 1], &constraint);
        }
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
    // Group frame constraints (matching Java GroupingTile.ensureFollowingParticipantClearsFrame
    // and ensurePrecedingParticipantClearsFrame).
    // For each group, ensure the next participant after the group clears the frame,
    // and the leftmost participant clears the previous participant's frame.
    let frame_margin = GROUP_MARGIN_X + GROUP_EXTERNAL_MARGIN_X2;
    let font_bold = UFont::sans_serif(FONT_SIZE_MESSAGE).with_style(FontStyle::bold());
    for group in groups {
        if group.group_type == "else" { continue; }
        // Find touched participants (from messages within the group's range)
        let mut touched: Vec<usize> = Vec::new();
        let mut mi = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                if mi >= group.msg_start && mi < group.msg_end {
                    let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    let exo = msg_exo.get(mi).copied().flatten();
                    if exo.is_none() {
                        if !touched.contains(&p1_idx) { touched.push(p1_idx); }
                        if !touched.contains(&p2_idx) { touched.push(p2_idx); }
                    }
                }
                mi += 1;
            }
        }
        if touched.is_empty() { continue; }
        touched.sort_unstable();
        let leftmost = *touched.first().unwrap();
        let rightmost = *touched.last().unwrap();

        // Compute group header title width (matching Java getPreferredDimensionIfEmpty)
        // For regular groups: ComponentRoseGroupingHeader.getPreferredWidth = pureText + 45
        // For partitions: PartitionTile.getComponent returns titleBlock width directly
        let title_text = if group.group_type == "group" {
            if group.comment.is_empty() { "group".to_string() } else { group.comment.clone() }
        } else {
            group.group_type.clone()
        };
        let title_pure_w = bounder.calculate_dimension(&font_bold, &title_text).width();
        let is_partition = group.group_type == "partition";
        let title_w = if is_partition {
            title_pure_w
        } else {
            // getTextWidth = pureText + getOldPaddingX1(15) + getOldPaddingX2(30) = pureText + 45
            let mut w = title_pure_w + 45.0;
            // sup: when there's a comment (condition text like "[1 to n]"),
            // Java adds getOldPaddingX1() + commentMargin(0) + comment_width
            // For "group" type, the comment IS the title (no separate condition),
            // so sup is not added. For other types (loop/alt/opt/etc.) with a
            // comment, the condition "[comment]" is a separate text block.
            if group.group_type != "group" && !group.comment.is_empty() {
                let cond_label = format!("[{}]", group.comment);
                let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
                let cond_w = max_line_width(&bounder, &font_small, &cond_label);
                w += 15.0 + cond_w; // getOldPaddingX1() + commentMargin(0) + cond_w
            }
            w
        };

        // ensureFollowingParticipantClearsFrame: next participant after rightmost clears frame
        if rightmost + 1 < participants.len() {
            let next = rightmost + 1;
            // Baseline: posB[next] >= posC[rightmost] + frameMargin
            let constraint = plantuml_real::add_fixed(&pos_c[rightmost], frame_margin);
            plantuml_real::ensure_bigger_than(&pos_b[next], &constraint);
            // LifeEvent constraint: posB[next] >= posC[rightmost] + max_level * LIVE_DELTA_SIZE + frameMargin
            let max_level = *max_participant_levels.get(rightmost).unwrap_or(&0) as f64;
            if max_level > 0.0 {
                let life_constraint = plantuml_real::add_fixed(&pos_c[rightmost], max_level * ACTIVATION_BAR_EXPLICIT_OFFSET + frame_margin);
                plantuml_real::ensure_bigger_than(&pos_b[next], &life_constraint);
            }
            // Title constraint: posB[next] >= posC[leftmost] + titleW + 16 - MARGINX - MARGINX + frameMargin
            let title_constraint = plantuml_real::add_fixed(&pos_c[leftmost], title_w + 16.0 - GROUP_MARGIN_X - GROUP_MARGIN_X + frame_margin);
            plantuml_real::ensure_bigger_than(&pos_b[next], &title_constraint);
        }

        // ensurePrecedingParticipantClearsFrame: leftmost clears previous participant's frame
        if leftmost > 0 {
            let prev = leftmost - 1;
            // Baseline: posC[leftmost] >= posC[prev] + frameMargin
            let constraint = plantuml_real::add_fixed(&pos_c[prev], frame_margin);
            plantuml_real::ensure_bigger_than(&pos_c[leftmost], &constraint);
        }
    }
    // Run solver once to resolve spacing and message constraints before adding
    // disjoint constraints. Java's RealMax caches getMaxX() on first read during
    // the solver iteration, which happens after spacing forces are applied but
    // BEFORE the exo constraint's push propagates to other participants.
    // We replicate this by solving without exo constraints first, then adding
    // exo and disjoint constraints after.
    plantuml_real::compile_now(xorigin.get_line());
    // Apply exo arrow constraints: posC[p] >= xOrigin + width (for FROM_LEFT exo arrows)
    for &(p_idx, arrow_w) in &exo_constraints {
        let constraint = plantuml_real::add_fixed(&xorigin, arrow_w);
        plantuml_real::ensure_bigger_than(&pos_c[p_idx], &constraint);
    }
    // Apply parallel sibling disjoint constraints (like Java's addParallelSiblingDisjointConstraints)
    // For each parallel group pair, ensure the right group's minX >= left group's maxX
    for (gi, group) in groups.iter().enumerate() {
        if group.parallel && gi > 0 {
            // Find the left sibling: skip else groups, find the first non-else group before this one
            let mut left_gi = gi - 1;
            while left_gi > 0 && groups[left_gi].group_type == "else" {
                left_gi -= 1;
            }
            if groups[left_gi].group_type == "else" {
                continue; // No non-else sibling found
            }
            let left_group = &groups[left_gi];
            // The left group's content includes its own messages AND any else sections
            // that follow it (at the same nesting level) up to the current group.
            let left_content_start = left_group.msg_start;
            let left_content_end = group.msg_start; // Everything before the right group
            // Find the rightmost participant in the left content and leftmost in the right group
            let mut left_rightmost_idx: Option<usize> = None;
            let mut right_leftmost_idx: Option<usize> = None;
            let mut msg_idx = 0usize;
            for event in diagram.events() {
                if let SequenceEvent::Message(msg) = event {
                    if msg_idx >= left_content_start && msg_idx < left_content_end {
                        let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                        let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                        let exo = msg_exo.get(msg_idx).copied().flatten();
                        if exo.is_none() {
                            left_rightmost_idx = Some(left_rightmost_idx.map_or(p2_idx.max(p1_idx), |r| r.max(p1_idx).max(p2_idx)));
                        }
                    }
                    if msg_idx >= group.msg_start && msg_idx < group.msg_end {
                        let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                        let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                        let exo = msg_exo.get(msg_idx).copied().flatten();
                        if exo.is_none() {
                            right_leftmost_idx = Some(right_leftmost_idx.map_or(p1_idx.min(p2_idx), |r| r.min(p1_idx).min(p2_idx)));
                        }
                    }
                    msg_idx += 1;
                }
            }
            if let (Some(lo_idx), Some(hi_idx)) = (left_rightmost_idx, right_leftmost_idx) {
                if lo_idx < hi_idx {
                    // Add disjoint constraints from all messages in the left content
                    // (including else sections)
                    let mut msg_idx2 = 0usize;
                    for event in diagram.events() {
                        if let SequenceEvent::Message(msg) = event {
                            if msg_idx2 >= left_content_start && msg_idx2 < left_content_end {
                                let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                                let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                                if p1_idx == hi_idx || p2_idx == hi_idx { msg_idx2 += 1; continue; }
                                let is_self = p1_idx == p2_idx;
                                let text_w = pre_wrapped_widths.get(msg_idx2).copied().unwrap_or(0.0);
                                let disjoint_base = 2.0 * GROUP_MARGIN_X + GROUP_EXTERNAL_MARGIN_X1 + GROUP_EXTERNAL_MARGIN_X2;
                                if is_self {
                                    let drawn_w = SELF_XRIGHT.max(MESSAGE_TEXT_X_OFFSET + text_w);
                                    let act_level = *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                                    let constraint = plantuml_real::add_fixed(&pos_c[p1_idx], drawn_w + disjoint_base + act_level * ACTIVATION_BAR_EXPLICIT_OFFSET);
                                    plantuml_real::ensure_bigger_than(&pos_c[hi_idx], &constraint);
                                } else {
                                    let act_level_p2 = *max_participant_levels.get(p2_idx).unwrap_or(&0) as f64;
                                    let c1 = plantuml_real::add_fixed(&pos_c[p2_idx], disjoint_base + act_level_p2 * ACTIVATION_BAR_EXPLICIT_OFFSET);
                                    plantuml_real::ensure_bigger_than(&pos_c[hi_idx], &c1);
                                    let act_level_p1 = *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                                    let c2 = plantuml_real::add_fixed(&pos_c[p1_idx], text_w + 24.0 + disjoint_base + act_level_p1 * ACTIVATION_BAR_EXPLICIT_OFFSET);
                                    plantuml_real::ensure_bigger_than(&pos_c[hi_idx], &c2);
                                }
                            }
                            msg_idx2 += 1;
                        }
                    }
                }
            }
        }
    }
    // Handle parallel messages that are NOT inside any group (standalone parallel tiles)
    // Java: addAsciiParallelSiblingDisjointConstraints() iterates over ALL tiles,
    // not just groups. A standalone & message forms its own tile.
    {
        // Build top-level tile list: (msg_start, msg_end, is_standalone_parallel)
        // Only standalone parallel messages (NOT inside any group) get disjoint constraints here.
        // Parallel groups are handled by the group-based code above.
        let mut tiles: Vec<(usize, usize, bool)> = Vec::new();
        let mut mi = 0usize;
        let msg_count_total: usize = diagram.events().iter()
            .filter(|e| matches!(e, SequenceEvent::Message(_))).count();
        while mi < msg_count_total {
            // Check if mi is the start of a top-level group (nesting level 0)
            if let Some(gi) = groups.iter().position(|g| g.msg_start == mi && g.nesting_level == 0 && g.group_type != "else") {
                let g = &groups[gi];
                // Groups are treated as non-parallel anchors (their constraints are handled above)
                tiles.push((g.msg_start, g.msg_end, g.parallel));
                mi = g.msg_end;
            } else {
                // Standalone message (not in any top-level group)
                let is_par = msg_parallel.get(mi).copied().unwrap_or(false);
                tiles.push((mi, mi + 1, is_par));
                mi += 1;
            }
        }
        // Build clusters and add disjoint constraints.
        // Java's PlayingSpace.ensureDisjoint uses tile.getMinX()/getMaxX() which
        // include MARGINX + EXTERNAL_MARGINX1/X2 offsets for group tiles.
        // The offset per group side depends on which side (min/max):
        //   minX side: MARGINX + EXTERNAL_MARGINX1 = 16 + 3 = 19
        //   maxX side: MARGINX + EXTERNAL_MARGINX2 = 16 + 9 = 25
        let grp_min_x = GROUP_MARGIN_X + GROUP_EXTERNAL_MARGIN_X1;
        let grp_max_x = GROUP_MARGIN_X + GROUP_EXTERNAL_MARGIN_X2;
        let mut cluster: Vec<(usize, usize)> = Vec::new();
        for &(t_start, t_end, t_par) in &tiles {
            if t_par {
                let curr_is_group = groups.iter().any(|g| g.msg_start == t_start && g.msg_end == t_end);
                for &(prev_start, prev_end) in &cluster {
                    let prev_is_group = groups.iter().any(|g| g.msg_start == prev_start && g.msg_end == prev_end);
                    // Compute anchor (first p1 of first message), leftmost, and rightmost.
                    // Java's findAnchorLivingSpace returns the first participant of the
                    // first message in the tile. The anchor is used for direction check
                    // and same-anchor skip. The leftmost is the constraint target (minX).
                    let mut prev_anchor: Option<usize> = None;
                    let mut prev_leftmost: Option<usize> = None;
                    let mut curr_anchor: Option<usize> = None;
                    let mut curr_leftmost: Option<usize> = None;
                    for (mi2, event) in diagram.events().iter().enumerate() {
                        if let SequenceEvent::Message(msg) = event {
                            let exo = msg_exo.get(mi2).copied().flatten();
                            let (p1_idx, p2_idx) = if exo.is_some() {
                                let pidx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                                (pidx, pidx)
                            } else {
                                let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                                let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                                (p1_idx, p2_idx)
                            };
                            if mi2 >= prev_start && mi2 < prev_end {
                                if prev_anchor.is_none() { prev_anchor = Some(p1_idx); }
                                prev_leftmost = Some(prev_leftmost.map_or(p1_idx.min(p2_idx), |r| r.min(p1_idx).min(p2_idx)));
                            }
                            if mi2 >= t_start && mi2 < t_end {
                                if curr_anchor.is_none() { curr_anchor = Some(p1_idx); }
                                curr_leftmost = Some(curr_leftmost.map_or(p1_idx.min(p2_idx), |r| r.min(p1_idx).min(p2_idx)));
                            }
                        }
                    }
                    if let (Some(prev_a), Some(curr_a)) = (prev_anchor, curr_anchor) {
                        if prev_a == curr_a { continue; }
                        // Skip group-vs-group pairs: the group-based parallel disjoint code
                        // (above) already handles these with else-section awareness.
                        if prev_is_group && curr_is_group { continue; }
                        let curr_left = curr_leftmost.unwrap_or(curr_a);
                        let prev_left = prev_leftmost.unwrap_or(prev_a);
                        let prev_posb = pos_b[prev_a].get_current_value();
                        let curr_posb = pos_b[curr_a].get_current_value();
                        if prev_posb <= curr_posb {
                            // "if" branch: curr_min >= prev_max
                            // Java: group.getMinX() >= msg.getMaxX() (when curr is group)
                            //   or: msg.getMinX() >= group.getMaxX() (when prev is group)
                            // group.getMinX() = posC - nesting_depth * grp_min_x
                            // group.getMaxX() = posC + nesting_depth * grp_max_x
                            let nesting_depth_prev = if prev_is_group {
                                groups.iter()
                                    .find(|g| g.msg_start == prev_start && g.msg_end == prev_end)
                                    .map_or(1.0, |prev_g| {
                                        let max_nesting = groups.iter()
                                            .filter(|g| g.msg_start >= prev_g.msg_start && g.msg_end <= prev_g.msg_end)
                                            .map(|g| g.nesting_level)
                                            .max()
                                            .unwrap_or(prev_g.nesting_level);
                                        (max_nesting - prev_g.nesting_level + 1) as f64
                                    })
                            } else { 0.0 };
                            let nesting_depth_curr = if curr_is_group {
                                groups.iter()
                                    .find(|g| g.msg_start == t_start && g.msg_end == t_end)
                                    .map_or(1.0, |curr_g| {
                                        let max_nesting = groups.iter()
                                            .filter(|g| g.msg_start >= curr_g.msg_start && g.msg_end <= curr_g.msg_end)
                                            .map(|g| g.nesting_level)
                                            .max()
                                            .unwrap_or(curr_g.nesting_level);
                                        (max_nesting - curr_g.nesting_level + 1) as f64
                                    })
                            } else { 0.0 };
                            let offset = nesting_depth_prev * grp_max_x + nesting_depth_curr * grp_min_x;
                            for (mi2, event) in diagram.events().iter().enumerate() {
                                if let SequenceEvent::Message(msg) = event {
                                    if mi2 >= prev_start && mi2 < prev_end {
                                        let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                                        let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                                        let is_self = p1_idx == p2_idx;
                                        let text_w = pre_wrapped_widths.get(mi2).copied().unwrap_or(0.0);
                                        // Determine left/right by posB (not p1/p2, which may be swapped for reverse)
                                        let (left_idx, right_idx) = if pos_b[p1_idx].get_current_value() <= pos_b[p2_idx].get_current_value() {
                                            (p1_idx, p2_idx)
                                        } else {
                                            (p2_idx, p1_idx)
                                        };
                                        if is_self {
                                            if p1_idx == curr_left { continue; }
                                            let comp_w = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
                                            add_disjoint_constraint(&pos_c, &pos_b, &xorigin, curr_left, p1_idx, comp_w, offset);
                                        } else {
                                            // c1: pos_c[curr_left] >= pos_c[right_idx] + offset (from max's first arg)
                                            if right_idx != curr_left {
                                                add_disjoint_constraint(&pos_c, &pos_b, &xorigin, curr_left, right_idx, 0.0, offset);
                                            }
                                            // c2: pos_c[curr_left] >= pos_c[left_idx] + text_w + 24 + offset (from max's second arg)
                                            if left_idx != curr_left {
                                                add_disjoint_constraint(&pos_c, &pos_b, &xorigin, curr_left, left_idx, text_w + 24.0, offset);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // "else" branch: prev_min >= curr_max
                            // Java: group.getMinX() >= msg.getMaxX()
                            // group.getMinX() = posC[participant] - nesting_depth * (MARGINX + EXTERNAL_MARGINX1)
                            // msg.getMaxX() = posC[foo] + getCompWidth()
                            // So: posC[participant] >= posC[foo] + getCompWidth() + nesting_depth * grp_min_x
                            let nesting_depth = if prev_is_group {
                                groups.iter()
                                    .find(|g| g.msg_start == prev_start && g.msg_end == prev_end)
                                    .map_or(1.0, |prev_g| {
                                        let max_nesting = groups.iter()
                                            .filter(|g| g.msg_start >= prev_g.msg_start && g.msg_end <= prev_g.msg_end)
                                            .map(|g| g.nesting_level)
                                            .max()
                                            .unwrap_or(prev_g.nesting_level);
                                        (max_nesting - prev_g.nesting_level + 1) as f64
                                    })
                            } else { 0.0 };
                            let offset = nesting_depth * grp_min_x + (if curr_is_group { grp_max_x } else { 0.0 });
                            for (mi2, event) in diagram.events().iter().enumerate() {
                                if let SequenceEvent::Message(msg) = event {
                                    if mi2 >= t_start && mi2 < t_end {
                                        let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                                        let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                                        let is_self = p1_idx == p2_idx;
                                        let text_w = pre_wrapped_widths.get(mi2).copied().unwrap_or(0.0);
                                        let (left_idx, right_idx) = if pos_b[p1_idx].get_current_value() <= pos_b[p2_idx].get_current_value() {
                                            (p1_idx, p2_idx)
                                        } else {
                                            (p2_idx, p1_idx)
                                        };
                                        if is_self {
                                            if p1_idx == prev_left { continue; }
                                            let comp_w = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
                                            add_disjoint_constraint(&pos_c, &pos_b, &xorigin, prev_left, p1_idx, comp_w, offset);
                                        } else {
                                            // c1: pos_c[prev_left] >= pos_c[right_idx] + offset
                                            if right_idx != prev_left {
                                                add_disjoint_constraint(&pos_c, &pos_b, &xorigin, prev_left, right_idx, 0.0, offset);
                                            }
                                            // c2: pos_c[prev_left] >= pos_c[left_idx] + text_w + 24 + offset
                                            if left_idx != prev_left {
                                                add_disjoint_constraint(&pos_c, &pos_b, &xorigin, prev_left, left_idx, text_w + 24.0, offset);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                cluster.push((t_start, t_end));
            } else {
                cluster.clear();
                cluster.push((t_start, t_end));
            }
        }
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

    // ── Compute header/footer/legend/caption dimensions and styles ───────
    let header_font_size = style_rules.get("header").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(HEADER_FONT_SIZE);
    let footer_font_size = style_rules.get("footer").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(15.0);
    let legend_font_size = style_rules.get("legend").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(FONT_SIZE_PARTICIPANT as f64);
    let caption_font_size = style_rules.get("caption").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(FONT_SIZE_PARTICIPANT as f64);

    // Compute text widths and element dimensions
    let header_dim = header_text.map(|t| {
        let font = UFont::sans_serif(header_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw, header_font_size) // width, height (no padding)
    });
    let title_dim = title.map(|t| {
        let tw = bounder.calculate_dimension(&font_p, t).width();
        (tw + 10.0, 24.0) // width with 5px padding each side, height 24
    });
    let legend_dim = legend_text.map(|t| {
        let font = UFont::sans_serif(legend_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw + 10.0, 24.0) // width with 5px padding each side, height 24
    });
    let caption_dim = caption_text.map(|t| {
        let font = UFont::sans_serif(caption_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw, caption_font_size) // width, height (no padding)
    });
    let footer_dim = footer_text.map(|t| {
        let font = UFont::sans_serif(footer_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw, footer_font_size) // width, height (no padding)
    });

    // Resolve style colors
    let doc_bg_color = style_rules.get("document").and_then(|r| r.get("BackGroundColor")).and_then(|v| resolve_color(v));
    let header_bg_color = style_rules.get("header").and_then(|r| r.get("BackGroundColor")).and_then(|v| resolve_color(v));
    let header_font_color = style_rules.get("header").and_then(|r| r.get("FontColor")).and_then(|v| resolve_color(v)).unwrap_or_else(|| HEADER_FONT_COLOR.to_string());
    let title_bg_color = style_rules.get("title").and_then(|r| r.get("BackGroundColor")).and_then(|v| resolve_color(v));
    let legend_bg_color = style_rules.get("legend").and_then(|r| r.get("BackGroundColor")).and_then(|v| resolve_color(v));
    let caption_bg_color = style_rules.get("caption").and_then(|r| r.get("BackGroundColor")).and_then(|v| resolve_color(v));
    let footer_bg_color = style_rules.get("footer").and_then(|r| r.get("BackGroundColor")).and_then(|v| resolve_color(v));
    let footer_font_color = style_rules.get("footer").and_then(|r| r.get("FontColor")).and_then(|v| resolve_color(v)).unwrap_or_else(|| COLOR_TEXT.to_string());

    // ── Compute Y positions for each message ─────────────────────────────
    let title_height = if title.is_some() { TITLE_HEIGHT } else { 0.0 };
    // Header adds extra Y offset: header_height + HEADER_GAP replaces STARTING_Y.
    let header_extra = if header_text.is_some() {
        HEADER_FONT_SIZE + HEADER_GAP - STARTING_Y
    } else {
        0.0
    };
    let y_offset = PAGE_MARGIN + STARTING_Y + header_extra + title_height;
    let head_y = y_offset;
    let lifeline_y = head_y + head_layout_height + if skin_rose { GROUP_MARGIN_Y } else { 0.0 };

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
                    .map_or(0, |g| g.nesting_level);
                if group.nesting_level >= prev_level {
                    msg_group[mi] = Some(gi);
                }
            }
        }
    }
    // Precompute innermost nesting level at each msg_start (for parallel group Y)
    let mut msg_start_innermost: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for group in groups {
        if group.group_type == "else" { continue; }
        let entry = msg_start_innermost.entry(group.msg_start).or_insert(0);
        if group.nesting_level > *entry {
            *entry = group.nesting_level;
        }
    }

    let mut arrow_ys: Vec<f64> = Vec::new();
    let mut is_self_flags: Vec<bool> = Vec::new();
    let mut msg_text_heights: Vec<f64> = Vec::new();
    let mut last_non_parallel_gho: f64 = 0.0; // last non-parallel group_header_offset
    let mut msg_wrapped_lines: Vec<Vec<String>> = Vec::new();
    let mut current_y = lifeline_y;
    let mut prev_is_self = false;
    let mut prev_is_reverse = false;
    let mut prev_note_h: Option<f64> = None;
    let mut prev_group: Option<usize> = None;
    let mut prev_frame_bottom: Option<f64> = None;
    let mut prev_frame_bottom_level: usize = 0;
    // Track the start Y of the current group cluster (for parallel siblings)
    let mut cluster_start_y: f64 = 0.0;
    let mut msg_idx = 0usize;
    // Java's PlayingSpace.startingY = 8: the tile stack starts 8px below the body origin.
    // Standalone LifeEvents before the first message use this as their Y position.
    let mut current_position = lifeline_y + 8.0;
    // Activation tracking: (participant_idx, start_y, end_y, level)
    let mut activations: Vec<(usize, f64, f64, i32)> = Vec::new();
    // Destroy tracking: (participant_idx, destroy_y)
    let mut destroys: Vec<(usize, f64)> = Vec::new();
    // Pending activations per participant: each participant has its own stack of (start_y, level)
    let mut pending_activations: Vec<Vec<(f64, i32)>> = vec![Vec::new(); participants.len()];
    // Per-participant activation level (running count of activates minus deactivates)
    let mut participant_levels: Vec<i32> = vec![0; participants.len()];
    // Track which parallel groups had the Y offset (GROUP_HEADER_HEIGHT - 2) applied.
    // Used to adjust group_lifeline_bottom correctly.
    let mut group_parallel_offset: Vec<bool> = vec![false; groups.len()];
    // Track whether the current cluster has preceding parallel standalone messages.
    // The +GROUP_HEADER_HEIGHT-2 offset for parallel groups is only needed when
    // the group follows parallel standalone messages (not when it's the first parallel tile).
    let mut has_parallel_in_cluster = false;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let _has_text = !msg.label().is_empty();
            // Use precomputed wrapped lines
            let wrapped_lines = pre_wrapped_lines[msg_idx].clone();
            let line_count = wrapped_lines.len();
            msg_wrapped_lines.push(wrapped_lines);
            let text_h = (line_count as f64) * 13.0 + 13.0;
            msg_text_heights.push(text_h);
            let exo = msg_exo.get(msg_idx).copied().flatten();
            let is_self = exo.is_none() && msg.p1().code() == msg.p2().code();
            let is_reverse = is_self && msg.arrow_config().is_reverse_define();
            is_self_flags.push(is_self);
            let curr_group = msg_group.get(msg_idx).copied().flatten();
            let is_first_in_group = curr_group.is_some() && curr_group != prev_group
                && curr_group.is_some_and(|gi| msg_idx == groups[gi].msg_start);
            let is_else = curr_group
                .is_some_and(|gi| groups[gi].group_type == "else");
            let is_partition = curr_group
                .is_some_and(|gi| groups[gi].group_type == "partition");
            let header_extra = if is_partition { PARTITION_HEADER_EXTRA } else { 0.0 };
            let is_parallel = msg_parallel.get(msg_idx).copied().unwrap_or(false);

            let mut group_header_offset = 0.0;
            if is_parallel && msg_idx > 0 {
                if is_first_in_group && !is_else {
                    // Parallel group: top-aligned with the cluster's chaining point,
                    // plus group header offset for each nesting level.
                    let gi = curr_group.unwrap();
                    let g_msg_start = groups[gi].msg_start;
                    let innermost = *msg_start_innermost.get(&g_msg_start).unwrap_or(&groups[gi].nesting_level);
                    let par_level = groups.iter()
                        .find(|g| g.msg_start == g_msg_start && g.parallel && g.group_type != "else")
                        .map_or(groups[gi].nesting_level, |g| g.nesting_level);
                    let headers = (innermost + 1) - par_level;
                    let needs_offset = last_non_parallel_gho == 0.0 && has_parallel_in_cluster;
                    group_header_offset = last_non_parallel_gho.max((GROUP_HEADER_OFFSET * headers as f64) + if needs_offset { GROUP_HEADER_HEIGHT - 2.0 } else { 0.0 } + header_extra);
                    if needs_offset { group_parallel_offset[gi] = true; }
                    current_y = cluster_start_y + group_header_offset;
                } else if is_first_in_group && is_else {
                    // Else section within a parallel group: normal increment + else tile height
                    let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                    let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                    current_y += inc + ELSE_TILE_HEIGHT;
                } else {
                    // Parallel message not first in a group: use the cluster's chaining point
                    current_y = cluster_start_y;
                }
            } else if msg_idx == 0 {
                // First message overall
                current_y = lifeline_y + 1.0 + text_h;
                if is_first_in_group {
                    let curr_level = curr_group
                        .and_then(|gi| groups.get(gi))
                        .map_or(0, |g| g.nesting_level);
                    group_header_offset = GROUP_HEADER_OFFSET * (curr_level as f64 + 1.0) + header_extra;
                    current_y += group_header_offset;
                }
            } else if is_first_in_group && is_else {
                // Else section: normal increment + else tile height (no group gap/header)
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                current_y += inc + ELSE_TILE_HEIGHT;
            } else if is_first_in_group {
                // First message in a subsequent group (non-else)
                let curr_level = curr_group
                    .and_then(|gi| groups.get(gi))
                    .map_or(0, |g| g.nesting_level);
                let prev_level = prev_group
                    .and_then(|gi| groups.get(gi))
                    .map_or(0, |g| g.nesting_level);
                if curr_level > prev_level {
                    // Nested group within an else/parent: normal increment + header offset
                    let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                    let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                    current_y += inc;
                    group_header_offset = GROUP_HEADER_OFFSET + header_extra;
                    current_y += group_header_offset;
                } else if let Some(fb) = prev_frame_bottom {
                    // After a group ends: chaining point = fb + GROUP_GAP
                    let base = fb + GROUP_GAP;
                    group_header_offset = GROUP_HEADER_OFFSET + GROUP_HEADER_HEIGHT + header_extra;
                    current_y = base + group_header_offset;
                } else {
                    // First group after non-grouped messages
                    let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                    let inc = if let Some(nh) = prev_note_h { nh.max(1.0 + text_h + prev_self_extra) } else { 1.0 + text_h + prev_self_extra };
                    current_y += inc;
                    group_header_offset = GROUP_HEADER_OFFSET + header_extra;
                    current_y += group_header_offset;
                }
            } else if let Some(nh) = prev_note_h {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                current_y += nh.max(1.0 + text_h + prev_self_extra);
            } else {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                current_y += 1.0 + text_h + prev_self_extra;
            }

            // Update cluster_start_y for non-parallel group starts and non-grouped messages.
            // This tracks the chaining point of the previous TILE (not every message),
            // matching Java's YGauge.createParallel which shares the previous tile's min.
            if !is_parallel {
                if is_first_in_group && !is_else {
                    // Group start: chaining point = arrow_y - group_header_offset
                    cluster_start_y = current_y - group_header_offset;
                    last_non_parallel_gho = group_header_offset;
                } else if curr_group.is_none() {
                    // Non-grouped message: chaining point = arrow_y (no group header offset)
                    cluster_start_y = current_y;
                    last_non_parallel_gho = 0.0;
                }
                // Non-parallel tile starts a new cluster: no preceding parallel tiles.
                has_parallel_in_cluster = false;
            }
            // For parallel group starts, update cluster_start_y to include the group
            // header height so subsequent parallel messages in the same cluster chain
            // from below the group header. Only needed when preceded by parallel
            // standalone messages (has_parallel_in_cluster), not when the group is
            // the first parallel tile in the cluster.
            if is_parallel && is_first_in_group && !is_else && last_non_parallel_gho == 0.0 && has_parallel_in_cluster {
                cluster_start_y = cluster_start_y + GROUP_HEADER_HEIGHT - 2.0;
            }
            // Track parallel standalone messages for the next parallel group's offset check.
            if is_parallel && curr_group.is_none() {
                has_parallel_in_cluster = true;
            }

            // After a group ends, ensure next message/else clears the frame bottom
            if !is_parallel {
                if let Some(fb) = prev_frame_bottom {
                    let fb_level = prev_frame_bottom_level;
                    let curr_level = curr_group
                        .and_then(|gi| groups.get(gi))
                        .map_or(0, |g| g.nesting_level);
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
            prev_is_reverse = is_reverse;
            // Update current_position for LifeEvent Y tracking.
            // Activations on self-messages use arrow_y + SELF_ARROW_HEIGHT (bottom of self-arrow),
            // but deactivations use just arrow_y. We store the base arrow_y here and add
            // SELF_ARROW_HEIGHT only when pushing activations.
            current_position = current_y;

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
                    if group.parallel {
                        prev_frame_bottom = Some(prev_frame_bottom.map_or(fb, |existing| existing.max(fb)));
                    } else {
                        prev_frame_bottom = Some(fb);
                    }
                    prev_frame_bottom_level = group.nesting_level;

                    // If this is an else group, also check the parent group
                    // (which ends at the same message and has a taller frame)
                    if group.group_type == "else" {
                        let else_level = group.nesting_level;
                        for pgi in (0..gi).rev() {
                            if groups[pgi].nesting_level == else_level && groups[pgi].group_type != "else"
                                && groups[pgi].msg_start < group.msg_start
                            {
                                let pgroup = &groups[pgi];
                                // Parent covers from its msg_start to the else's msg_end
                                let p_last_mi = group.msg_end - 1;
                                let p_body_height = arrow_ys[p_last_mi] - arrow_ys[pgroup.msg_start] + last_msg_h;
                                let p_fy = arrow_ys[pgroup.msg_start] - GROUP_HEADER_OFFSET - GROUP_HEADER_HEIGHT;
                                let p_fh = p_body_height + GROUP_HEADER_HEIGHT + GROUP_MARGIN_Y_MAGIC / 2.0;
                                let p_fb = p_fy + p_fh;
                                if p_fb > fb {
                                    if pgroup.parallel {
                                        prev_frame_bottom = Some(prev_frame_bottom.map_or(p_fb, |existing| existing.max(p_fb)));
                                    } else {
                                        prev_frame_bottom = Some(p_fb);
                                    }
                                    prev_frame_bottom_level = pgroup.nesting_level;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            prev_group = curr_group;

            msg_idx += 1;
        } else if let SequenceEvent::LifeEvent(le) = event {
            // Determine Y position based on whether this is an inline or standalone LifeEvent.
            let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
            // Y position for standalone LifeEvents: after a self-message, the gauge's
            // chaining point (getMax) = current_y + SELF_ARROW_HEIGHT + 2*getPaddingY.
            // getPaddingY() = 4 (from AbstractComponentRoseArrow.getPaddingY).
            // Inline LifeEvents (attached to a message via ++/--) use the message's
            // arrow endpoint Y = current_y + SELF_ARROW_HEIGHT (no extra padding).
            const PADDING_Y: f64 = 4.0;
            let is_inline = le.message_index().is_some();
            let self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
            let standalone_extra = if prev_is_self { 2.0 * PADDING_Y } else { 0.0 };
            let act_y = current_position + self_extra + if is_inline { 0.0 } else { standalone_extra };
            let deact_y = if prev_is_self && !prev_is_reverse {
                current_position + self_extra + if is_inline { 0.0 } else { standalone_extra }
            } else {
                current_position
            };
            if le.is_activate() {
                if p_idx < pending_activations.len() {
                    let level = participant_levels[p_idx] + 1;
                    pending_activations[p_idx].push((act_y, level));
                }
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] += 1;
                }
            } else if le.is_deactivate() {
                if p_idx < pending_activations.len() {
                    if let Some((start_y, level)) = pending_activations[p_idx].pop() {
                        // Java's LiveBoxes.addStep: when a deactivation's Y matches
                        // an existing step Y, add 5.0 to avoid zero-height bars.
                        // The tile Y offset (8) + deactivation offset (5) = 13 total.
                        let end_y = if (start_y - deact_y).abs() < 0.001 {
                            deact_y + 13.0
                        } else {
                            deact_y
                        };
                        activations.push((p_idx, start_y, end_y, level));
                    }
                }
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] = (participant_levels[p_idx] - 1).max(0);
                }
            } else if le.is_destroy() {
                if p_idx < pending_activations.len() {
                    if let Some((start_y, level)) = pending_activations[p_idx].pop() {
                        activations.push((p_idx, start_y, deact_y, level));
                    }
                }
                destroys.push((p_idx, deact_y));
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] = (participant_levels[p_idx] - 1).max(0);
                }
            }
        }
    }

    // ── Compute group frame dimensions ──────────────────────────────────
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
                            ExoType::ToLeft => (0.0, p2_c - exo_w, p2_c),
                            ExoType::FromRight => (0.0, p1_c, p1_c + exo_w),
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
                            + 2.0 * NOTE_PADDING_X
                            + delta_shadow;
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
        // Also account for LifeEvent tiles (activation bars) inside the group.
        // Java GroupingTile constructor uses tile.getDrawnMinX()/getDrawnMaxX()
        // for each tile, including LifeEventTiles. LifeEventTile.getMaxX() =
        // posC + levelAt * LIVE_DELTA_SIZE, getMinX() = posC - LIVE_DELTA_SIZE
        // (if level > 0). These extend the frame beyond the message endpoints.
        // Track per-group running activation levels: iterate ALL events to build
        // the correct running level, but only record max levels within the group's
        // range so the frame width doesn't include activations from other groups.
        let mut run_levels: Vec<i32> = vec![0; participants.len()];
        let mut group_max_levels: Vec<i32> = vec![0; participants.len()];
        let mut mi_check = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(_) = event {
                mi_check += 1;
                // At the first message in the group, record the current levels
                // as the initial max (activations from before the group carry over).
                if mi_check == group.msg_start + 1 {
                    for pi in 0..participants.len() {
                        group_max_levels[pi] = run_levels[pi];
                    }
                }
            } else if let SequenceEvent::LifeEvent(le) = event {
                let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                if p_idx < participants.len() {
                    if le.is_activate() {
                        run_levels[p_idx] += 1;
                    } else if le.is_deactivate() || le.is_destroy() {
                        run_levels[p_idx] = (run_levels[p_idx] - 1).max(0);
                    }
                    // Only update max levels for LifeEvents within the group's range.
                    if mi_check > group.msg_start && mi_check <= group.msg_end {
                        group_max_levels[p_idx] = group_max_levels[p_idx].max(run_levels[p_idx]);
                    }
                }
                // LifeEvents inside the group's message range extend the frame.
                if mi_check > group.msg_start && mi_check <= group.msg_end
                    && p_idx < pos_c_vals.len() {
                        let level = group_max_levels[p_idx] as f64;
                        if level > 0.0 {
                            max_x = max_x.max(pos_c_vals[p_idx] + level * ACTIVATION_BAR_EXPLICIT_OFFSET);
                            min_x = min_x.min(pos_c_vals[p_idx] - ACTIVATION_BAR_EXPLICIT_OFFSET);
                        }
                    }
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
            // Frame Y: the group's chaining point (firstY) is at
            // arrowY - contactPointRelative, and the frame top is at
            // firstY + EXTERNAL_MARGINY. So:
            //   fy = arrowY - contactPointRelative + EXTERNAL_MARGINY - GROUP_HEADER_OFFSET * n
            // where contactPointRelative = text_h - 7 (text_h = getTextHeight + 11,
            // contactPointRelative = getTextHeight + 4) and EXTERNAL_MARGINY = 4.
            // Simplified: fy = arrowY - (GROUP_HEADER_OFFSET * n + text_h - 11)
            let innermost = *msg_start_innermost.get(&group.msg_start).unwrap_or(&group.nesting_level);
            let headers_to_subtract = (innermost + 1) - group.nesting_level;
            let first_text_h = msg_text_heights[group.msg_start];
            let fy = arrow_ys[group.msg_start] - GROUP_HEADER_OFFSET * headers_to_subtract as f64 - first_text_h + 11.0 - p_extra;
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
                && (parent_gi.is_none()
                    || (groups[pj].msg_end - groups[pj].msg_start)
                        < (groups[parent_gi.unwrap()].msg_end - groups[parent_gi.unwrap()].msg_start))
                {
                    parent_gi = Some(pj);
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
    for _gf in group_frames.iter() {
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
    let (last_arrow_y, last_is_self, last_arrow_idx) = arrow_ys
        .iter()
        .enumerate()
        .zip(is_self_flags.iter())
        .max_by(|((_, ya), _), ((_, yb), _)| ya.partial_cmp(yb).unwrap_or(std::cmp::Ordering::Equal))
        .map_or((lifeline_y + ARROW_Y_BASE, false, 0), |((mi, &y), &s)| (y, s, mi));

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
            let g = &groups[i];
            // Check if this group or any of its else children have nesting Y extension.
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
            let val = if g.parallel {
                // Parallel groups: frame_bottom + margin. When the Y offset
                // (GROUP_HEADER_HEIGHT - 2) was applied (group follows standalone
                // parallel messages), subtract GROUP_HEADER_HEIGHT because the frame
                // already includes the header offset in its Y position.
                // When no Y offset was applied (group follows non-parallel group),
                // use the normal margin.
                if group_parallel_offset[i] {
                    frame_bottom + GROUP_MARGIN_Y_MAGIC + GROUP_MARGIN_Y - GROUP_HEADER_HEIGHT
                } else {
                    frame_bottom + GROUP_MARGIN_Y_MAGIC + GROUP_MARGIN_Y
                }
            } else if has_nesting {
                // -1 correction for text_h=26 vs Java's 25, only when self-messages present
                frame_bottom - nesting_y_ext[i] + GROUP_MARGIN_Y_MAGIC + GROUP_MARGIN_Y - if has_self_msgs { 1.0 } else { 0.0 }
            } else if has_parallel {
                // Use per-group msg_ygauge_max to avoid inflation from messages outside the group
                let group_msg_ygauge_max = arrow_ys.iter().enumerate()
                    .filter(|&(mi, _)| mi >= g.msg_start && mi < g.msg_end)
                    .map(|(_, &y)| y + java_preferred_height)
                    .fold(0.0_f64, f64::max);
                group_msg_ygauge_max + PAGE_MARGIN
            } else {
                frame_bottom + GROUP_MARGIN_Y_MAGIC + GROUP_MARGIN_Y
            };
            val
        })
        .fold(0.0_f64, f64::max);
    // When parallel messages are present, cap normal_lifeline_bottom at msg_ygauge_max + PAGE_MARGIN
    // to avoid 1px overshoot from text_h=26 vs Java's 25 in self-messages.
    // Also cap at group_lifeline_bottom when the max-arrow-Y message is inside a parallel group,
    // because its height is already accounted for by the group's own lifeline_bottom.
    let max_arrow_in_parallel_group = groups.iter().any(|g| g.parallel && last_arrow_idx >= g.msg_start && last_arrow_idx < g.msg_end);
    let normal_lifeline_bottom = if has_parallel {
        let capped = normal_lifeline_bottom.min(msg_ygauge_max + PAGE_MARGIN);
        if max_arrow_in_parallel_group && group_lifeline_bottom > 0.0 {
            capped.min(group_lifeline_bottom)
        } else {
            capped
        }
    } else {
        normal_lifeline_bottom
    };
    let lifeline_bottom = normal_lifeline_bottom.max(note_lifeline_bottom).max(group_lifeline_bottom);
    // Flush remaining pending activations (autoactivate without explicit deactivate)
    for (pi, stack) in pending_activations.iter().enumerate() {
        for &(start_y, level) in stack {
            activations.push((pi, start_y, lifeline_bottom, level));
        }
    }

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
        let layout_w = note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2 + 2.0 * NOTE_PADDING_X + delta_shadow_note;
        let layout_left = if is_self_msg && is_reverse {
            // LEFT note on reverse self-message: CommunicationTileSelfNoteLeft.getMinX()
            // = tile.getMinX() - note_width = (posC - comp_width - liveDeltaAdj) - layout_w
            let label_w = pre_wrapped_widths.get(msg_idx).copied().unwrap_or_else(|| max_line_width(&bounder, &font_m, &msg_label));
            let comp_width = (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
            let level = msg_p1_levels.get(msg_idx).copied().unwrap_or(0);
            let live_delta_adj = if level > 0 { ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 };
            p_center_solver - comp_width - live_delta_adj - layout_w
        } else {
            p_center_solver - layout_w
        };
        if layout_left < min_layout_left {
            min_layout_left = layout_left;
        }
    }
    // Account for reverse self-message loops extending left (getMinX).
    // Java CommunicationTileSelf.getMinX() for reverse: posC - compWidth.
    // This is critical for single-participant diagrams where no disjoint
    // constraint pushes the participant right.
    let mut sm_msg_idx = 0usize;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
            if p1_idx == p2_idx && msg.arrow_config().is_reverse_define() {
                let text_w = pre_wrapped_widths.get(sm_msg_idx).copied().unwrap_or(0.0);
                let max_act = *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                let comp_width = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0) + ACTIVATION_BAR_EXPLICIT_OFFSET * max_act;
                let p_center = pos_c_vals.get(p1_idx).copied().unwrap_or(0.0);
                let loop_left = p_center - comp_width;
                if loop_left < min_layout_left {
                    min_layout_left = loop_left;
                }
            }
            sm_msg_idx += 1;
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
    // Track the note path's actual right edge (for ensureVisible/total width).
    // The path width is x2 = (int)(pureTextWidth + oldPaddingX1 + oldPaddingX2),
    // which is smaller than layout_w (= getPreferredWidth = x2 + 2*paddingX + deltaShadow).
    let mut max_note_path_right = 0.0_f64;
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
        let layout_w = note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2 + 2.0 * NOTE_PADDING_X + delta_shadow_note;
        // Java CommunicationTileNoteRight.getNotePosition = posC + level * LIVE_DELTA_SIZE
        // getMaxX = getNotePosition + getPreferredWidth = posC + level*5 + layout_w
        let note_level = if is_reverse { note_p1_levels.get(msg_idx).copied().unwrap_or(0) } else { note_p2_levels.get(msg_idx).copied().unwrap_or(0) };
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
        // Compute the note path's actual right edge for ensureVisible.
        // path_right = noteX + x2 where noteX = posC + level_dx + NOTE_PADDING_X
        // and x2 = (int)(pureTextWidth + oldPaddingX1 + oldPaddingX2)
        let x2 = (note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2).trunc();
        let note_x = if is_self_msg && !is_reverse {
            let label_w = pre_wrapped_widths.get(msg_idx).copied().unwrap_or_else(|| max_line_width(&bounder, &font_m, &msg_label));
            let comp_width = (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
            p_center + comp_width + level_dx + NOTE_PADDING_X
        } else {
            p_center + level_dx + NOTE_PADDING_X
        };
        let path_right = note_x + x2;
        if path_right > max_note_path_right {
            max_note_path_right = path_right;
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
    // Include header/footer/legend/caption widths in total width
    let header_w = header_dim.map_or(0.0, |(w, _)| w);
    let title_w = title_dim.map_or(0.0, |(w, _)| w);
    let legend_w = legend_dim.map_or(0.0, |(w, _)| w);
    let caption_w = caption_dim.map_or(0.0, |(w, _)| w);
    let footer_w = footer_dim.map_or(0.0, |(w, _)| w);
    let max_element_width = header_w.max(title_w).max(legend_w).max(caption_w).max(footer_w);
    let content_right = rightmost_x.max(title_rightmost).max(max_note_right).max(max_frame_right);
    let has_extra_elements = header_text.is_some() || footer_text.is_some() || legend_text.is_some() || caption_text.is_some();
    let total_width = if has_extra_elements {
        content_right.max(max_element_width + PAGE_MARGIN * 2.0) + PAGE_MARGIN * 2.0
    } else if title_width > 0.0 {
        content_right + PAGE_MARGIN * 2.0 + 1.0
    } else {
        content_right + PAGE_MARGIN * 2.0
    } as i64;

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
    let height_extra = if hide_footbox { 0.0 } else if has_actor { 0.0 } else { HEIGHT_EXTRA };
    // Compute extra height for legend, caption, footer (below footbox)
    let legend_h = legend_dim.map_or(0.0, |(_, h)| h);
    let caption_h = caption_dim.map_or(0.0, |(_, h)| h);
    let footer_h = footer_dim.map_or(0.0, |(_, h)| h);
    let bottom_extra = if legend_text.is_some() || caption_text.is_some() || footer_text.is_some() {
        let mut extra = 0.0;
        if legend_text.is_some() { extra += LEGEND_GAP + legend_h; }
        if caption_text.is_some() { extra += CAPTION_GAP + caption_h; }
        if footer_text.is_some() { extra += FOOTER_GAP + footer_h; }
        extra
    } else {
        0.0
    };
    let total_height = if has_extra_elements {
        footbox_bottom + PAGE_MARGIN + height_extra + delta_shadow + bottom_extra
    } else {
        footbox_bottom + PAGE_MARGIN * 2.0 + height_extra + delta_shadow
    };

    // ── Create SvgGraphics ────────────────────────────────────────────────
    let mut option = SvgOption::basic();
    option.set_root_attribute("data-diagram-type", "SEQUENCE");
    if let Some(ref bg) = doc_bg_color {
        if let Some((r, g, b)) = parse_hex_color(bg) {
            option.set_backcolor(plantuml_klimt::color::HColor::rgb(r, g, b));
        } else {
            option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
        }
    } else {
        option.set_backcolor(plantuml_klimt::color::HColor::rgb(0xFF, 0xFF, 0xFF));
    }
    if let Some(title) = svg_title {
        option.set_title(title);
    }
    if let Some(desc) = svg_desc {
        option.set_desc(desc);
    }
    let mut svg = SvgGraphics::new(0, option);
    // Add shadow filter for rose skin and get the filter ID for applying to elements
    let shadow_filter_id: Option<String> = if skin_rose {
        let fid = svg.add_shadow_filter();
        Some(fid.to_string())
    } else {
        None
    };
    let skin = SkinConfig {
        skin_rose,
        color_stroke,
        color_arrow,
        color_note_back,
        shadow_filter_id,
    };

    // ── Draw header (if present) ─────────────────────────────────────────
    // Centering width = rightmost_x + PAGE_MARGIN * 2 - 1 (matches Java layout).
    let centering_width = rightmost_x + PAGE_MARGIN * 2.0 - 1.0;
    if let Some(header_txt) = header_text {
        let font_h = UFont::sans_serif(header_font_size as i32);
        let text_w = bounder.calculate_dimension(&font_h, header_txt).width();
        let rect_w = header_dim.map_or(text_w, |(w, _)| w);
        let rect_h = header_dim.map_or(header_font_size, |(_, h)| h);
        let rect_y = PAGE_MARGIN;
        // Right-aligned: x = centering_width - PAGE_MARGIN - rect_w
        let rect_x = centering_width - PAGE_MARGIN - rect_w;
        let text_y = rect_y + header_font_size * 7.0 / 9.0;

        let line_str = header_line.map(|n| n.to_string());
        let mut attrs = vec![("class", "header")];
        if let Some(ref ls) = line_str {
            attrs.push(("data-source-line", ls.as_str()));
        }
        svg.open_group_with_attrs(&attrs);

        // Background rect
        // Background rect (stroke_width=1 with stroke=none produces style="stroke:none;")
        svg.set_fill_color(header_bg_color.as_deref().unwrap_or("none"));
        svg.set_stroke_color(None);
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(rect_x, rect_y, rect_w, rect_h, 0.0, 0.0, 0.0);

        // Text
        svg.set_fill_color(&header_font_color);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            header_txt,
            rect_x,
            text_y,
            None,
            header_font_size as i32,
            None,
            None,
            None,
            text_w,
            &indexmap::IndexMap::new(),
            None,
        );
        svg.close_group();
    }

    // ── Draw title (if present) ──────────────────────────────────────────
    if let Some(title_text) = title {
        let text_w = bounder.calculate_dimension(&font_p, title_text).width();
        let rect_w = text_w + 10.0; // 5px padding each side
        let rect_h = 24.0;
        let rect_y = PAGE_MARGIN + STARTING_Y + header_extra;
        // When no header/footer/legend/caption: title uses fixed x = PAGE_MARGIN + 10
        // When extra elements present: title is centered using centering_width
        let (rect_x, text_x) = if has_extra_elements {
            let rx = (centering_width - rect_w) / 2.0;
            (rx, rx + 5.0)
        } else {
            (PAGE_MARGIN + 10.0, PAGE_MARGIN + 10.0)
        };
        let text_y = rect_y + 15.889; // 3px top padding + ASCENT_14
        let line_str = title_line.map(|n| n.to_string());
        let mut attrs = vec![("class", "title")];
        if let Some(ref ls) = line_str {
            attrs.push(("data-source-line", ls.as_str()));
        }
        svg.open_group_with_attrs(&attrs);

        // Background rect (if style defines one)
        if let Some(ref bg) = title_bg_color {
            svg.set_fill_color(bg);
            svg.set_stroke_color(None);
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(rect_x, rect_y, rect_w, rect_h, 0.0, 0.0, 0.0);
        }

        // Text
        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            title_text,
            text_x,
            text_y,
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
            // Draw background fill rect first (if group has a backcolor)
            if let Some(ref backcolor) = groups[gi].backcolor {
                svg.set_fill_color(backcolor);
                svg.set_stroke_color(Some(backcolor));
                svg.set_stroke_width(1.0, None);
                svg.svg_rectangle(fx + x_offset, fy, fw, fh, 0.0, 0.0, 0.0);
            }
            // Draw border rect on top
            svg.set_fill_color("none");
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
        svg.set_stroke_color(Some(color_lifeline));
        svg.set_stroke_width(stroke_width_lifeline, Some([5.0, 5.0]));
        svg.svg_line(cx, ly, cx, ly + ll_height, 0.0);

        svg.close_group();

        // Draw activation bars for this participant, sorted by level (ascending)
        // Java renders LifeEventTiles in ascending level order (outermost first).
        let mut participant_acts: Vec<(usize, f64, f64, i32)> = activations.iter()
            .filter(|(pi, _, _, _)| *pi == i)
            .copied()
            .collect();
        participant_acts.sort_by_key(|(_, _, _, level)| *level);
        for &(_pi, start_y, end_y, level) in &participant_acts {
            svg.open_group(None);
            svg.title("");
            svg.set_fill_color("#FFF");
            svg.set_stroke_color(Some(color_lifeline));
            svg.set_stroke_width(1.0, None);
            let level_dx = (level as f64 - 1.0) * ACTIVATION_BAR_EXPLICIT_OFFSET;
            svg.set_filter(skin.shadow_filter_id.as_deref());
            svg.svg_rectangle(
                cx - ACTIVATION_BAR_EXPLICIT_OFFSET + level_dx,
                start_y,
                ACTIVATION_BAR_EXPLICIT_WIDTH,
                end_y - start_y,
                0.0,
                0.0,
                0.0,
            );
            svg.set_filter(None);
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
            // Actor: draw text first, then stickman (matching Java's ComponentRoseActor order)
            // Text below stickman
            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
            let text_y = head_y + STICKMAN_HEIGHT + ASCENT_14 - 2.0;

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

            // Stickman head circle
            let head_cy = head_y + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
            svg.set_fill_color(color_back);
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(STICKMAN_THICKNESS, None);
            svg.svg_ellipse(cx, head_cy, STICKMAN_HEAD_DIAM / 2.0, STICKMAN_HEAD_DIAM / 2.0, 0.0);

            // Body + arms + legs as path
            let body_top = head_cy + STICKMAN_HEAD_DIAM / 2.0;
            let body_bottom = body_top + STICKMAN_BODY_LEN;
            let arms_y = body_top + 8.0;
            let legs_bottom = body_bottom + STICKMAN_LEGS_Y;

            svg.set_fill_color("none");
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(STICKMAN_THICKNESS, None);
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
            svg.set_fill_color(color_back);
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(stroke_width_head, None);
            svg.set_filter(skin.shadow_filter_id.as_deref());
            svg.svg_rectangle(x, head_y, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
            svg.set_filter(None);

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
            let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
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
            // Actor text height = ACTOR_HEAD_LAYOUT_HEIGHT - STICKMAN_HEIGHT = 74 - 60 = 14
            // (pure text block height for font-size 14, no top/bottom padding for actors)
            let stickman_top = footbox_y + (ACTOR_HEAD_LAYOUT_HEIGHT - STICKMAN_HEIGHT);
            let head_cy = stickman_top + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
            svg.set_fill_color(color_back);
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(STICKMAN_THICKNESS, None);
            svg.svg_ellipse(cx, head_cy, STICKMAN_HEAD_DIAM / 2.0, STICKMAN_HEAD_DIAM / 2.0, 0.0);

            let body_top = head_cy + STICKMAN_HEAD_DIAM / 2.0;
            let body_bottom = body_top + STICKMAN_BODY_LEN;
            let arms_y = body_top + 8.0;
            let legs_bottom = body_bottom + STICKMAN_LEGS_Y;

            svg.set_fill_color("none");
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(STICKMAN_THICKNESS, None);
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
            svg.set_fill_color(color_back);
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(stroke_width_head, None);
            svg.set_filter(skin.shadow_filter_id.as_deref());
            svg.svg_rectangle(x, footbox_y, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
            svg.set_filter(None);

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
    // Background rectangle sets SVG dimensions via ensure_visible.
    // ensure_visible(total_width, ...) → max_x = total_width + 1 = SVG width.
    svg.svg_rectangle(0.0, 0.0, total_width as f64, total_height, 0.0, 0.0, 0.0);
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
            // Skip drawing for hidden messages (they occupy Y space but aren't drawn)
            let is_hidden = msg_hidden.get(msg_idx).copied().unwrap_or(false);
            if !is_hidden {
                let exo = msg_exo.get(msg_idx).copied().flatten();
                if let Some(exo_type) = exo {
                    draw_exo_message(
                        &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, y,
                        p1_idx, exo_type,
                        msg_wrapped_lines.get(msg_idx).map_or(&[], std::vec::Vec::as_slice),
                        &skin,
                    );
                } else {
                    draw_message(
                        &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, y,
                        p1_idx, p2_idx,
                        msg_self_levels.get(msg_idx).map_or(0, |&(li, _lc)| li),
                        msg_self_levels.get(msg_idx).map_or(0, |&(_li, lc)| lc),
                        msg_wrapped_lines.get(msg_idx).map_or(&[], std::vec::Vec::as_slice),
                        msg_p1_levels.get(msg_idx).copied().unwrap_or(0),
                        msg_p2_levels.get(msg_idx).copied().unwrap_or(0),
                        &skin,
                    );
                }
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
                    note_p1_levels.get(msg_idx).copied().unwrap_or(0),
                    note_p2_levels.get(msg_idx).copied().unwrap_or(0),
                    &skin,
                );
            }
            msg_idx += 1;
        }
    }

    // ── Draw legend, caption, footer (below footbox) ─────────────────────
    if legend_text.is_some() || caption_text.is_some() || footer_text.is_some() {
        let mut below_y = footbox_bottom + LEGEND_GAP;

        // Legend
        if let Some(legend_txt) = legend_text {
            let font_l = UFont::sans_serif(legend_font_size as i32);
            let text_w = bounder.calculate_dimension(&font_l, legend_txt).width();
            let rect_w = text_w + 10.0;
            let rect_h = 24.0;
            let rect_x = (centering_width - rect_w) / 2.0;
            let text_x = rect_x + 5.0;
            let text_y = below_y + 15.889;

            svg.open_group_with_attrs(&[("class", "legend")]);
            // Background rect with rounded corners and border
            svg.set_fill_color(legend_bg_color.as_deref().unwrap_or("none"));
            svg.set_stroke_color(Some("#000000"));
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(rect_x, below_y, rect_w, rect_h, LEGEND_CORNER_RADIUS, LEGEND_CORNER_RADIUS, 0.0);
            // Text
            svg.set_fill_color(COLOR_TEXT);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            svg.text(
                legend_txt,
                text_x,
                text_y,
                None,
                legend_font_size as i32,
                None,
                None,
                None,
                text_w,
                &indexmap::IndexMap::new(),
                None,
            );
            svg.close_group();

            below_y += rect_h + CAPTION_GAP;
        }

        // Caption
        if let Some(caption_txt) = caption_text {
            let font_c = UFont::sans_serif(caption_font_size as i32);
            let text_w = bounder.calculate_dimension(&font_c, caption_txt).width();
            let rect_w = text_w;
            let rect_h = caption_font_size;
            let rect_x = (centering_width - rect_w) / 2.0;
            let text_y = below_y + caption_font_size * 7.0 / 9.0;

            let line_str = caption_line.map(|n| n.to_string());
            let mut attrs = vec![("class", "caption")];
            if let Some(ref ls) = line_str {
                attrs.push(("data-source-line", ls.as_str()));
            }
            svg.open_group_with_attrs(&attrs);
            // Background rect
            svg.set_fill_color(caption_bg_color.as_deref().unwrap_or("none"));
            svg.set_stroke_color(None);
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(rect_x, below_y, rect_w, rect_h, 0.0, 0.0, 0.0);
            // Text
            svg.set_fill_color(COLOR_TEXT);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            svg.text(
                caption_txt,
                rect_x,
                text_y,
                None,
                caption_font_size as i32,
                None,
                None,
                None,
                text_w,
                &indexmap::IndexMap::new(),
                None,
            );
            svg.close_group();

            below_y += rect_h + FOOTER_GAP;
        }

        // Footer
        if let Some(footer_txt) = footer_text {
            let font_f = UFont::sans_serif(footer_font_size as i32);
            let text_w = bounder.calculate_dimension(&font_f, footer_txt).width();
            let rect_w = text_w;
            let rect_h = footer_font_size;
            let rect_x = (centering_width - rect_w) / 2.0;
            let text_y = below_y + footer_font_size * 7.0 / 9.0;

            let line_str = footer_line.map(|n| n.to_string());
            let mut attrs = vec![("class", "footer")];
            if let Some(ref ls) = line_str {
                attrs.push(("data-source-line", ls.as_str()));
            }
            svg.open_group_with_attrs(&attrs);
            // Background rect
            svg.set_fill_color(footer_bg_color.as_deref().unwrap_or("none"));
            svg.set_stroke_color(None);
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(rect_x, below_y, rect_w, rect_h, 0.0, 0.0, 0.0);
            // Text
            svg.set_fill_color(&footer_font_color);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            svg.text(
                footer_txt,
                rect_x,
                text_y,
                None,
                footer_font_size as i32,
                None,
                None,
                None,
                text_w,
                &indexmap::IndexMap::new(),
                None,
            );
            svg.close_group();
        }
    }


    // ── Clean up Real constraints ───────────────────────────────────────
    plantuml_real::clear_forces(xorigin.get_line());

    svg.create_xml()
}
/// Skin configuration for rendering (rose skin overrides).
struct SkinConfig {
    skin_rose: bool,
    color_stroke: &'static str,
    color_arrow: &'static str,
    color_note_back: &'static str,
    shadow_filter_id: Option<String>,
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
    _msg_p2_level: i32,
    note_p1_level: i32,
    note_p2_level: i32,
    skin: &SkinConfig,
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
    let polygon_w = text_width_total.trunc();
    // Note layout width = text_width + oldPaddingX1 + oldPaddingX2 + 2*paddingX
    let layout_w = text_width_total + 2.0 * NOTE_PADDING_X + if skin.skin_rose { 9.0 } else { 0.0 };
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
        NotePosition::Right => if is_reverse { note_p1_level } else { note_p2_level },
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
                // LEFT on <-- : polygon at posC - comp_width - liveDeltaAdj - layout_w + paddingX
                // Rose skin uses 2*paddingX (matching older PlantUML with deltaShadow).
                let live_delta_adj = if msg_p1_level > 0 { ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 };
                let padding = if skin.skin_rose { 2.0 * NOTE_PADDING_X } else { NOTE_PADDING_X };
                p_center - comp_width - live_delta_adj - layout_w + padding
            } else {
                // LEFT on --> or normal: polygon at posC - layout_w + paddingX
                p_center - layout_w + NOTE_PADDING_X
            }
        }
    };

    // Draw note shape (folded corner rectangle) with absolute coordinates
    svg.set_fill_color(skin.color_note_back);
    svg.set_stroke_color(Some(skin.color_stroke));
    svg.set_stroke_width(if skin.skin_rose { 1.0 } else { STROKE_WIDTH_BOX }, None);
    let path_d = format_note_path_abs(note_x, note_y_top, polygon_w, note_h, NOTE_CORNERSIZE);
    svg.set_filter(skin.shadow_filter_id.as_deref());
    svg.svg_path(&path_d, 0.0);
    // Java's svgPath calls ensureVisible for each path coordinate + 2*deltaShadow.
    // Our svg_path doesn't do this, so we manually call ensure_visible for the note path's bounding box.
    let delta_shadow_note_ensure = if skin.skin_rose { 10.0 } else { 0.0 };
    svg.ensure_visible(note_x + polygon_w + 2.0 * delta_shadow_note_ensure, note_y_top + note_h + 2.0 * delta_shadow_note_ensure);
    svg.set_filter(None);

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
    skin: &SkinConfig,
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
            // Java: loop_x = posC - liveDeltaAdj - xRight
            // where liveDeltaAdj = if levelIgnore > 0 { LIVE_DELTA_SIZE } else { 0 }
            // (from CommunicationTileSelf.getMinX() for reverse)
            let live_delta_adj = if level_ignore > 0 { ld } else { 0.0 };
            (cx - live_delta_adj - SELF_XRIGHT, -1.0)
        } else {
            // --> : loop to the right, arrowhead points left
            // Java: x1 = posC + ld * levelIgnore + (if levelIgnore < levelConsidere { ld * (levelConsidere - levelIgnore) } else { 0 })
            // = posC + ld * max(levelIgnore, levelConsidere)
            (cx + ld * max_level + SELF_XRIGHT, 1.0)
        };
        let y_bottom = y + SELF_ARROW_HEIGHT;

        // Compute top and bottom near points based on activation levels
        // Ported from ComponentRoseSelfArrow.drawLeftSide (reverse) and drawRightSide (non-reverse).
        // The Java code computes local x1/x2 offsets based on dx = (levelIgnore - levelConsidere) * LD
        // and level = levelIgnore, then the absolute positions are:
        //   top_near = posC - liveDeltaAdj - x1_local  (reverse)
        //   bottom_near = posC - liveDeltaAdj - x2_local - extraline  (reverse)
        // where liveDeltaAdj = if levelIgnore > 0 { LD } else { 0 }
        let (top_near, bottom_near) = if is_reverse {
            let live_delta_adj = if level_ignore > 0 { ld } else { 0.0 };
            let extra_live_delta_indent = level_ignore as f64 * ld;
            // x1_local and x2_local before the final x1 += 1
            let (x1_pre, x2_local) = if delta_x1 < 0.0 {
                let x1 = 0.0;
                let x2 = 1.0 + if level_ignore > 0 { -extra_live_delta_indent } else { ld };
                (x1, x2)
            } else if delta_x1 > 0.0 {
                let x1 = if level_ignore > 1 { ld - extra_live_delta_indent } else { 0.0 };
                let x2 = 1.0 + if level_ignore == 1 { -ld } else { 0.0 };
                (x1, x2)
            } else if level_ignore > 1 {
                let adj = extra_live_delta_indent - ld;
                (-adj, 1.0 - adj)
            } else {
                (0.0, 1.0)
            };
            let x1_local = x1_pre + 1.0; // Java: x1 += 1
            // extraline = 1 for normal full arrowheads (regardless of dashed line).
            // dressing2 holds the arrowhead for both --> and <--.
            let arrow_head = msg.arrow_config().dressing2().head();
            let arrow_part = msg.arrow_config().dressing2().part();
            let extraline = if arrow_head == plantuml_skin::ArrowHead::Normal
                && arrow_part == plantuml_skin::ArrowPart::Full
            { 1.0 } else { 0.0 };
            // Circle decoration shortening (from ComponentRoseSelfArrow.drawLeftSide):
            // decoration1 (circle at top): x1 += diamCircle/2 - thinCircle (+ thinCircle if head==None)
            //   = 4 - 1.5 = 2.5, + 1.5 if dressing1.head == None → 4.0 total if None, 2.5 otherwise
            // decoration2 (circle at bottom): x2 += diamCircle/2 + thinCircle = 4 + 1.5 = 5.5
            let has_circle1 = msg.arrow_config().decoration1() == plantuml_skin::ArrowDecoration::Circle;
            let has_circle2 = msg.arrow_config().decoration2() == plantuml_skin::ArrowDecoration::Circle;
            let circle_shorten1 = if has_circle1 {
                let base = 4.0 - 1.5; // diamCircle/2 - thinCircle
                if msg.arrow_config().dressing1().head() == plantuml_skin::ArrowHead::None {
                    base + 1.5 // + thinCircle
                } else {
                    base
                }
            } else { 0.0 };
            let circle_shorten2 = if has_circle2 { 5.5 } else { 0.0 }; // diamCircle/2 + thinCircle
            let top_n = cx - live_delta_adj - x1_local - circle_shorten1;
            let bottom_n = cx - live_delta_adj - x2_local - extraline - circle_shorten2;
            (top_n, bottom_n)
        } else {
            // Non-reverse (drawRightSide): original formula matching older PlantUML behavior.
            // top_near = posC + LD * levelIgnore
            // bottom_near = posC + LD * levelConsidere + (1 if deltaX1 <= 0 else 0)
            // Circle shortening for non-reverse (from drawRightSide):
            // decoration1: x1 += diamCircle/2 + thinCircle + 1 (- thinCircle+1 if head==None)
            //   = 4 + 1.5 + 1 = 6.5, - 2.5 if head==None → 4.0 if None, 6.5 otherwise
            // decoration2: x2 += diamCircle/2 + thinCircle = 5.5
            let has_circle1 = msg.arrow_config().decoration1() == plantuml_skin::ArrowDecoration::Circle;
            let has_circle2 = msg.arrow_config().decoration2() == plantuml_skin::ArrowDecoration::Circle;
            let circle_shorten1 = if has_circle1 {
                let base = 4.0 + 1.5 + 1.0; // diamCircle/2 + thinCircle + 1
                if msg.arrow_config().dressing1().head() == plantuml_skin::ArrowHead::None {
                    base - (1.5 + 1.0) // - (thinCircle + 1)
                } else {
                    base
                }
            } else { 0.0 };
            let circle_shorten2 = if has_circle2 { 5.5 } else { 0.0 };
            let top_n = cx + ld * level_ignore as f64 + circle_shorten1;
            let bottom_n = cx + ld * level_considere as f64 + (if delta_x1 <= 0.0 { 1.0 } else { 0.0 }) + circle_shorten2;
            (top_n, bottom_n)
        };

        // Circle decoration rendering for self-messages (drawn BEFORE lines, matching Java order)
        // Ported from ComponentRoseSelfArrow.drawLeftSide (reverse) / drawRightSide (non-reverse)
        // Circle: diamCircle=8 (rx=ry=4), thinCircle=1.5 (stroke-width)
        // Position: cx = participant_center ∓ (diamCircle/2 + thinCircle) = cx ∓ 5.5
        // Y: circle center is thinCircle/2 (0.75) above the line it sits on
        let has_circle1 = msg.arrow_config().decoration1() == plantuml_skin::ArrowDecoration::Circle;
        let has_circle2 = msg.arrow_config().decoration2() == plantuml_skin::ArrowDecoration::Circle;
        if has_circle1 || has_circle2 {
            const DIAM_CIRCLE: f64 = 8.0;
            const THIN_CIRCLE: f64 = 1.5;
            let circle_offset = DIAM_CIRCLE / 2.0 + THIN_CIRCLE; // 5.5
            let circle_rx = DIAM_CIRCLE / 2.0; // 4.0
            let circle_ry = DIAM_CIRCLE / 2.0; // 4.0
            let circle_cx = if is_reverse { cx - circle_offset } else { cx + circle_offset };
            svg.set_fill_color("#000000");
            svg.set_stroke_color(Some(skin.color_arrow));
            svg.set_stroke_width(THIN_CIRCLE, None);
            if has_circle1 {
                // Circle at top (decoration1): cy = y - thinCircle/2
                svg.svg_ellipse(circle_cx, y - THIN_CIRCLE / 2.0, circle_rx, circle_ry, 0.0);
            }
            if has_circle2 {
                // Circle at bottom (decoration2): cy = y_bottom - thinCircle/2
                svg.svg_ellipse(circle_cx, y_bottom - THIN_CIRCLE / 2.0, circle_rx, circle_ry, 0.0);
            }
        }

        svg.set_stroke_color(Some(skin.color_arrow));
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

        // Arrowhead rendering for self-messages
        // Ported from ComponentRoseSelfArrow.drawLeftSide/drawRightSide
        let arrow_head = msg.arrow_config().dressing2().head();
        let arrow_part = msg.arrow_config().dressing2().part();
        let is_async_head = arrow_head == plantuml_skin::ArrowHead::Async;
        // x2 += 1 for both Normal+Full and Async (Java code lines 233-236)
        // arrowhead_tip_x = bottom_near - (1 - extraline)
        let extraline_val = if arrow_part == plantuml_skin::ArrowPart::Full
            && arrow_head == plantuml_skin::ArrowHead::Normal
        { 1.0 } else { 0.0 };
        let tip_x = bottom_near - (1.0 - extraline_val);
        let delta_x = 10.0_f64; // getArrowDeltaX()
        let delta_y = 4.0_f64;  // getArrowDeltaY()

        if is_async_head {
            // Async arrowhead: draw lines (not polygon)
            svg.set_stroke_color(Some(skin.color_arrow));
            svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
            if arrow_part != plantuml_skin::ArrowPart::BottomPart {
                // Top half: ULine(-10, -4) for reverse, ULine(10, -4) for non-reverse
                if is_reverse {
                    svg.svg_line(tip_x, y_bottom, tip_x - delta_x, y_bottom - delta_y, 0.0);
                } else {
                    svg.svg_line(tip_x, y_bottom, tip_x + delta_x, y_bottom - delta_y, 0.0);
                }
            }
            if arrow_part != plantuml_skin::ArrowPart::TopPart {
                // Bottom half: ULine(-10, 4) for reverse, ULine(10, 4) for non-reverse
                if is_reverse {
                    svg.svg_line(tip_x, y_bottom, tip_x - delta_x, y_bottom + delta_y, 0.0);
                } else {
                    svg.svg_line(tip_x, y_bottom, tip_x + delta_x, y_bottom + delta_y, 0.0);
                }
            }
        } else {
            // Normal arrowhead: draw filled polygon
            let base_x = tip_x + tip_dir * delta_x;
            let half = delta_y;
            svg.set_fill_color(skin.color_arrow);
            svg.set_stroke_color(Some(skin.color_arrow));
            svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
            if arrow_part == plantuml_skin::ArrowPart::TopPart {
                // Top half only
                svg.svg_polygon(0.0, &[base_x, y_bottom - half, tip_x, y_bottom, base_x, y_bottom]);
            } else if arrow_part == plantuml_skin::ArrowPart::BottomPart {
                // Bottom half only
                svg.svg_polygon(0.0, &[base_x, y_bottom, tip_x, y_bottom, base_x, y_bottom + half]);
            } else {
                // Full arrowhead
                svg.svg_polygon(
                    0.0,
                    &[base_x, y_bottom - half, tip_x, y_bottom, base_x, y_bottom + half, base_x - 4.0 * tip_dir, y_bottom],
                );
            }
        }
    } else if is_return {
        // Return arrow: right-to-left, left-pointing arrowhead
        // tip is 1px past target lifeline, base is 10px further right
        let tip_x = x2 + 1.0;
        let base_x = tip_x + ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        let is_async = msg.arrow_config().is_async2();

        svg.set_fill_color(skin.color_arrow);
        svg.set_stroke_color(Some(skin.color_arrow));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        if is_async {
            // ASYNC (<<-): thin arrowhead as 2 lines, matching Java's drawDressing1
            svg.svg_line(tip_x, y, base_x, y - ARROW_DELTA_Y, 0.0);
            svg.svg_line(tip_x, y, base_x, y + ARROW_DELTA_Y, 0.0);
        } else {
            svg.svg_polygon(
                0.0,
                &[base_x, y - half, tip_x, y, base_x, y + half, base_x - 4.0, y],
            );
        }

        // Line: ASYNC starts at x2 (start=0 in Java), NORMAL starts at x2+5 (start=arrowDeltaX/2)
        let line_start = if is_async { x2 } else { x2 + 5.0 };
        svg.set_stroke_color(Some(skin.color_arrow));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
        svg.svg_line(line_start, y, x1 - 1.0, y, 0.0);
    } else {
        // Normal arrow: left-to-right, solid, right-pointing arrowhead
        let tip_x = x2 - ARROWHEAD_TIP_OFFSET;
        let base_x = tip_x - ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        let is_async = msg.arrow_config().is_async2();

        svg.set_fill_color(skin.color_arrow);
        svg.set_stroke_color(Some(skin.color_arrow));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
        if is_async {
            // ASYNC (->>): thin arrowhead as 2 lines, matching Java's drawDressing2
            svg.svg_line(tip_x, y, base_x, y - ARROW_DELTA_Y, 0.0);
            svg.svg_line(tip_x, y, base_x, y + ARROW_DELTA_Y, 0.0);
        } else {
            svg.svg_polygon(
                0.0,
                &[base_x, y - half, tip_x, y, base_x, y + half, base_x + 4.0, y],
            );
        }

        // Solid line: from source to target. ASYNC extends to x2-1, NORMAL to x2-6.
        let line_end = if is_async { x2 - 1.0 } else { x2 - ARROW_LINE_END_OFFSET };
        svg.set_stroke_color(Some(skin.color_arrow));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
        svg.svg_line(x1, y, line_end, y, 0.0);
    }

    // Message text (only if label is non-empty)
    let label = msg.label();
    if !label.is_empty() {
        // Use wrapped lines if provided, otherwise split by \\n
        let lines: Vec<&str> = if wrapped_lines.is_empty() {
            label.split("\\n").collect()
        } else {
            wrapped_lines.iter().map(String::as_str).collect()
        };
        let text_x = if is_self {
            if is_reverse {
                // <-- : text at component origin + oldPaddingX1
                // Java: x1_global = posC - getCompWidth() - liveDeltaAdj
                // text drawn at x1_global + getOldPaddingX1()
                let label_w = lines.iter().map(|l| bounder.calculate_dimension(font, l).width()).fold(0.0_f64, f64::max);
                // Java: getTextWidth = getPureTextWidth + getOldPaddingX1() + getOldPaddingX2()
                // = pureTextWidth + 7 + 7 (two separate additions, not + 14)
                let comp_width = (label_w + MESSAGE_TEXT_X_OFFSET + MESSAGE_TEXT_X_OFFSET).max(50.0);
                let live_delta_adj = if level_ignore > 0 { ld } else { 0.0 };
                // Java: getMinX() = posC.addFixed(-compWidth - liveDeltaAdj)
                // then text drawn at getMinX() + getOldPaddingX1()
                x1 - comp_width - live_delta_adj + MESSAGE_TEXT_X_OFFSET
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
        // Reverse self-messages with activation levels need HALF_EVEN x/y formatting
        // to match Java's rounding at the .xx5 boundary. The accumulated floating-point
        // sum can produce a double exactly at the nearest double to .xx5 (e.g., 67.5375,
        // 172.5125). HALF_UP would round both up, but the expected SVG rounds 67.5375→67.538
        // (7 odd, up) and 172.5125→172.512 (2 even, down). HALF_EVEN handles both correctly.
        let use_exact = is_self && is_reverse && level_ignore > 0;
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = text_y + (line_idx as f64) * 13.0;
            if is_wrapped {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.is_empty() {
                    let text_w = bounder.calculate_dimension(font, line).width();
                    if use_exact {
                        svg.text_exact(
                            line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                            text_w, &indexmap::IndexMap::new(), None,
                        );
                    } else {
                        svg.text(
                            line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                            text_w, &indexmap::IndexMap::new(), None,
                        );
                    }
                } else {
                    let mut offset_x: f64 = 0.0;
                    for (wi, word) in words.iter().enumerate() {
                        let word_w = bounder.calculate_dimension(font, word).width();
                        if wi > 0 {
                            if use_exact {
                                svg.text_exact(
                                    "\u{00A0}", text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                    0.0, &indexmap::IndexMap::new(), None,
                                );
                            } else {
                                svg.text(
                                    "\u{00A0}", text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                    0.0, &indexmap::IndexMap::new(), None,
                                );
                            }
                        }
                        if use_exact {
                            svg.text_exact(
                                word, text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                word_w, &indexmap::IndexMap::new(), None,
                            );
                        } else {
                            svg.text(
                                word, text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                word_w, &indexmap::IndexMap::new(), None,
                            );
                        }
                        offset_x += word_w;
                    }
                }
            } else {
                let text_w = bounder.calculate_dimension(font, line).width();
                if use_exact {
                    svg.text_exact(
                        line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                        text_w, &indexmap::IndexMap::new(), None,
                    );
                } else {
                    svg.text(
                        line, text_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                        text_w, &indexmap::IndexMap::new(), None,
                    );
                }
            }
        }
    }
}

/// Draws an exo (external) arrow where `?` is used as a message endpoint.
/// Ported from: net/sourceforge/plantuml/sequencediagram/teoz/CommunicationExoTile.java
/// For `TO_RIGHT` (`A->?`): arrow from posC[A] to posC[A] + width
/// For `FROM_LEFT` (`?->E`): arrow from posC[E] - width to posC[E]
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
    skin: &SkinConfig,
) {
    let label = msg.label();
    let pos_c_val = pos_c[p_idx] + x_offset;

    // Compute text width (use wrapped lines if available, otherwise raw label)
    let lines: Vec<&str> = if wrapped_lines.is_empty() {
        label.split("\\n").collect()
    } else {
        wrapped_lines.iter().map(String::as_str).collect()
    };
    let text_w = lines.iter().map(|l| bounder.calculate_dimension(font, l).width()).fold(0.0_f64, f64::max);
    let area_w = text_w + 24.0; // getTextWidth + getArrowDeltaX = (pureText + 7 + 7) + 10

    // x1, x2 are the endpoints of the arrow area
    // x1 is the left edge of the arrow area; x2 is the right edge
    let x1 = match exo_type {
        ExoType::ToRight | ExoType::FromRight => pos_c_val,
        ExoType::FromLeft | ExoType::ToLeft => pos_c_val - area_w,
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
    svg.set_fill_color(skin.color_arrow);
    svg.set_stroke_color(Some(skin.color_arrow));
    svg.set_stroke_width(STROKE_WIDTH_ARROW, None);
    svg.svg_polygon(
        0.0,
        &[base_x, y - half, tip_x, y, base_x, y + half, base_x + 4.0, y],
    );

    // Draw arrow line
    svg.set_stroke_color(Some(skin.color_arrow));
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
/// Uses `HALF_UP` rounding via shortest-representation to match Java's `BigDecimal` behavior.
fn format_number_path(n: f64) -> String {
    let shortest = format!("{n}");
    let rounded = round_half_up_path(&shortest, 3);
    // Strip trailing zeros but keep at least one decimal
    let s = rounded.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

/// Rounds a decimal string to `decimal` fractional digits using `HALF_UP`.
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
    /// For `group #color Title`, the `#color` is stripped and stored in `backcolor`.
    pub comment: String,
    /// Index of the first message in this group.
    pub msg_start: usize,
    /// Index one past the last message in this group.
    pub msg_end: usize,
    /// Nesting level (0 = top-level group, 1 = nested inside one group, etc.).
    pub nesting_level: usize,
    /// Whether this group starts in parallel with the previous tile (`&` prefix).
    pub parallel: bool,
    /// Background color from `#color` in the group comment (e.g., "#FFA" from `group #ffa Title`).
    /// None if no color specified.
    pub backcolor: Option<String>,
}

/// Type of exo (external) arrow, when `?`, `[`, or `]` is used as a message endpoint.
/// Ported from: net/sourceforge/plantuml/sequencediagram/MessageExoType.java
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExoType {
    /// `A->?` or `A->]` — arrow goes right from participant A.
    ToRight,
    /// `?->E` or `[->E` — arrow comes from the left to participant E.
    FromLeft,
    /// `E<-?` or `[<-E` — arrow goes left from participant E.
    ToLeft,
    /// `?<-,E` or `]<-,E` — arrow comes from the right to participant E.
    FromRight,
}

/// Parsed sequence diagram with SVG metadata.
pub struct ParsedSequence {
    /// The sequence diagram.
    pub diagram: SequenceDiagram,
    /// SVG title (from `!option svgTitle`).
    pub svg_title: Option<String>,
    /// SVG description (from `!option svgDesc`).
    pub svg_desc: Option<String>,
    /// `PlantUML` title (from `title` command).
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
    /// Whether each message is hidden (from `[hidden]` arrow style — not drawn but occupies Y space).
    pub msg_hidden: Vec<bool>,
    /// Whether `skin rose` is active (shadow filters, rose colors).
    pub skin_rose: bool,
    /// Arrow color override from `skinparam sequence { ArrowColor <color> }`.
    pub arrow_color: Option<String>,
    /// Header text (from `header` command).
    pub header_text: Option<String>,
    /// Source line number of the header (1-indexed).
    pub header_line: Option<usize>,
    /// Footer text (from `footer` command).
    pub footer_text: Option<String>,
    /// Source line number of the footer (1-indexed).
    pub footer_line: Option<usize>,
    /// Legend text (from `legend` command).
    pub legend_text: Option<String>,
    /// Caption text (from `caption` command).
    pub caption_text: Option<String>,
    /// Source line number of the caption (1-indexed).
    pub caption_line: Option<usize>,
    /// Style rules from `<style>` block: element name → property → value.
    pub style_rules: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

impl ParsedSequence {
    /// Renders this parsed sequence diagram to an SVG string.
    #[must_use]
    pub fn render(&self) -> String {
        render_sequence_svg(
            &self.diagram,
            self.svg_title.as_deref(),
            self.svg_desc.as_deref(),
            self.title.as_deref(),
            self.title_line,
            self.hide_footbox,
            &self.notes,
            &self.groups,
            &self.msg_activates,
            &self.msg_deactivates,
            &self.msg_parallel,
            self.max_message_size,
            &self.msg_exo,
            &self.msg_hidden,
            self.skin_rose,
            self.arrow_color.as_deref(),
            self.header_text.as_deref(),
            self.header_line,
            self.footer_text.as_deref(),
            self.footer_line,
            self.legend_text.as_deref(),
            self.caption_text.as_deref(),
            self.caption_line,
            &self.style_rules,
        )
    }
}
/// Parses a simple `PlantUML` sequence diagram from text, including SVG options.
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
    let mut group_stack: Vec<(usize, String, String, usize, bool, Option<String>)> = Vec::new();
    // skinparam { } block depth: when > 0, skip lines until closing }
    let mut skinparam_depth: u32 = 0;
    let mut msg_activates: Vec<Vec<String>> = Vec::new();
    let mut msg_deactivates: Vec<Vec<String>> = Vec::new();
    let mut header_text: Option<String> = None;
    let mut header_line: Option<usize> = None;
    let mut footer_text: Option<String> = None;
    let mut footer_line: Option<usize> = None;
    let mut legend_text: Option<String> = None;
    let mut _legend_line: Option<usize> = None;
    let mut caption_text: Option<String> = None;
    let mut caption_line: Option<usize> = None;
    let mut style_rules: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
        std::collections::HashMap::new();
    let mut style_element: Option<String> = None;
    let mut msg_parallel: Vec<bool> = Vec::new();
    let mut msg_exo: Vec<Option<ExoType>> = Vec::new();
    let mut msg_hidden: Vec<bool> = Vec::new();
    let mut next_msg_parallel = false;
    let mut autoactivate = false;
    let mut in_note_block: Option<(NotePosition, Vec<String>)> = None;
    let mut max_message_size: Option<f64> = None;
    let mut skin_rose = false;
    let mut arrow_color: Option<String> = None;
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
        // Handle skinparam { } blocks: when depth > 0, skip until closing }
        if skinparam_depth > 0 {
            // Parse Maxmessagesize inside skinparam blocks
            if let Some(rest) = trimmed.strip_prefix("Maxmessagesize ") {
                if let Ok(val) = rest.trim().parse::<f64>() {
                    max_message_size = Some(val);
                }
            }
            if let Some(rest) = trimmed.strip_prefix("ArrowColor ") {
                arrow_color = Some(rest.trim().to_string());
            }
            if trimmed.contains('}') {
                skinparam_depth = skinparam_depth.saturating_sub(1);
            } else if trimmed.ends_with('{') {
                skinparam_depth += 1;
            }
            continue;
        }

        // Skip comment lines (must be before style_element check to avoid
        // catching commented lines like 'skinparam sequence {)
        if trimmed.starts_with('\'') {
            continue;
        }

        // Handle <style> blocks: parse CSS-like rules
        if style_element.is_some() || trimmed.starts_with("</style>") {
            // </style> closes the style block
            if trimmed.starts_with("</style>") {
                style_element = None;
                continue;
            }
            // } closes the current element
            if trimmed.starts_with('}') {
                style_element = None;
                continue;
            }
            // Parse property value pairs inside an element
            if let Some(ref elem) = style_element {
                // Properties are "PropertyName value" or "PropertyName value;"
                let prop_line = trimmed.trim_end_matches(';');
                if let Some((prop, val)) = prop_line.split_once(char::is_whitespace) {
                    style_rules
                        .entry(elem.to_lowercase())
                        .or_default()
                        .insert(prop.to_string(), val.trim().to_string());
                }
            }
            continue;
        }

        // Detect element start inside <style>: "elementname {"
        if trimmed.ends_with('{') && !trimmed.starts_with("skinparam") {
            let elem = trimmed.trim_end_matches('{').trim();
            style_element = Some(elem.to_string());
            continue;
        }

        if trimmed == "skin rose" {
            skin_rose = true;
            continue;
        }
        // Handle !pragma, comments, skin, !theme (ignore — simplified renderer)
        if trimmed.starts_with("!pragma ")
            || trimmed.starts_with('\'')
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
                skinparam_depth += 1;
            }
            continue;
        }

        // Handle <style> block start
        if trimmed.starts_with("<style>") {
            continue;
        }

        // Handle "end" — close the current group (pop from stack)
        if trimmed == "end" {
            if let Some((start, gtype, comment, level, parallel, backcolor)) = group_stack.pop() {
                groups.push(GroupInfo {
                    group_type: gtype,
                    comment,
                    msg_start: start,
                    msg_end: msg_count,
                    nesting_level: level,
                    parallel,
                    backcolor,
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
            if let Some((start, gtype, comment, level, parallel, backcolor)) = group_stack.pop() {
                groups.push(GroupInfo {
                    group_type: gtype,
                    comment,
                    msg_start: start,
                    msg_end: msg_count,
                    nesting_level: level,
                    parallel,
                    backcolor,
                });
                // Start new else section at the same nesting level (inherit parallel flag)
                group_stack.push((msg_count, "else".to_string(), else_comment, level, parallel, None));
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
            let raw_comment = if trimmed.len() > group_keyword.len() {
                trimmed[group_keyword.len()..].trim().to_string()
            } else {
                String::new()
            };
            // Parse leading `#color` from the comment (e.g., `group #ffa Title`).
            // Java's CommandGrouping regex captures `#color` as COLORS[0] separately.
            // The color becomes the group's background; the rest is the title.
            let (comment, backcolor) = if let Some(rest) = raw_comment.strip_prefix('#') {
                let color_end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
                let color = &raw_comment[..=color_end]; // includes '#'
                let title = rest[color_end..].trim().to_string();
                (title, Some(color.to_ascii_uppercase()))
            } else {
                (raw_comment, None)
            };
            let is_parallel = next_msg_parallel;
            next_msg_parallel = false;
            group_stack.push((msg_count, group_keyword.to_string(), comment, nesting_level, is_parallel, backcolor));
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
            // In Java, activate is attached to the previous message if it deals with
            // this participant (SequenceDiagram.activate: lastEventWithDeactivate.dealWith(p)).
            // When attached, the LifeEvent is inline (uses message's Y for activation).
            // When not attached (participant not in last message), it's standalone.
            let mut attached = false;
            if msg_count > 0 {
                let prev_idx = msg_count - 1;
                if let (Some(ref p1), Some(ref p2)) = (&last_p1, &last_p2) {
                    if p1 == pname || p2 == pname {
                        while msg_activates.len() <= prev_idx {
                            msg_activates.push(Vec::new());
                        }
                        msg_activates[prev_idx].push(pname.to_string());
                        diagram.activate_inline(&p, LifeEventType::Activate, prev_idx);
                        attached = true;
                    }
                }
            }
            if !attached {
                diagram.activate(&p, LifeEventType::Activate);
            }
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("deactivate ") {
            let pname = name.trim();
            let p = diagram.get_or_create_participant(pname);
            let mut attached = false;
            if msg_count > 0 {
                let prev_idx = msg_count - 1;
                if let (Some(ref p1), Some(ref p2)) = (&last_p1, &last_p2) {
                    if p1 == pname || p2 == pname {
                        while msg_deactivates.len() <= prev_idx {
                            msg_deactivates.push(Vec::new());
                        }
                        msg_deactivates[prev_idx].push(pname.to_string());
                        diagram.activate_inline(&p, LifeEventType::Deactivate, prev_idx);
                        attached = true;
                    }
                }
            }
            if !attached {
                diagram.activate(&p, LifeEventType::Deactivate);
            }
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("destroy ") {
            let pname = name.trim();
            let p = diagram.get_or_create_participant(pname);
            let mut attached = false;
            if msg_count > 0 {
                let prev_idx = msg_count - 1;
                if let (Some(ref p1), Some(ref p2)) = (&last_p1, &last_p2) {
                    if p1 == pname || p2 == pname {
                        while msg_deactivates.len() <= prev_idx {
                            msg_deactivates.push(Vec::new());
                        }
                        msg_deactivates[prev_idx].push(pname.to_string());
                        diagram.activate_inline(&p, LifeEventType::Destroy, prev_idx);
                        attached = true;
                    }
                }
            }
            if !attached {
                diagram.activate(&p, LifeEventType::Destroy);
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
        // Handle "autoactivate on/off"
        if trimmed == "autoactivate on" {
            autoactivate = true;
            continue;
        }
        if trimmed == "autoactivate off" {
            autoactivate = false;
            continue;
        }

        // Handle "title" command
        if let Some(val) = trimmed.strip_prefix("title ") {
            title = Some(val.trim().to_string());
            title_line = Some(line_num - startuml_line.unwrap_or(0));
            continue;
        }

        // Handle "header", "footer", "legend", "caption" commands
        if let Some(val) = trimmed.strip_prefix("header ") {
            header_text = Some(val.trim().to_string());
            header_line = Some(line_num - startuml_line.unwrap_or(0));
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("footer ") {
            footer_text = Some(val.trim().to_string());
            footer_line = Some(line_num - startuml_line.unwrap_or(0));
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("legend ") {
            legend_text = Some(val.trim().to_string());
            _legend_line = Some(line_num - startuml_line.unwrap_or(0));
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("caption ") {
            caption_text = Some(val.trim().to_string());
            caption_line = Some(line_num - startuml_line.unwrap_or(0));
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
                        let display = &name[1..=end];
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

        if let Some((p1_code, p2_code, label, arrow_config, inline_activate, inline_deactivate, inline_destroy)) = parse_arrow_line(trimmed) {
            let is_reverse_arrow = arrow_config.is_reverse_define();
            // Detect exo arrows: ?, [, or ] as message endpoint.
            let is_left_exo = p1_code == "?" || p1_code == "[" || p1_code == "]";
            let is_right_exo = p2_code == "?" || p2_code == "]" || p2_code == "[";
            let exo = if is_left_exo {
                if p1_code == "]" {
                    if is_reverse_arrow { Some(ExoType::ToRight) } else { Some(ExoType::FromRight) }
                } else if is_reverse_arrow { Some(ExoType::ToLeft) } else { Some(ExoType::FromLeft) }
            } else if is_right_exo {
                if p2_code == "[" {
                    if is_reverse_arrow { Some(ExoType::FromLeft) } else { Some(ExoType::ToLeft) }
                } else if is_reverse_arrow { Some(ExoType::FromRight) } else { Some(ExoType::ToRight) }
            } else {
                None
            };
            // For exo arrows, use the real participant as both p1 and p2
            let (real_p1, real_p2) = match exo {
                Some(ExoType::ToRight | ExoType::FromRight) => {
                    let real_code = &p1_code;
                    last_p1 = Some(real_code.clone());
                    last_p2 = Some(p2_code.clone());
                    let p1 = diagram.get_or_create_participant(real_code);
                    (p1.clone(), p1)
                }
                Some(ExoType::FromLeft | ExoType::ToLeft) => {
                    let real_code = &p2_code;
                    last_p1 = Some(p1_code.clone());
                    last_p2 = Some(real_code.clone());
                    let p2 = diagram.get_or_create_participant(real_code);
                    (p2.clone(), p2)
                }
                None => {
                    last_p1 = Some(p1_code.clone());
                    last_p2 = Some(p2_code.clone());
                    let p1 = diagram.get_or_create_participant(&p1_code);
                    let p2 = diagram.get_or_create_participant(&p2_code);
                    // Java's CommandArrow swaps p1/p2 for reverseDefine arrows:
                    // p1 = PART2 (right), p2 = PART1 (left).
                    if is_reverse_arrow { (p2, p1) } else { (p1, p2) }
                }
            };
            let msg_num = diagram.get_next_message_number();
            let is_dotted = arrow_config.is_dotted();
            let msg = Message::new(real_p1, real_p2, label, arrow_config, msg_num);
            diagram.add_message(msg);
            // Apply inline activation/deactivation and autoactivate.
            // Java's CommandArrow swaps p1/p2 for reverseDefine arrows:
            //   p1 = PART2 (right), p2 = PART1 (left)
            // So ++ activates p2 (left), -- deactivates p1 (right), !! destroys p2 (left).
            // In our code, p1_code is always left, p2_code is always right.
            // For reverse arrows, swap: ++ → activate p1_code, -- → deactivate p2_code, !! → destroy p1_code.
            let mut acts = Vec::new();
            let mut deacts = Vec::new();
            let mut dests = Vec::new();
            if inline_deactivate {
                if is_reverse_arrow { deacts.push(p2_code.clone()); } else { deacts.push(p1_code.clone()); }
            }
            if inline_activate {
                if is_reverse_arrow { acts.push(p1_code.clone()); } else { acts.push(p2_code.clone()); }
            }
            if inline_destroy {
                if is_reverse_arrow { dests.push(p1_code.clone()); } else { dests.push(p2_code.clone()); }
            }
            // Autoactivate: solid arrow activates receiver, dotted deactivates sender
            if autoactivate && exo.is_none() {
                if is_dotted {
                    if is_reverse_arrow {
                        deacts.push(p2_code.clone());
                    } else {
                        deacts.push(p1_code.clone());
                    }
                } else if is_reverse_arrow {
                    acts.push(p1_code.clone());
                } else {
                    acts.push(p2_code.clone());
                }
            }
            // Create LifeEvents for inline and autoactivate activations/deactivations
            for code in &acts {
                let p = diagram.participants().iter()
                    .find(|p| p.code() == code)
                    .cloned();
                if let Some(p) = p {
                    diagram.activate_inline(&p, LifeEventType::Activate, msg_count);
                }
            }
            for code in &deacts {
                let p = diagram.participants().iter()
                    .find(|p| p.code() == code)
                    .cloned();
                if let Some(p) = p {
                    diagram.activate_inline(&p, LifeEventType::Deactivate, msg_count);
                }
            }
            // Create LifeEvents for inline destroy (!!)
            for code in &dests {
                let p = diagram.participants().iter()
                    .find(|p| p.code() == code)
                    .cloned();
                if let Some(p) = p {
                    diagram.activate_inline(&p, LifeEventType::Destroy, msg_count);
                }
            }
            msg_activates.push(acts);
            let mut all_deacts = deacts.clone();
            all_deacts.extend(dests.iter().cloned());
            msg_deactivates.push(all_deacts);
            let in_parallel_group = group_stack.iter().any(|(_, _, _, _, is_par, _)| *is_par);
            msg_parallel.push(next_msg_parallel || in_parallel_group);
            msg_exo.push(exo);
            msg_hidden.push(trimmed.contains("[hidden]"));
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
            msg_hidden,
            skin_rose,
            arrow_color,
            header_text,
            header_line,
            footer_text,
            footer_line,
            legend_text,
            caption_text,
            caption_line,
            style_rules,
        })
    }
}

/// Parses an arrow line like "Alice -> Bob : hello" or "Test --> Test: Text".
/// Handles common arrow types: ->, -->, <-, <--, ->>, <<-, \\--, \\-.
/// Checks if a byte is an arrow dressing character (o, x, <, >, /, \).
fn is_dressing_char(b: u8) -> bool {
    matches!(b, b'o' | b'O' | b'x' | b'X' | b'<' | b'>' | b'/' | b'\\')
}

/// Builds an `ArrowConfiguration` from parsed dressings and body length.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/command/CommandArrow.java` (executeArg)
fn build_arrow_config(
    dressing1: &str,
    dressing2: &str,
    body_len: usize,
) -> plantuml_skin::ArrowConfiguration {
    use plantuml_skin::{ArrowBody, ArrowDecoration, ArrowHead, ArrowPart};

    let has_dir1 = dressing1.contains('<') || dressing1.contains('\\') || dressing1.contains('/');
    let has_dir2 = dressing2.contains('>') || dressing2.contains('\\') || dressing2.contains('/');
    let x1 = dressing1.contains('x');
    let x2 = dressing2.contains('x');

    let reverse_define = if has_dir2 || (x1 && x2) {
        false
    } else {
        has_dir1
    };

    let dotted = body_len > 1;

    let mut config = if has_dir1 && has_dir2 {
        plantuml_skin::ArrowConfiguration::with_direction_both()
    } else {
        plantuml_skin::ArrowConfiguration::with_direction_normal()
    };

    if dotted {
        config = config.with_body(ArrowBody::Dotted);
    }

    let (circle_at_start, circle_at_end, sync1, sync2) = if reverse_define {
        (
            dressing2.contains('o'),
            dressing1.contains('o'),
            dressing2.contains(">>") || dressing2.contains("\\\\") || dressing2.contains("//"),
            dressing1.contains("<<") || dressing1.contains("\\\\") || dressing1.contains("//"),
        )
    } else {
        (
            dressing1.contains('o'),
            dressing2.contains('o'),
            dressing1.contains("<<") || dressing1.contains("\\\\") || dressing1.contains("//"),
            dressing2.contains(">>") || dressing2.contains("\\\\") || dressing2.contains("//"),
        )
    };

    if sync1 {
        config = config.with_head1(ArrowHead::Async);
    }
    if sync2 {
        config = config.with_head2(ArrowHead::Async);
    }

    if dressing2.contains('\\') || dressing1.contains('/') {
        config = config.with_part(ArrowPart::TopPart);
    }
    if dressing2.contains('/') || dressing1.contains('\\') {
        config = config.with_part(ArrowPart::BottomPart);
    }

    if circle_at_end {
        config = config.with_decoration2(ArrowDecoration::Circle);
    }
    if circle_at_start {
        config = config.with_decoration1(ArrowDecoration::Circle);
    }

    if reverse_define {
        if x1 {
            config = config.with_head2(ArrowHead::CrossX);
        }
        if x2 {
            config = config.with_head1(ArrowHead::CrossX);
        }
    } else {
        if x1 {
            config = config.with_head1(ArrowHead::CrossX);
        }
        if x2 {
            config = config.with_head2(ArrowHead::CrossX);
        }
    }

    if reverse_define {
        config = config.reverse_define();
    }

    config
}

/// Parses a `PlantUML` sequence arrow line, extracting participants, label, and arrow configuration.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/command/CommandArrow.java`
///
/// Returns `(p1_code, p2_code, label, arrow_config, inline_activate, inline_deactivate, inline_destroy)`.
fn parse_arrow_line(
    line: &str,
) -> Option<(
    String,
    String,
    String,
    plantuml_skin::ArrowConfiguration,
    bool,
    bool,
    bool,
)> {
    // Strip [hidden] and other [style] modifiers from arrow notation
    // e.g., "B -[hidden]-> C" becomes "B -> C"
    let line = if let Some(bracket_start) = line.find("-[") {
        if let Some(bracket_end) = line[bracket_start..].find(']') {
            let before = &line[..=bracket_start]; // includes the first '-'
            let after = &line[bracket_start + bracket_end + 1..];
            format!("{before}{after}")
        } else {
            line.to_string()
        }
    } else {
        line.to_string()
    };
    let line = line.as_str();
    let bytes = line.as_bytes();

    // Find all runs of '-' characters as candidate arrow bodies
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'-' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'-' {
                i += 1;
            }
            candidates.push((start, i));
        } else {
            i += 1;
        }
    }
    // Try longest bodies first (greedy: prefer dotted `--` over solid `-`)
    candidates.sort_by(|a, b| (b.1 - b.0).cmp(&(a.1 - a.0)));

    for &(body_start, body_end) in &candidates {
        let body_len = body_end - body_start;

        // Parse dressing1: scan backward from body_start
        let text_before = line[..body_start].trim_end();
        let tb_bytes = text_before.as_bytes();
        let mut d1_len = 0;
        while d1_len < tb_bytes.len() && is_dressing_char(tb_bytes[tb_bytes.len() - 1 - d1_len]) {
            d1_len += 1;
        }
        let (dressing1, p1_code) = if d1_len > 0 && d1_len < tb_bytes.len() {
            let before_dressing = &text_before[..text_before.len() - d1_len];
            if before_dressing.ends_with(|c: char| c.is_whitespace()) {
                let d1 = &text_before[text_before.len() - d1_len..];
                (d1.to_lowercase(), before_dressing.trim().to_string())
            } else {
                (String::new(), text_before.trim().to_string())
            }
        } else {
            (String::new(), text_before.trim().to_string())
        };
        if p1_code.is_empty() {
            continue;
        }

        // Parse dressing2: scan forward from body_end
        let text_after = &line[body_end..];
        let ta_bytes = text_after.as_bytes();
        let mut d2_len = 0;
        while d2_len < ta_bytes.len() && is_dressing_char(ta_bytes[d2_len]) {
            d2_len += 1;
        }
        let (dressing2, rest) = if d2_len > 0 {
            // Dressing2 chars (>, >>, /, \, o, x) can be directly followed by
            // the participant name without whitespace (e.g., "Bob->Bob").
            // Only "o" or "x" alone (no direction indicator) require whitespace,
            // but we accept all cases and let validation filter invalid ones.
            (text_after[..d2_len].to_lowercase(), &text_after[d2_len..])
        } else {
            (String::new(), text_after)
        };

        // Validate: must have a direction indicator or x in dressings
        let has_dir1 = dressing1.contains('<') || dressing1.contains('\\') || dressing1.contains('/');
        let has_dir2 = dressing2.contains('>') || dressing2.contains('\\') || dressing2.contains('/');
        let has_x = dressing1.contains('x') || dressing2.contains('x');
        if !has_dir1 && !has_dir2 && !has_x {
            continue;
        }

        // Parse p2_part and label from rest
        let (p2_part_raw, label) = if let Some(colon_pos) = rest.find(':') {
            (
                rest[..colon_pos].trim().to_string(),
                rest[colon_pos + 1..].trim().to_string(),
            )
        } else {
            (rest.trim().to_string(), String::new())
        };
        if p2_part_raw.is_empty() {
            continue;
        }

        // Strip #color modifier and parse inline activation markers
        let mut p2_part = p2_part_raw.clone();
        if let Some(hash_pos) = p2_part.find('#') {
            p2_part = p2_part[..hash_pos].trim().to_string();
        }
        let mut inline_activate = false;
        let mut inline_deactivate = false;
        let mut inline_destroy = false;
        if p2_part.ends_with("--++") {
            inline_deactivate = true;
            inline_activate = true;
            p2_part = p2_part[..p2_part.len() - 4].trim().to_string();
        } else if p2_part.ends_with("++--") {
            inline_activate = true;
            inline_deactivate = true;
            p2_part = p2_part[..p2_part.len() - 4].trim().to_string();
        } else if p2_part.ends_with("!!") {
            inline_destroy = true;
            p2_part = p2_part[..p2_part.len() - 2].trim().to_string();
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

        let config = build_arrow_config(&dressing1, &dressing2, body_len);
        return Some((
            p1_code,
            p2_part,
            label,
            config,
            inline_activate,
            inline_deactivate,
            inline_destroy,
        ));
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

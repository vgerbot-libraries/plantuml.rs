//! Sequence diagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/sequencediagram/teoz/` package
//!
//! Minimal renderer for simple `A -> B : msg` sequence diagrams.
//! Full Teoz layout engine will be ported incrementally.

use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::{FontStyle, UFont};
use plantuml_klimt::string_bounder_svg::StringBounderSvg;
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
/// Java: padding.top from style = 9.0769 for font 14.
const PADDING_TOP: f64 = 9.0769;
/// Text block height for font size 14 (getTextHeight = textDim.height + padding.top + padding.bottom).
/// Java: 16.068 (ascent+descent) + 9.0769 + 7.923 = 33.0679.
const TEXT_BLOCK_HEIGHT: f64 = 33.0679;
/// Extra height from `getPreferredHeight` formula (`+ 1`).
const HEIGHT_EXTRA: f64 = 1.0;
/// Spacing between participants (from LivingSpaces.addConstraints).
const PARTICIPANT_SPACING: f64 = 10.0;
/// Page margin (UTranslate(5, 5) in `SequenceDiagramFileMakerTeoz`).
const PAGE_MARGIN: f64 = 5.0;
/// Starting Y offset (0 — page margin already provides the 5px offset).
const STARTING_Y: f64 = 0.0;
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
/// Activation bar offset from center (posC - width/2 = posC - 4).
const ACTIVATION_BAR_OFFSET: f64 = 4.0;
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
/// AWT line height for font 13 (getStringBounds height with FRACTIONALMETRICS_ON).
const MSG_LINE_HEIGHT: f64 = 17.706;
/// AWT ascent for font 13.
const ASCENT_13: f64 = 13.897;
/// Message padding Y (getPaddingY = 4 from AbstractComponentRoseArrow).
const MSG_PADDING_Y: f64 = 4.0;
/// Message starting Y offset (startingY=8 in PlayingSpace + 2px from yOrigin).
const MSG_STARTING_Y: f64 = 10.0;
/// Message text padding top (from ClockwiseTopRightBottomLeft(1, 7, 1, 7)).
const MSG_PADDING_TOP: f64 = 1.0;
/// Message text padding bottom.
const MSG_PADDING_BOTTOM: f64 = 1.0;
/// Message text X offset from source center.
const MESSAGE_TEXT_X_OFFSET: f64 = 7.0;
/// Base arrow Y offset from lifeline start (paddingY + arrowDeltaY).
const ARROW_Y_BASE: f64 = 14.0;
/// Base lifeline height (without message text).
const LIFELINE_HEIGHT_BASE: f64 = 32.0;
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
/// Extra width of the bordered title block over its bold text width.
///
/// Derived from `Style.createTextBlockBordered` (padding 5+5, `TextBlockBordered`
/// +1 and the style margin); grounded empirically on the 1.2026.6 jar at 14px.
const TITLE_BLOCK_W_EXTRA: f64 = 11.0;
/// Vertical offset the bordered title block pushes participants down by.
///
/// Title block height (`DecorateEntityImage.addTop`): style margin 5+5,
/// border padding 5+5, text height 19.0679, +1.
const TITLE_SHIFT_Y: f64 = 40.0679;
/// Baseline offset of the title text inside its bordered block
/// (rect top + 7.0769 top padding + 12.889 ascent).
const TITLE_TEXT_OFFSET: f64 = 19.9659;

/// Per-character advances (px) for bold 14px sans-serif, measured from the
/// PlantUML jar `textLength` attributes (single-char strings plus trailing
/// context for punctuation); bold uses per-glyph advances different from the
/// plain tables in `plantuml-klimt`.
const BOLD14_WIDTHS: &[(&str, f64)] = &[
    (" ", 3.6399), ("!", 4.0040), ("\"", 6.6080), ("#", 9.0440),
    ("$", 8.0080), ("%", 12.6140), ("&", 10.5), ("'", 3.7240),
    ("(", 4.7460), (")", 4.7460), ("*", 7.63), ("+", 8.0080),
    (",", 3.99), ("-", 4.5080), (".", 3.99), ("/", 5.7820),
    ("0", 8.0080), ("1", 8.0080), ("2", 8.0080), ("3", 8.0080),
    ("4", 8.0080), ("5", 8.0080), ("6", 8.0080), ("7", 8.0080),
    ("8", 8.0080), ("9", 8.0080), (":", 3.99), (";", 3.99),
    ("<", 8.0080), ("=", 8.0080), (">", 8.0080), ("?", 6.6780),
    ("@", 12.5580), ("A", 9.66), ("B", 9.4080), ("C", 8.9180),
    ("D", 10.36), ("E", 7.84), ("F", 7.6860), ("G", 10.1360),
    ("H", 10.71), ("I", 5.4460), ("J", 4.6340), ("K", 9.2960),
    ("L", 7.91), ("M", 13.2020), ("N", 11.3820), ("O", 11.1440),
    ("P", 8.7920), ("Q", 11.1440), ("R", 9.24), ("S", 7.7140),
    ("T", 8.1060), ("U", 10.5840), ("V", 9.1), ("W", 13.5379),
    ("X", 9.3380), ("Y", 8.7360), ("Z", 8.1060), ("[", 4.6340),
    ("\\", 5.7820), ("]", 4.6340), ("^", 8.0080), ("_", 5.7540),
    ("`", 5.0680), ("a", 8.4560), ("b", 8.8620), ("c", 7.1960),
    ("d", 8.8620), ("e", 8.2740), ("f", 5.4180), ("g", 8.8620),
    ("h", 9.1980), ("i", 4.27), ("j", 4.27), ("k", 8.68),
    ("l", 4.27), ("m", 13.7479), ("n", 9.1980), ("o", 8.6660),
    ("p", 8.8620), ("q", 8.8620), ("r", 6.3560), ("s", 6.9580),
    ("t", 6.0760), ("u", 9.1980), ("v", 7.9660), ("w", 11.9839),
    ("x", 8.0920), ("y", 7.9660), ("z", 6.8320), ("{", 5.5160),
    ("|", 7.7140), ("}", 5.5160), ("~", 8.0080),
];

/// Returns the bold 14px advance width of `text`. Uses the exact AWT per-glyph
/// table (f32 accumulation, matching the JAR's TextLayout), falling back to
/// [`BOLD14_WIDTHS`] for text containing non-ASCII characters.
fn bold14_width(text: &str) -> f64 {
    if let Some(w) = StringBounderSvg::exact_ascii_width(14.0, true, text) {
        return w;
    }
    text.chars()
        .map(|c| {
            let key = &c.to_string()[..];
            BOLD14_WIDTHS
                .iter()
                .find(|(k, _)| *k == key)
                .map_or(8.0, |(_, w)| *w)
        })
        .sum()
}
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
/// Extra participant-box outer margin (Puma2 `DrawableSetInitializer`: `outMargin = 5`).
const NOTE_OUT_MARGIN: f64 = 5.0;
/// Note fold corner size (from `Opale.cornersize`).
const NOTE_CORNERSIZE: f64 = 10.0;
/// Note horizontal padding (from `Rose.paddingX`).
const NOTE_PADDING_X: f64 = 5.0;
/// Note old padding X1 (from `ClockwiseTopRightBottomLeft` left, non-CENTER alignment).
const NOTE_OLD_PADDING_X1: f64 = 6.0;
/// Note old padding X2 (from `ClockwiseTopRightBottomLeft` right, non-CENTER alignment).
const NOTE_OLD_PADDING_X2: f64 = 15.0;
/// Total vertical extent of a standalone one-line note tile in the Y gauge.
/// `ComponentRoseNote.getPreferredHeight` = textBlock height (17.706) + 2*paddingY(5).
/// Ported from: skin/rose/ComponentRoseNote.java
const STANDALONE_NOTE_H: f64 = 37.706;
/// Each additional text line adds the line height to a standalone note tile.
const STANDALONE_NOTE_LINE_H: f64 = 13.0;

// ── Colors ───────────────────────────────────────────────────────────────

const COLOR_BACK: &str = "#E2E2F0";
const COLOR_STROKE: &str = "#181818";
const COLOR_TEXT: &str = "#000000";
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
const GROUP_HEADER_HEIGHT: f64 = 20.706;
/// Group header offset added to first message Y inside a group.
/// = header preferred height (19.706) … measured: first group message is
/// 26.706 below the ungrouped position (jar-verified).
const GROUP_HEADER_OFFSET: f64 = 26.706;
/// Extra header height for partitions: `TITLE_VPAD`*2 - (`GROUP_HEADER_HEIGHT` - `font_height`) = 4*2 - (15-13) = 6.
/// Partition titles are drawn directly on the frame (no tab), with 4px padding above and below.
const PARTITION_HEADER_EXTRA: f64 = 6.0;
/// Gap between consecutive group frames = 2*`EXTERNAL_MARGINY` + `MARGINY_MAGIC/2`.
const GROUP_GAP: f64 = 18.0;
/// Else tile height in the legacy (Puma) renderer the jar uses.
/// Jar-verified: the else section's chained advance is 16.982 (not the
/// 17.0 the Teoz port uses).
const ELSE_TILE_HEIGHT: f64 = 16.982;
// ── Divider (`== label ==`) geometry, legacy Puma ─────────────────────
/// Bold13 label rect height = component text height.
const DIVIDER_LABEL_H: f64 = 25.706;
/// Added to text height for the tile's preferred height (`+20`).
const DIVIDER_TILE_EXTRA: f64 = 20.0;
/// Full divider tile height = label_h + 20 (jar-verified advance).
const DIVIDER_TILE_H: f64 = DIVIDER_LABEL_H + DIVIDER_TILE_EXTRA;
/// Label rect width = rendered bold textLength + this (jar-verified).
const DIVIDER_LABEL_PAD_X: f64 = 17.38;
/// Band rect local Y inside the tile.
const DIVIDER_BAND_Y: f64 = 21.853;
/// Band rect height.
const DIVIDER_BAND_H: f64 = 3.0;
/// Label rect local Y inside the tile.
const DIVIDER_LABEL_Y: f64 = 10.0;
/// Label text baseline local Y inside the tile.
const DIVIDER_TEXT_BASELINE: f64 = 27.897;
/// Divider band/label background fill.
const COLOR_DIVIDER_BG: &str = "#EEEEEE";
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
/// Visual folded-tab height (jar-verified): tab bottom sits 19.706 below fy.
const GROUP_TAB_HEIGHT: f64 = 19.706;
/// Tab title baseline offset within the tab (13px bold, jar-verified).
const GROUP_TAB_TITLE_BASELINE: f64 = 14.897;
/// Group header fill color.
const COLOR_GROUP_HEADER: &str = "#EEEEEE";
/// Group frame stroke color.
const COLOR_GROUP_STROKE: &str = "#000000";

/// Wraps message text to fit within `max_width`, splitting by words.
/// Ported from Java's `StringBounder` word-wrap logic for `MaxMessageSize`.
fn wrap_message_text(
    bounder: &dyn StringBounder,
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
/// Draws every explicit activation (live) box, outer level first.
///
/// Java emits this set twice (`LiveBoxes.drawBoxes` is invoked once over the
/// full height behind the lifelines and a second time after the heads): once
/// before the lifeline group and again after the heads, before the messages.
/// Each box is wrapped in `<g><title>..</title><rect/></g>`.
fn draw_activation_bars(
    svg: &mut SvgGraphics,
    activations: &[(usize, f64, f64, i32)],
    participants: &[plantuml_sequence::Participant],
    pos_c: &[f64],
    x_offset: f64,
) {
    // JAR LifeLine.drawU iterates participants in insertion order, emitting
    // each participant's closed segments in ascending level order.
    let mut ordered: Vec<(usize, f64, f64, i32)> = activations.to_vec();
    ordered.sort_by_key(|&(pi, _, _, level)| (pi, level));
    for &(pi, start_y, end_y, level) in &ordered {
        let cx = pos_c[pi] + x_offset;
        let level_dx = (level as f64 - 1.0) * ACTIVATION_BAR_EXPLICIT_OFFSET;
        svg.open_group(None);
        svg.title(participants[pi].display());
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some(COLOR_STROKE));
        svg.set_stroke_width(1.0, None);
        svg.set_filter(None);
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
}

#[allow(clippy::fn_params_excessive_bools)]
pub fn render_sequence_svg(
    diagram: &SequenceDiagram,
    svg_title: Option<&str>,
    svg_desc: Option<&str>,
    title: Option<&str>,
    title_line: Option<usize>,
    hide_footbox: bool,
    notes: &[NoteInfo],
    groups: &[GroupInfo],
    dividers: &[DividerInfo],
    autonumber: bool,
    msg_activates: &[Vec<String>],
    msg_deactivates: &[Vec<String>],
    msg_parallel: &[bool],
    max_message_size: Option<f64>,
    msg_exo: &[Option<ExoType>],
    msg_hidden: &[bool],
    skin_rose: bool,
    teoz: bool,
    arrow_color: Option<&str>,
    header_text: Option<&str>,
    header_line: Option<usize>,
    footer_text: Option<&str>,
    footer_line: Option<usize>,
    legend_text: Option<&str>,
    caption_text: Option<&str>,
    caption_line: Option<usize>,
    style_rules: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    participant_source_lines: &[Option<usize>],
    msg_source_lines: &[Option<usize>],
    teoz_toks: &[TeozTok],
) -> String {
    let bounder = StringBounderSvg::new(FileFormat::Svg);
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

    // The teoz engine (`!pragma teoz true`) has a distinct coordinate and
    // emission model (no 10px inter-participant gap, different Y gauge,
    // batched bare-sibling emission). Keep it fully separate so the legacy
    // path stays byte-identical.
    if teoz {
        return render_sequence_svg_teoz(
            diagram,
            svg_title,
            svg_desc,
            title,
            title_line,
            hide_footbox,
            notes,
            groups,
            dividers,
            autonumber,
            msg_activates,
            msg_deactivates,
            msg_parallel,
            max_message_size,
            msg_exo,
            msg_hidden,
            skin_rose,
            teoz,
            arrow_color,
            header_text,
            header_line,
            footer_text,
            footer_line,
            legend_text,
            caption_text,
            caption_line,
            style_rules,
            participant_source_lines,
            msg_source_lines,
            teoz_toks,
        );
    }

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
    // Own preferred head component heights (each includes its trailing pixel).
    // Box: ComponentRoseParticipant height = rect 33.0679 + 1 = 34.0679.
    // Actor: ComponentRoseActor.getPreferredHeight = stickman(60) + text(19.0679) = 79.0679.
    // Heads are bottom aligned (LivingSpaces.drawHeads uses VerticalAlignment.BOTTOM)
    // and the lifeline starts at origin + max(own heights).
    const ACTOR_OWN_HEIGHT: f64 = 79.0679;
    // Actor text block: preferred height 19.0679; baseline offset 14.9659.
    const ACTOR_TEXT_HEIGHT: f64 = 19.0679;
    const ACTOR_TEXT_BASELINE: f64 = 14.9659;
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
    let mut own_heights: Vec<f64> = Vec::with_capacity(participants.len());
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
            own_heights.push(ACTOR_OWN_HEIGHT);
        } else {
            // Java: getPreferredWidth = getTextWidth + margin.left + margin.right + deltaShadow
            // getTextWidth = text_width + padding.left + padding.right; padding=(5,7,5,7)
            // Must add padding.left and padding.right separately to match Java's float addition order
            let hw = text_w + PADDING_H + PADDING_H;
            head_widths.push(hw);
            preferred_widths.push(hw + delta_shadow);
            own_heights.push(TEXT_BLOCK_HEIGHT + HEIGHT_EXTRA);
        }
    }
    // Heads are bottom aligned to a common height: LivingSpaces.getHeadHeight
    // returns max(own preferred heights). Each component's preferred height
    // already includes its trailing pixel (box = rect 33.0679 + 1; actor =
    // stickman 60 + text 19.0679), so the lifeline offset is exactly common_own.
    let common_own = own_heights
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let head_layout_height = common_own;
    // head_rect_height: tail contribution to footbox_bottom. The tail is placed
    // one pixel above footbox_y (tail_y = footbox_y - 1), so a box tail adds its
    // rectangle height (33.0679) while an actor tail adds its full 79.0679.
    let has_actor = is_actor.iter().any(|&a| a);
    let head_rect_height = if has_actor {
        ACTOR_OWN_HEIGHT
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
    // Participants destroyed at some point in the diagram. Post-destroy they
    // have no live segment, so forward self tiles reserve level 0.
    let mut destroyed_participants: std::collections::HashSet<usize> = std::collections::HashSet::new();
    // Track activation levels at each message for spacing constraints
    let mut msg_p1_levels: Vec<i32> = Vec::new();
    let mut msg_p2_levels: Vec<i32> = Vec::new();
    // Inline destroy participants per message. A destroy must NOT lower the
    // self tile's considered level (JAR SMALLER applies after the tile), so
    // level_considere subtracts real deactivates only, never destroys.
    let mut msg_destroy_codes: Vec<Vec<String>> = Vec::new();
    for ev in diagram.events() {
        if let SequenceEvent::LifeEvent(le) = ev {
            if le.is_destroy() {
                if let Some(m) = le.message_index() {
                    while msg_destroy_codes.len() <= m {
                        msg_destroy_codes.push(Vec::new());
                    }
                    msg_destroy_codes[m].push(le.participant().code().to_string());
                }
            }
        }
    }
    {
        let mut mi = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                let is_self = msg_exo.get(mi).copied().flatten().is_none() && msg.p1().code() == msg.p2().code();
                if is_self {
                    let p_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let level_ignore = pre_participant_levels.get(p_idx).copied().unwrap_or(0);
                    let future_acts = msg_activates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
                    let future_deacts_all = msg_deactivates.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
                    let future_destroys = msg_destroy_codes.get(mi).map_or(0, |v| v.iter().filter(|c| *c == msg.p1().code()).count() as i32);
                    // JAR: a destroy does not lower the self tile's considered
                    // level (SMALLER applies after the tile); only real deactivates.
                    let future_deacts = (future_deacts_all - future_destroys).max(0);
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
                let mut p1_level_for_constraint = p1_level + inline_acts_p1;
                let mut p2_level_for_constraint = p2_level + inline_acts_p2;
                // A participant already destroyed before this message has no live
                // segment at the message Y (JAR getLevelAt returns 0 post-destroy),
                // so its activation contributes no live-segment width.
                if destroyed_participants.contains(&p1_idx) {
                    p1_level_for_constraint = 0;
                }
                if destroyed_participants.contains(&p2_idx) {
                    p2_level_for_constraint = 0;
                }
                msg_p1_levels.push(p1_level_for_constraint);
                msg_p2_levels.push(p2_level_for_constraint);
                mi += 1;
            } else if let SequenceEvent::LifeEvent(le) = event {
                let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                if p_idx < pre_participant_levels.len() {
                    if le.is_activate() {
                        pre_participant_levels[p_idx] += 1;
                        max_participant_levels[p_idx] = max_participant_levels[p_idx].max(pre_participant_levels[p_idx]);
                    } else if le.is_destroy() {
                        pre_participant_levels[p_idx] = (pre_participant_levels[p_idx] - 1).max(0);
                        destroyed_participants.insert(p_idx);
                    } else if le.is_deactivate() {
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
    // Autonumber: bold13 number per message and its prefix width
    // (number advance + 4px gap). The prefix joins the tile reservation only
    // when the message also has a label; the number is drawn regardless.
    let auto_font_bold = UFont::sans_serif(FONT_SIZE_MESSAGE).with_style(FontStyle::bold());
    let auto_num_strings: Vec<String> = (0..pre_wrapped_widths.len())
        .map(|i| (i + 1).to_string())
        .collect();
    let auto_num_widths: Vec<f64> = auto_num_strings
        .iter()
        .map(|s| bounder.calculate_dimension(&auto_font_bold, s).width())
        .collect();
    let auto_prefix: Vec<f64> = auto_num_widths.iter().map(|w| w + 4.0).collect();
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
            let text_w = if autonumber && !label.is_empty() {
                text_w + auto_prefix[spacing_msg_idx]
            } else {
                text_w
            };
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
                let max_act = if destroyed_participants.contains(&p1_idx) {
                    0
                } else {
                    *max_participant_levels.get(p1_idx).unwrap_or(&0)
                } as f64;
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
            // A forward self tile on a participant already destroyed contributes
            // no live-segment reserve (JAR getMaxPosition at that tile is 0).
            let reserve_level = if destroyed_participants.contains(&p) {
                0
            } else {
                *max_participant_levels.get(p).unwrap_or(&0)
            };
            let self_max_pos = 5.0 * reserve_level as f64;
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
        // Leftmost head renders at PAGE_MARGIN past its solver value; Java places
        // it at x=20 and the frame edge 10px left of it, so floor posB at 15.
        let indent_floor = plantuml_real::add_fixed(&xorigin, 15.0);
        plantuml_real::ensure_bigger_than(&pos_b[leftmost], &indent_floor);

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
        title_width = bold14_width(title_text);
        // Java: `DecorateEntityImage` stacks the bordered title block above the
        // raw diagram; when the block (`bold` width + TITLE_BLOCK_W_EXTRA) is
        // wider than the participant span, participants are shifted right by
        // half the excess so the title can center on the raw diagram width.
        let participant_span = pos_d_vals.last().copied().unwrap_or(0.0)
            - pos_b_vals.first().copied().unwrap_or(0.0);
        let excess = title_width + TITLE_BLOCK_W_EXTRA - participant_span;
        if excess > 0.0 {
            let shift = excess / 2.0;
            for v in &mut pos_b_vals {
                *v += shift;
            }
            for v in &mut pos_c_vals {
                *v += shift;
            }
            for v in &mut pos_d_vals {
                *v += shift;
            }
        }
    }

    // ── Compute header/footer/legend/caption dimensions and styles ───────
    let header_font_size = style_rules.get("header").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(HEADER_FONT_SIZE);
    let footer_font_size = style_rules.get("footer").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(HEADER_FONT_SIZE);
    let legend_font_size = style_rules.get("legend").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(FONT_SIZE_PARTICIPANT as f64);
    let caption_font_size = style_rules.get("caption").and_then(|r| r.get("FontSize")).and_then(|v| v.parse::<f64>().ok()).unwrap_or(FONT_SIZE_PARTICIPANT as f64);

    // Compute text widths and element dimensions
    // AWT LineMetrics: line height = size*1.362, ascent = size*1.069 for the
    // plain sans-serif banner text. Ribbon rects (header/caption/footer) use the
    // bare line height with no vertical padding; legend/title bordered blocks
    // add 10 (5+5 style padding).
    let ribbon_h = |size: f64| size * 1.362;
    let ribbon_asc = |size: f64| size * 1.069;
    let header_dim = header_text.map(|t| {
        let font = UFont::sans_serif(header_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw, ribbon_h(header_font_size))
    });
    let title_dim = title.map(|t| (bold14_width(t) + TITLE_BLOCK_W_EXTRA, 29.068));
    let legend_dim = legend_text.map(|t| {
        let font = UFont::sans_serif(legend_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw + 10.0, 29.068)
    });
    let caption_dim = caption_text.map(|t| {
        let font = UFont::sans_serif(caption_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw, ribbon_h(caption_font_size))
    });
    let footer_dim = footer_text.map(|t| {
        let font = UFont::sans_serif(footer_font_size as i32);
        let tw = bounder.calculate_dimension(&font, t).width();
        (tw, ribbon_h(footer_font_size))
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
    let title_height = if title.is_some() { TITLE_SHIFT_Y } else { 0.0 };
    // Header pushes the body by ribbon line height (size*1.362) plus the +1
    // outer TextBlockBordered pixel, replacing STARTING_Y.
    let header_extra = if header_text.is_some() {
        ribbon_h(header_font_size) + 1.0 - STARTING_Y
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

    // ── Puma ArrowAndNoteBox vertical centering (legacy non-teoz) ──────────
    // An attached note ("note right/left:") on a NON-SELF message shares its
    // message tile. Java's ArrowAndNoteBox sizes the tile boxH = max(arrowH,
    // noteH) and pushes the arrow and note down by (boxH - ownH)/2 so both are
    // vertically centered. Metrics are live-JAR measured (rose skin, font 13):
    //   arrowH = textH + arrowDeltaY(4) + 2*paddingY(4) = textH + 12
    //   noteH  = 20 + k*17.706  (k = note lines; note text base 10 + 2*5 pad)
    // Self-message notes use MessageSelfArrow + ArrowAndNoteBox with a
    // different layout and are intentionally excluded.
    let mut tile_is_self: Vec<bool> = Vec::with_capacity(n_msgs);
    {
        let mut mi = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(m) = event {
                let exo = msg_exo.get(mi).copied().flatten();
                tile_is_self.push(exo.is_none() && m.p1().code() == m.p2().code());
                mi += 1;
            }
        }
    }
    let mut note_line_k: Vec<usize> = vec![0; n_msgs];
    for n in notes {
        if n.participant.is_none() && n.msg_index < n_msgs {
            let k = n.text.split("\\n").count();
            if k > note_line_k[n.msg_index] {
                note_line_k[n.msg_index] = k;
            }
        }
    }
    let mut note_arrow_push: Vec<f64> = vec![0.0; n_msgs];
    let mut note_tile_extra: Vec<f64> = vec![0.0; n_msgs];
    // Extra height below the tile's own contact (above the +10 lifeline margin).
    // Plain non-self tile: 12 (arrowDeltaY + 2*paddingY). A tile carrying a
    // taller attached note grows this to boxH - pushA - textH.
    let mut chain_tail: Vec<f64> = vec![ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y; n_msgs];
    // Per-group: true when the group's first message chains directly off the
    // bottom of a previously closed sibling group (its frame-top external
    // margin term is 0 instead of the usual 3). Live-JAR verified.
    let mut group_off_sibling: Vec<bool> = vec![false; groups.len()];
    for mi in 0..n_msgs {
        if tile_is_self.get(mi).copied().unwrap_or(false) {
            continue;
        }
        let lc = pre_wrapped_lines[mi].len().max(1) as f64;
        let text_h = lc * MSG_LINE_HEIGHT + MSG_PADDING_TOP + MSG_PADDING_BOTTOM;
        let arrow_h = text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y;
        let k = note_line_k[mi];
        if k > 0 {
            let note_h = 2.0 * NOTE_MARGIN_Y + (k as f64) * MSG_LINE_HEIGHT + 10.0;
            let box_h = arrow_h.max(note_h);
            note_arrow_push[mi] = (box_h - arrow_h) / 2.0;
            note_tile_extra[mi] = box_h - arrow_h;
            chain_tail[mi] = box_h - note_arrow_push[mi] - text_h;
        }
    }
    // Per-message self-carry lifecycle flags (legacy Puma createMessageSelfArrow
    // deltaY and DrawableSetInitializer LARGER/SMALLER positions). For each message
    // index we record whether an inline LifeEvent on the message's OWN participant
    // activates or deactivates/destroys it.
    let mut self_carry_act: Vec<bool> = vec![false; n_msgs];
    // (is_destroy) for a self message that carries a deactivate/destroy.
    let mut self_carry_deact: Vec<Option<bool>> = vec![None; n_msgs];
    for event in diagram.events() {
        if let SequenceEvent::LifeEvent(le) = event {
            if let Some(mi) = le.message_index() {
                if mi < n_msgs && tile_is_self.get(mi).copied().unwrap_or(false) {
                    // A self message has one participant, so any inline
                    // LifeEvent is on its own participant.
                    if le.is_activate() {
                        self_carry_act[mi] = true;
                    } else if le.is_destroy() {
                        self_carry_deact[mi] = Some(true);
                    } else if le.is_deactivate() {
                        self_carry_deact[mi] = Some(false);
                    }
                }
            }
        }
    }
    // arrow (constructed in Step1Message ctor) consumes one value; that inner
    // uid is the one emitted on <g class="message">. When the message carries an
    // attached note, createArrow ALSO builds an ArrowAndNoteBox wrapper which
    // consumes the NEXT value, but that wrapper uid is never drawn. So the drawn
    // uid for message mi = 1 + mi + (noted messages strictly before mi).
    let mut msg_uid: Vec<usize> = vec![0; n_msgs];
    {
        let mut noted_before = 0usize;
        for mi in 0..n_msgs {
            msg_uid[mi] = 1 + mi + noted_before;
            if note_line_k[mi] > 0 {
                noted_before += 1;
            }
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
    // Destroy X emitted inline right after its carrying message (legacy
    // GraphicalElementLifeEvent/DrawableSet addEvent ordering): Some(center_y).
    let mut inline_destroy: Vec<Option<(usize, f64)>> = vec![None; n_msgs];
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
    // Total height of standalone notes ("note right of X: ...") sitting in the
    // gap AFTER each message index. Standalone notes are independent tiles in
    // the Y gauge and stack cumulatively (each one-line note adds
    // STANDALONE_NOTE_H), unlike attached notes ("note right:") which only
    // grow the same message tile via a maximum.
    let mut standalone_gap_h: Vec<f64> = vec![0.0; diagram.events().iter().filter(|e| matches!(e, SequenceEvent::Message(_))).count().max(1)];
    for note in notes {
        if note.participant.is_none() {
            continue;
        }
        if note.msg_index >= standalone_gap_h.len() {
            continue;
        }
        let line_count = note.text.split("\\n").count().max(1) as f64;
        let h = STANDALONE_NOTE_H + (line_count - 1.0) * STANDALONE_NOTE_LINE_H;
        standalone_gap_h[note.msg_index] += h;
    }
    // Divider tiles in the gap before each following message. A divider is an
    // independent tile (like a standalone note): add its full preferred height
    // to the gap after message `before_msg - 1`.
    let mut divider_gap_h: Vec<f64> = vec![0.0; standalone_gap_h.len()];
    for div in dividers {
        if div.before_msg > 0 {
            let gi = div.before_msg - 1;
            if gi < divider_gap_h.len() {
                divider_gap_h[gi] += DIVIDER_TILE_H;
            }
        }
    }
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let _has_text = !msg.label().is_empty();
            // Use precomputed wrapped lines
            let wrapped_lines = pre_wrapped_lines[msg_idx].clone();
            let line_count = wrapped_lines.len();
            msg_wrapped_lines.push(wrapped_lines);
            let text_h = (line_count as f64) * MSG_LINE_HEIGHT + MSG_PADDING_TOP + MSG_PADDING_BOTTOM;
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
                    let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
                    current_y += inc + ELSE_TILE_HEIGHT;
                } else {
                    // Parallel message not first in a group: use the cluster's chaining point
                    current_y = cluster_start_y;
                }
            } else if msg_idx == 0 {
                // First message overall
                current_y = lifeline_y + MSG_STARTING_Y + MSG_PADDING_Y + text_h;
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
                let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
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
                    let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
                    current_y += inc;
                    group_header_offset = GROUP_HEADER_OFFSET + header_extra;
                    current_y += group_header_offset;
                } else if let Some(fb) = prev_frame_bottom {
                    // After a group ends: chaining point = fb + gap (10).
                    if let Some(cgi) = curr_group {
                        group_off_sibling[cgi] = true;
                    }
                    let base = fb + 10.0;
                    group_header_offset = GROUP_HEADER_OFFSET + GROUP_HEADER_HEIGHT + header_extra;
                    current_y = base + group_header_offset;
                } else {
                    // First group after non-grouped messages
                    let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                    let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
                    current_y += inc;
                    group_header_offset = GROUP_HEADER_OFFSET + header_extra;
                    current_y += group_header_offset;
                }
            } else {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                if !prev_is_self && msg_idx > 0 {
                    // ArrowAndNoteBox chaining: previous tile contributes
                    // chain_tail (12 plain; taller when a note is attached).
                    current_y += chain_tail[msg_idx - 1] + text_h;
                } else if let Some(nh) = prev_note_h {
                    current_y += nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra);
                } else {
                    current_y += text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra;
                }
            }
            // Standalone notes between the previous message and this one are
            // independent gauge tiles: always add the previous gap's total.
            if msg_idx > 0 && !is_parallel {
                current_y += standalone_gap_h[msg_idx - 1];
            }
            if msg_idx > 0 && !is_parallel {
                current_y += divider_gap_h[msg_idx - 1];
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
            // ArrowAndNoteBox centering: shift this tile's arrow contact down
            // when the attached note is the taller tile child.
            current_y += note_arrow_push[msg_idx];
            arrow_ys.push(current_y);
            prev_note_h = notes.iter().find(|n| n.msg_index == msg_idx && n.participant.is_none()).map(|note| {
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
                    let last_note = notes.iter().find(|n| n.msg_index == last_mi && n.participant.is_none());
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
                        // Frame bottom = last contact + chain_tail - 4 (same
                        // live-JAR rule used for the drawn frame), so inter-group
                        // chaining starts from the correct edge.
                        let fb = if last_is_self || group.group_type == "partition" {
                            let fh = body_height + GROUP_HEADER_HEIGHT + GROUP_MARGIN_Y_MAGIC / 2.0 + p_extra;
                            fy + fh
                        } else {
                            arrow_ys[last_mi] + chain_tail[last_mi] - 4.0
                        };
                        let frame_height = fb - fy;
                        (fy, frame_height)
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
            const PADDING_Y: f64 = 4.0;
            let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
            // Puma anchors a carrying LifeEvent to the self tile's LEVEL Ys
            // (DrawableSetInitializer.prepareLiveEvent). arrow_ys[mi] is the
            // unshifted START level (bottom hline = start+13): LARGER bar top
            // = startLevel+8, SMALLER bar bottom = endLevel-7 = start+6. The
            // LifeDestroy component (18 tall) is placed with its TOP at
            // endLevel-9 = start+4. For a non-self tile start/end level
            // coincide at the contact: bar top = contact, bottom = contact-7,
            // destroy top = contact-9.
            let (act_y, deact_y, destroy_top) = if let Some(mi) = le.message_index() {
                let contact = arrow_ys.get(mi).copied().unwrap_or(current_position);
                let carrying_self = is_self_flags.get(mi).copied().unwrap_or(false);
                if carrying_self {
                    (contact + 8.0, contact + 6.0, contact + 4.0)
                } else {
                    // DrawableSetInitializer.prepareLiveEvent: non-self
                    // ACTIVATE SMALLER/LARGER anchor at getPosYendLevel with
                    // no delta (the +7 delta applies to self messages only);
                    // DESTROY anchors compHeight/2 (9) above endLevel.
                    (contact, contact, contact - 9.0)
                }
            } else {
                // Standalone LifeEvent (before any message / other participant).
                // Puma positions the LARGER bar top at the free-Y frontier:
                //  - no previous tile: frontier+2 (PlayingSpace start offset);
                //  - after a reverse self tile: frontier+2*paddingY (+8), the
                //    reverse self contributes no arrow-only height;
                //  - after a forward self tile: frontier+SELF_ARROW_HEIGHT+8.
                let a = if prev_is_self {
                    if prev_is_reverse {
                        current_position + 2.0 * PADDING_Y
                    } else {
                        current_position + SELF_ARROW_HEIGHT + 2.0 * PADDING_Y
                    }
                } else {
                    current_position + 2.0
                };
                let d = if prev_is_self && !prev_is_reverse {
                    current_position + SELF_ARROW_HEIGHT + 2.0 * PADDING_Y
                } else {
                    current_position
                };
                (a, d, current_position)
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
                        activations.push((p_idx, start_y, deact_y, level));
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
                if let Some(mi) = le.message_index() {
                    if mi < n_msgs {
                        inline_destroy[mi] = Some((p_idx, destroy_top));
                    } else {
                        destroys.push((p_idx, destroy_top));
                    }
                } else {
                    destroys.push((p_idx, destroy_top));
                }
                if p_idx < participant_levels.len() {
                    participant_levels[p_idx] = (participant_levels[p_idx] - 1).max(0);
                }
            }
        }
    }

    // ── Compute group frame dimensions ──────────────────────────────────
    let mut group_frames: Vec<(f64, f64, f64, f64)> = Vec::new(); // (x, y, w, h)
    // Per group: a graphic tile (attached note or self-message arrow), rather
    // than the leftmost participant head, sets the in-groupable list minimum.
    let mut graphic_drives_left: Vec<bool> = Vec::new();
    // Per group: a self-message arrow tile (not an attached note) sets the
    // list minimum; its list-to-chrome inset is one pixel larger.
    let mut self_drives_left: Vec<bool> = Vec::new();
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
        let last_note = notes.iter().find(|n| n.msg_index == last_mi && n.participant.is_none());
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
        // Which kind of graphic tile drives this group's minimum X (a
        // self-message arrow vs an attached note), so the chrome inset can
        // match the JAR list-member inset.
        let mut min_via_self = false;
        let mut min_via_left_note = false;
        // Frame wraps the participant head rectangles (± 10px), verified
        // against the jar: frame left = leftmost head x - 10, frame right =
        // rightmost head (x+w) + 10.
        let mut hd_min = f64::MAX;
        let mut hd_max = f64::MIN;
        let mut msg_iter_idx = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                if msg_iter_idx >= group.msg_start && msg_iter_idx < group.msg_end {
                    let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    for &pi in &[p1_idx, p2_idx] {
                        hd_min = hd_min.min(pos_b_vals[pi]);
                        hd_max = hd_max.max(pos_b_vals[pi] + head_widths[pi]);
                    }
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
                    let mi_note = notes.iter().find(|n| n.msg_index == msg_iter_idx && n.participant.is_none());
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
                            // JAR MessageSelfArrow.getPreferredWidth reserves the
                            // arrow component width, which insets the wrapped
                            // message text by MESSAGE_TEXT_X_OFFSET on BOTH the
                            // left and right (label_w + 14), floored by the
                            // self-loop's right extent (SELF_XRIGHT). The drawn
                            // loop visual is shorter, but the in-groupable
                            // reserve uses this full preferred width, which is
                            // what drives the page origin for a reverse self.
                            SELF_XRIGHT
                                .max(2.0 * MESSAGE_TEXT_X_OFFSET + label_w)
                                + ACTIVATION_BAR_EXPLICIT_OFFSET * max_act
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
                                    // JAR NoteBox.getStartingX (LEFT):
                                    // (int)(segment.pos1 - getPreferredWidth);
                                    // pos1 is the (possibly live-thickness)
                                    // segment left edge, equal to posC here;
                                    // getPreferredWidth is the full note
                                    // component width (incl. shadow), and the
                                    // result truncates toward zero.
                                    let note_p_idx = if is_reverse { p2_idx } else { p1_idx };
                                    let note_min = (pos_c_vals[note_p_idx] - note_comp_w).trunc();
                                    d_min = d_min.min(note_min);
                                }
                            }
                        }
                    }

                    // Record which tile kind drives the running group minimum.
                    if d_min < min_x {
                        min_via_self = is_self;
                        min_via_left_note = !is_self
                            && matches!(
                                mi_note.map(|n| n.position),
                                Some(NotePosition::Left)
                            );
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
            let top_margin_term = if group_off_sibling[gi] { 0.0 } else { 3.0 };
            let fy = arrow_ys[group.msg_start] - GROUP_HEADER_OFFSET * headers_to_subtract as f64 - first_text_h + top_margin_term - p_extra;
            let fh = if is_self_flags[last_mi] || group.group_type == "partition" {
                body_height + GROUP_HEADER_HEIGHT + GROUP_MARGIN_Y_MAGIC / 2.0 + p_extra
            } else {
                // Frame bottom = last contact + chain_tail - 4 (live-JAR rule:
                // the group closes 4px above the last tile's below-contact tail).
                let bottom = arrow_ys[last_mi] + chain_tail[last_mi] - 4.0;
                bottom - fy
            };
            (fy, fh)
        };
        // Java InGroupableList aggregates the participant boxes AND every
        // in-groupable graphic (the ArrowAndNoteBox spans the arrow plus any
        // attached note), then GroupingGraphicalElement insets by 10. So the
        // frame must wrap both the participant head rects (hd_min/hd_max) and
        // the drawn message/note extents (min_x/max_x).
        let eff_min = hd_min.min(min_x);
        let eff_max = hd_max.max(max_x);
        let frame_x = eff_min - 10.0;
        let frame_width = eff_max - eff_min + 20.0;
        // frame_x/frame_width are the GroupingGraphicalElement reservation
        // (10px outside the in-groupable list on each side). The visible chrome
        // rect inset at header time depends on whether a graphic tile or a
        // participant head reaches the list minimum.
        let graphic_left = min_x.is_finite() && min_x < hd_min;
        graphic_drives_left.push(graphic_left);
        self_drives_left.push(graphic_left && min_via_self && !min_via_left_note);
        group_frames.push((frame_x, frame_y, frame_width, frame_height));
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
        let tab_label_w = max_line_width(&bounder, &font_bold, &tab_label);
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
            let (efx, _efy, efw, _efh) = group_frames[gi];
            let new_left = pfx.min(efx);
            let new_right = (pfx + pfw).max(efx + efw);
            let new_width = new_right - new_left;
            let else_bottom = arrow_ys[groups[gi].msg_end - 1] + 8.0;
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
    // Apply header width after nesting extension: frame_max >= frame_min + header_width + MARGIN_X.
    for (gi, group) in groups.iter().enumerate() {
        if group.group_type == "else" || group.group_type == "partition" {
            continue;
        }
        let (fx, fy, fw, fh) = group_frames[gi];
        let frame_max = fx + fw;
        let header_max = fx + header_widths[gi] + PAGE_MARGIN;
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
            let (efx, _efy, efw, _efh) = group_frames[gi];
            let new_left = pfx.min(efx);
            let new_right = (pfx + pfw).max(efx + efw);
            let new_width = new_right - new_left;
            // Legacy frame ends 8px below the else section's last arrow.
            let else_last = groups[gi].msg_end - 1;
            let else_bottom = arrow_ys[else_last] + 8.0;
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

    // Compute note heights for each message
    let mut note_heights: Vec<f64> = vec![0.0; arrow_ys.len()];
    for note in notes {
        if note.participant.is_none() && note.msg_index < note_heights.len() {
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
            let msg_lines = msg_text_heights.get(i).map_or(1, |&h| ((h - MSG_PADDING_TOP - MSG_PADDING_BOTTOM) / MSG_LINE_HEIGHT) as usize);
            let note_y_top = arrow_ys[i] - ARROW_Y_BASE - (msg_lines.saturating_sub(1) as f64) * MSG_LINE_HEIGHT;
            let note_bottom = note_y_top + note_h;
            if note_bottom > max_note_bottom {
                max_note_bottom = note_bottom;
            }
        }
    }

    // Lifeline ends below the last contact by chain_tail + 6 (ArrowAndNoteBox:
    // the last tile's below-contact height plus the +6 outer margin). Plain
    // non-self tiles give 12 + 6 = 18; a tile carrying a taller attached note
    // grows chain_tail accordingly.
    let self_extra = if last_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
    let last_chain_tail = chain_tail.get(last_arrow_idx).copied().unwrap_or(ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y);
    let normal_lifeline_bottom = if last_is_self {
        last_arrow_y + self_extra + ARROW_Y_BASE + 4.0
    } else {
        last_arrow_y + last_chain_tail + 6.0
    };
    let note_lifeline_bottom = 0.0_f64;
    // Standalone notes after the last message extend the lifeline by their
    // total tile height (the Y gauge max + paddingY chain).
    let standalone_lifeline_bottom = normal_lifeline_bottom
        + standalone_gap_h.get(last_arrow_idx).copied().unwrap_or(0.0);
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
                // Puma runs the lifeline 17px below the last (innermost)
                // grouping tile's frame bottom; the frame itself already
                // reserves the member tail. This floor dominates the last
                // message's Y-gauge for a group that ends the diagram.
                let group_msg_ygauge_max = arrow_ys.iter().enumerate()
                    .filter(|&(mi, _)| mi >= g.msg_start && mi < g.msg_end)
                    .map(|(_, &y)| y + java_preferred_height)
                    .fold(frame_bottom + 17.0, f64::max);
                group_msg_ygauge_max
            };
            val
        })
        .fold(0.0_f64, f64::max);
    // When parallel messages are present, cap normal_lifeline_bottom at msg_ygauge_max + PAGE_MARGIN
    // to avoid 1px overshoot from text_h=26 vs Java's 25 in self-messages.
    // Also cap at group_lifeline_bottom when the max-arrow-Y message is inside a parallel group,
    // because its height is already accounted for by the group's own lifeline_bottom.
    let max_arrow_in_parallel_group = groups.iter().any(|g| g.parallel && last_arrow_idx >= g.msg_start && last_arrow_idx < g.msg_end);
    let _normal_lifeline_bottom = if has_parallel {
        let capped = normal_lifeline_bottom.min(msg_ygauge_max + PAGE_MARGIN);
        if max_arrow_in_parallel_group && group_lifeline_bottom > 0.0 {
            capped.min(group_lifeline_bottom)
        } else {
            capped
        }
    } else {
        normal_lifeline_bottom
    };
    let lifeline_bottom = standalone_lifeline_bottom.max(note_lifeline_bottom).max(group_lifeline_bottom);
    // Flush remaining pending activations (autoactivate without explicit
    // deactivate): legacy Puma closes open live boxes at the playing-space
    // maxY = lifeline_bottom - 9 (halfLifeWidth + bottom padding).
    let pending_bottom = lifeline_bottom - 9.0;
    for (pi, stack) in pending_activations.iter().enumerate() {
        for &(start_y, level) in stack {
            activations.push((pi, start_y, pending_bottom, level));
        }
    }

    let lifeline_height = if arrow_ys.is_empty() {
        LIFELINE_HEIGHT_BASE
    } else {
        lifeline_bottom - lifeline_y
    };
    let footbox_y = lifeline_y + lifeline_height;
    let tail_y = footbox_y - 1.0;

    // ── Compute total dimensions ──────────────────────────────────────────
    // First pass: compute min_layout_left using solver values (without x_offset)
    // to determine how far left notes extend, then set x_offset accordingly.
    let mut min_layout_left = 0.0_f64;
    for note in notes {
        if note.participant.is_some() || note.msg_index >= arrow_ys.len() {
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
                // MessageSelfArrow.getPreferredWidth = compPrefW + live
                // segment length; the segment at this level spans left+right
                // shifts (10 per level), not the half-width used for drawing.
                let comp_width = (text_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0) + 3.0 * ACTIVATION_BAR_EXPLICIT_OFFSET * max_act;
                let p_center = pos_c_vals.get(p1_idx).copied().unwrap_or(0.0);
                let loop_left = p_center - comp_width;
                if loop_left < min_layout_left {
                    min_layout_left = loop_left;
                }
            }
            sm_msg_idx += 1;
        }
    }
    // Also account for group frame left edges. JAR prepareMissingSpace
    // reserves delta1 = -GroupingGraphicalElement.getStartingX() where
    // getStartingX = listMin - 10; frame_x already holds that reservation
    // edge, so the page origin is driven directly by frame_x.
    let mut min_left = min_layout_left;
    for &(fx, _, _, _) in &group_frames {
        if fx < min_left {
            min_left = fx;
        }
    }
    let x_offset = PAGE_MARGIN - min_left.min(0.0);
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
        if note.participant.is_some() || note.msg_index >= arrow_ys.len() {
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
        } else if note_level > 0 {
            // Activated RIGHT note: the polygon already starts on the shifted
            // live segment (segment.pos2 = center + level*5 + 5), so the
            // playing space reserves only its integer polygon extent plus the
            // left old-padding, not the full preferred width.
            let x2_ext = (note_text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2).trunc();
            p_center + x2_ext + NOTE_OLD_PADDING_X1
        } else {
            // RIGHT note on an unactivated segment: reserve the full
            // preferred width from the participant center.
            p_center + layout_w
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
    // Standalone notes use a different (tile-level) geometry; compute their
    // right edges independently for the canvas width.
    let mut max_standalone_right = 0.0_f64;
    for note in notes {
        if note.participant.is_none() {
            continue;
        }
        let target = note.participant.as_deref().unwrap_or("");
        let code = target.split(',').next().unwrap_or(target);
        let p_idx = pcode_to_idx.get(code).copied().unwrap_or(0);
        let text_w = max_line_width(&bounder, &font_m, &note.text);
        let polygon_w = (text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2).trunc();
        let polygon_x = (pos_c_vals[p_idx] + x_offset + NOTE_PADDING_X).round();
        max_standalone_right = max_standalone_right.max(polygon_x + polygon_w + 1.0);
    }
    let max_frame_right: f64 = group_frames
        .iter()
        .map(|&(fx, _, fw, _)| fx + fw + PAGE_MARGIN + x_offset)
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
    // `CommunicationTileSelf.getMaxX` = posC2 + compWidth (the loop's layout
    // reservation to the right; includes unused slack when the label is short).
    let self_tile_right: f64 = max_forward_comp_width
        .iter()
        .enumerate()
        .map(|(p, &cw)| {
            let pos_c2 = pos_c_vals[p]
                + 5.0 * max_participant_levels.get(p).copied().unwrap_or(0) as f64;
            pos_c2 + cw
        })
        .fold(0.0_f64, f64::max);
    let content_right = rightmost_x
        .max(title_rightmost)
        .max(max_note_right)
        .max(max_frame_right)
        .max(self_tile_right)
        .max(max_standalone_right);
    let has_extra_elements = header_text.is_some() || footer_text.is_some() || legend_text.is_some() || caption_text.is_some();
    let total_width = if has_extra_elements {
        content_right.max(max_element_width + PAGE_MARGIN * 2.0) + 2.0 * PAGE_MARGIN
    } else if title_width > 0.0 {
        content_right + 2.0 * PAGE_MARGIN + 1.0
    } else {
        content_right + 2.0 * PAGE_MARGIN
    };
    // Override partition frame dimensions to span the full diagram width
    let full_right = rightmost_x.max(max_note_right).max(max_frame_right);
    for (gi, group) in groups.iter().enumerate() {
        if group.group_type == "partition" {
            let (_, fy, _, fh) = group_frames[gi];
            let frame_x = PAGE_MARGIN - x_offset;
            let frame_width = full_right - PAGE_MARGIN;
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
        // Mirror the draw chain: legend starts +13 (+14 with hidden footbox),
        // then caption +14, footer +2, each with its ribbon/block height.
        if legend_text.is_some() { extra += if hide_footbox { 14.0 } else { 13.0 } + legend_h; }
        if caption_text.is_some() { extra += CAPTION_GAP + caption_h; }
        if footer_text.is_some() { extra += FOOTER_GAP + footer_h; }
        extra
    } else {
        0.0
    };
    let total_height = if has_extra_elements {
        footbox_bottom + PAGE_MARGIN + height_extra + delta_shadow + bottom_extra
    } else {
        footbox_bottom + PAGE_MARGIN + height_extra + delta_shadow
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
    // Header is right-aligned; title/legend/caption/footer centered on this
    // canvas-interior reference edge.
    let centering_width = rightmost_x + PAGE_MARGIN - 1.0;
    if let Some(header_txt) = header_text {
        let font_h = UFont::sans_serif(header_font_size as i32);
        let text_w = bounder.calculate_dimension(&font_h, header_txt).width();
        let rect_w = header_dim.map_or(text_w, |(w, _)| w);
        let rect_h = header_dim.map_or(ribbon_h(header_font_size), |(_, h)| h);
        let rect_y = PAGE_MARGIN;
        // Right-aligned to the interior reference edge.
        let rect_x = centering_width - rect_w;
        let text_y = rect_y + ribbon_asc(header_font_size);

        let line_str = header_line.map(|n| n.to_string());
        let mut attrs = vec![("class", "header")];
        if let Some(ref ls) = line_str {
            attrs.push(("data-source-line", ls.as_str()));
        }
        svg.open_group_with_attrs(&attrs);

        // Background rect only when a style fill exists (TextBlockBordered
        // skips the polygon for a transparent background).
        if let Some(bg) = header_bg_color.as_deref() {
            svg.set_fill_color(bg);
            svg.set_stroke_color(None);
            svg.set_stroke_width(1.0, None);
            svg.svg_rectangle(rect_x, rect_y, rect_w, rect_h, 0.0, 0.0, 0.0);
        }

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
        let text_w = bold14_width(title_text);
        let rect_w = text_w + 10.0; // 5px padding each side
        let rect_h = 29.0679;
        let rect_y = PAGE_MARGIN + STARTING_Y + header_extra + 5.0;
        // Java centers the bordered title block on the raw diagram width; the
        // text sits 5px inside the block. With no header/footer/legend/caption
        // the block starts at x = PAGE_MARGIN (5); with extras it is centered.
        let (rect_x, text_x) = if has_extra_elements {
            let rx = (centering_width - rect_w) / 2.0;
            (rx, rx + 5.0)
        } else {
            (PAGE_MARGIN, PAGE_MARGIN + 5.0)
        };
        let text_y = rect_y + TITLE_TEXT_OFFSET;
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
            // The chrome border is the in-groupable list bounds, drawn here
            // before the lifelines (JAR emits both frame rects first). A
            // self-message-arrow-driven minimum insets 5px, a note-driven
            // minimum 4px, a participant-driven minimum 0; the right edge is
            // unchanged.
            let chrome_dx = if self_drives_left.get(gi).copied().unwrap_or(false) {
                5.0
            } else if graphic_drives_left.get(gi).copied().unwrap_or(false) {
                4.0
            } else {
                0.0
            };
            svg.set_fill_color("none");
            svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
            svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
            svg.svg_rectangle(
                fx + x_offset + chrome_dx,
                fy,
                fw - chrome_dx,
                fh,
                0.0,
                0.0,
                0.0,
            );
        }
    }

    // ── Draw explicit activation boxes (first pass, behind lifelines) ─────
    draw_activation_bars(&mut svg, &activations, participants, &pos_c_vals, x_offset);

    // ── Draw lifelines (background) ───────────────────────────────────────
    // ── Draw lifelines + activation bars + destroy marks (per participant) ──
    for (i, p) in participants.iter().enumerate() {
        let cx = pos_c_vals[i] + x_offset;
        // Java's ComponentRoseLine draws the line at posB + (int)(head_width/2),
        // truncating the fractional part — NOT at posC = posB + head_width/2.
        let line_x = pos_b_vals[i] + ((head_widths[i] / 2.0) as i64) as f64 + x_offset;
        let ly = lifeline_y;

        // Legacy Puma never truncates a lifeline at destroy (LifeLine.getDestroy
        // returns 0); the destroy X is a separate inline event. The dashed line
        // and transparent bar always run the full lifeline height.
        let ll_height = lifeline_height;

        // Wrapper <g class="participant-lifeline" ...>
        let uid = p.uid();
        let source_line = participant_source_lines.get(i).copied().flatten();
        let line_str = source_line.map(|n| n.to_string());
        let mut ll_attrs: Vec<(&str, String)> = vec![
            ("class", "participant-lifeline".to_string()),
            ("id", format!("{uid}-lifeline")),
            ("data-qualified-name", p.code().to_string()),
            ("data-entity-uid", uid.to_string()),
        ];
        if let Some(ref ls) = line_str {
            ll_attrs.push(("data-source-line", ls.clone()));
        }
        let ll_attrs_final: Vec<(&str, &str)> = ll_attrs.iter()
            .map(|(k, v)| (*k, v.as_str())).collect();
        svg.open_group_with_attrs(&ll_attrs_final);

        // Inner <g> with <title>, rect, line
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
        svg.svg_line(line_x, ly, line_x, ly + ll_height, 0.0);

        svg.close_group();
        svg.close_group();


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
        // Bottom align each head: offset its own origin so its bottom lands at
        // head_y + common_own (mirrors LivingSpaces.drawHeads BOTTOM).
        let hy = head_y + (common_own - own_heights[i]);
        // Wrapper <g class="participant participant-head" ...>
        let uid = p.uid();
        let source_line = participant_source_lines.get(i).copied().flatten();
        let line_str = source_line.map(|n| n.to_string());
        let mut head_attrs: Vec<(&str, String)> = vec![
            ("class", "participant participant-head".to_string()),
            ("id", format!("{uid}-head")),
            ("data-qualified-name", p.code().to_string()),
            ("data-entity-uid", uid.to_string()),
        ];
        if let Some(ref ls) = line_str {
            head_attrs.push(("data-source-line", ls.clone()));
        }
        let head_attrs_final: Vec<(&str, &str)> = head_attrs.iter()
            .map(|(k, v)| (*k, v.as_str())).collect();
        svg.open_group_with_attrs(&head_attrs_final);

        if is_actor[i] {
            // Actor: draw text first, then stickman (matching Java's ComponentRoseActor order)
            // Text below stickman
            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
            let text_y = hy + STICKMAN_HEIGHT + ACTOR_TEXT_BASELINE;

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
            let head_cy = hy + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
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
            svg.svg_rectangle(x, hy, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
            svg.set_filter(None);

            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = x + (w - text_w) / 2.0;
            let text_y = hy + PADDING_TOP + ASCENT_14;

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
        svg.close_group();

        // ── Draw footbox (tail) for this participant ──────────────────────
        if !hide_footbox {
            // Wrapper <g class="participant participant-tail" ...>
            let uid = p.uid();
            let source_line = participant_source_lines.get(i).copied().flatten();
            let line_str = source_line.map(|n| n.to_string());
            let mut tail_attrs: Vec<(&str, String)> = vec![
                ("class", "participant participant-tail".to_string()),
                ("id", format!("{uid}-tail")),
                ("data-qualified-name", p.code().to_string()),
                ("data-entity-uid", uid.to_string()),
            ];
            if let Some(ref ls) = line_str {
                tail_attrs.push(("data-source-line", ls.clone()));
            }
            let tail_attrs_final: Vec<(&str, &str)> = tail_attrs.iter()
                .map(|(k, v)| (*k, v.as_str())).collect();
            svg.open_group_with_attrs(&tail_attrs_final);
            if is_actor[i] {
                // Actor footbox: text above + stickman below (reversed from head)
                let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
                let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
                let text_y = tail_y + ACTOR_TEXT_BASELINE;

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
                // Stickman drawn below the text block (ComponentRoseActor tail).
                let stickman_top = tail_y + ACTOR_TEXT_HEIGHT;
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
                svg.svg_rectangle(x, tail_y, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
                svg.set_filter(None);

                let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
                let text_x = x + (w - text_w) / 2.0;
                let text_y = tail_y + PADDING_TOP + ASCENT_14;

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
            svg.close_group();
        }
    }
    // ── Force SVG dimensions ────────────────────────────────────────────
    svg.set_hidden(true);
    // Background rectangle sets SVG dimensions via ensure_visible.
    // ensure_visible(total_width, ...) → max_x = total_width + 1 = SVG width.
    svg.svg_rectangle(0.0, 0.0, total_width, total_height, 0.0, 0.0, 0.0);
    svg.set_hidden(false);

    // ── Draw explicit activation boxes (second pass, over heads/lifelines) ─
    draw_activation_bars(&mut svg, &activations, participants, &pos_c_vals, x_offset);

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
                        // The visible chrome is the in-groupable list bounds.
                        // A self-message arrow tile reaching the list minimum
                        // inset is 5px inside the reservation left edge; an
                        // attached-note tile is 4px; participant-driven frames
                        // draw at the edge. The right edge is unchanged.
                        let chrome_dx = if self_drives_left.get(gi).copied().unwrap_or(false) {
                            5.0
                        } else if graphic_drives_left.get(gi).copied().unwrap_or(false) {
                            4.0
                        } else {
                            0.0
                        };
                        let cx0 = fx_off + chrome_dx;
                        let cw = fw - chrome_dx;
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
                        let label_w = max_line_width(&bounder, &font_bold, &tab_label);
                        // Tab width = bold title width + 45 (15 left pad + 30 right).
                        let tab_w = label_w + GROUP_TEXT_PADDING + 2.0 * GROUP_TEXT_PADDING;

                        // Header tab path (folded corner). Fold drops
                        // height - cornersize = 9.706, diagonal closes to
                        // (tab_right - cornersize, tab bottom).
                        svg.set_fill_color(COLOR_GROUP_HEADER);
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                        let tab_right = cx0 + tab_w;
                        let path = format!(
                            "M{x},{y} L{x2},{y} L{x2},{y2} L{x3},{y3} L{x},{y3} L{x},{y}",
                            x = format_number_path(cx0),
                            y = format_number_path(fy),
                            x2 = format_number_path(tab_right),
                            y2 = format_number_path(fy + GROUP_TAB_HEIGHT - GROUP_TAB_CORNER),
                            x3 = format_number_path(tab_right - GROUP_TAB_CORNER),
                            y3 = format_number_path(fy + GROUP_TAB_HEIGHT),
                        );
                        svg.svg_path(&path, 0.0);

                        // Frame chrome rect: JAR draws it both in the
                        // background layer (before lifelines) and here on top
                        // of the tab; emit a second copy to match.
                        svg.set_fill_color("none");
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                        svg.svg_rectangle(cx0, fy, cw, fh, 0.0, 0.0, 0.0);

                        // Tab label text (13px bold)
                        svg.set_fill_color(COLOR_TEXT);
                        svg.set_stroke_color(None);
                        svg.set_stroke_width(0.0, None);
                        svg.text(
                            &tab_label,
                            cx0 + GROUP_TEXT_PADDING,
                            fy + GROUP_TAB_TITLE_BASELINE,
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
                                cx0 + tab_w + GROUP_TEXT_PADDING,
                                fy + 13.759,
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
                let (fx, _fy, fw, _fh) = group_frames[gi];
                let fx_off = fx + x_offset;
                let group = &groups[gi];
                let divider_y = arrow_ys[msg_idx.saturating_sub(1)] + 9.0;
                // Dashed divider line across the parent frame
                svg.set_fill_color("none");
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(1.0, Some([2.0, 2.0]));
                svg.svg_line(fx_off, divider_y, fx_off + fw, divider_y, 0.0);
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
                        divider_y + 11.759,
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
            // Divider (`== label ==`) preceding this message: band + label.
            if let Some(div) = dividers.iter().find(|d| d.before_msg == msg_idx) {
                // Band spans the global head range ±5.
                let mut hd_lo = f64::MAX;
                let mut hd_hi = f64::MIN;
                for pi in 0..participants.len() {
                    hd_lo = hd_lo.min(pos_b_vals[pi] + x_offset);
                    hd_hi = hd_hi.max(pos_b_vals[pi] + head_widths[pi] + x_offset);
                }
                let band_x = hd_lo - 5.0;
                let band_w = hd_hi - hd_lo + 10.0;
                let tile_top = arrow_ys[msg_idx - 1] + 8.0;
                let band_y = tile_top + DIVIDER_BAND_Y;

                // Band rect (fill + stroke same divider background).
                svg.set_fill_color(COLOR_DIVIDER_BG);
                svg.set_stroke_color(Some(COLOR_DIVIDER_BG));
                svg.set_stroke_width(1.0, None);
                svg.svg_rectangle(band_x, band_y, band_w, DIVIDER_BAND_H, 0.0, 0.0, 0.0);
                // Two black border lines at top and bottom of the band.
                svg.set_fill_color(COLOR_TEXT);
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(1.0, None);
                svg.svg_line(band_x, band_y, band_x + band_w, band_y, 0.0);
                svg.svg_line(band_x, band_y + DIVIDER_BAND_H, band_x + band_w, band_y + DIVIDER_BAND_H, 0.0);

                // Bold13 label rect (centered on the band).
                let text_w = max_line_width(&bounder, &font_bold, &div.text);
                let label_w = text_w + DIVIDER_LABEL_PAD_X;
                let label_x = band_x + (band_w - label_w) / 2.0;
                let label_y = tile_top + DIVIDER_LABEL_Y;
                svg.set_fill_color(COLOR_DIVIDER_BG);
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(2.0, None);
                svg.svg_rectangle(label_x, label_y, label_w, DIVIDER_LABEL_H, 0.0, 0.0, 0.0);
                // Label text.
                svg.set_fill_color(COLOR_TEXT);
                svg.set_stroke_color(None);
                svg.set_stroke_width(0.0, None);
                svg.text(
                    &div.text,
                    label_x + 6.0,
                    tile_top + DIVIDER_TEXT_BASELINE,
                    None,
                    FONT_SIZE_MESSAGE,
                    Some("700"),
                    None,
                    None,
                    text_w,
                    &indexmap::IndexMap::new(),
                    None,
                );
            }

            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(1);
            let y = arrow_ys[msg_idx];
            // Puma createMessageSelfArrow translates an activate-carrying self
            // tile up by halfLifeWidth(5); a deactivate-carrying tile down 5;
            // destroy is neither (no shift). Draw-only; arrow_ys (gauge
            // chaining) keeps the unshifted level Y.
            let self_draw_dy = if tile_is_self.get(msg_idx).copied().unwrap_or(false) {
                if self_carry_act.get(msg_idx).copied().unwrap_or(false) {
                    -5.0
                } else if self_carry_deact.get(msg_idx).copied().flatten() == Some(false) {
                    5.0
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let draw_y = y + self_draw_dy;

            // Draw arrow first, then note(s) for this message
            // Skip drawing for hidden messages (they occupy Y space but aren't drawn)
            let is_hidden = msg_hidden.get(msg_idx).copied().unwrap_or(false);
            if !is_hidden {
                // Wrapper <g class="message" ...>
                let msg_num_str = format!("msg{}", msg_uid[msg_idx]);
                let p1_uid = participants[p1_idx].uid().to_string();
                let p2_uid = participants[p2_idx].uid().to_string();
                let source_line = msg_source_lines.get(msg_idx).copied().flatten();
                let line_str = source_line.map(|n| n.to_string());
                let mut msg_attrs: Vec<(&str, String)> = vec![
                    ("class", "message".to_string()),
                    ("id", msg_num_str),
                    ("data-entity-1", p1_uid),
                    ("data-entity-2", p2_uid),
                ];
                if let Some(ref ls) = line_str {
                    msg_attrs.push(("data-source-line", ls.clone()));
                }
                let msg_attrs_final: Vec<(&str, &str)> = msg_attrs.iter()
                    .map(|(k, v)| (*k, v.as_str())).collect();
                svg.open_group_with_attrs(&msg_attrs_final);

                let exo = msg_exo.get(msg_idx).copied().flatten();
                if let Some(exo_type) = exo {
                    draw_exo_message(
                        &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, draw_y,
                        p1_idx, exo_type,
                        msg_wrapped_lines.get(msg_idx).map_or(&[], std::vec::Vec::as_slice),
                        &skin,
                    );
                } else {
                    draw_message(
                        &mut svg, msg, &pos_c_vals, &bounder, &font_m, x_offset, draw_y,
                        p1_idx, p2_idx,
                        msg_self_levels.get(msg_idx).map_or(0, |&(li, _lc)| li),
                        msg_self_levels.get(msg_idx).map_or(0, |&(_li, lc)| lc),
                        msg_wrapped_lines.get(msg_idx).map_or(&[], std::vec::Vec::as_slice),
                        msg_p1_levels.get(msg_idx).copied().unwrap_or(0),
                        msg_p2_levels.get(msg_idx).copied().unwrap_or(0),
                        &skin,
                        false,
                        if autonumber { Some(auto_num_strings[msg_idx].as_str()) } else { None },
                        auto_num_widths.get(msg_idx).copied().unwrap_or(0.0),
                    );
                }
                svg.close_group();
            }
            // Attached notes ("note right:" / "note left:") share this message
            // tile and are positioned relative to the arrow.
            for note in notes {
                if note.msg_index != msg_idx || note.participant.is_some() {
                    continue;
                }
                draw_note(
                    &mut svg, note, msg, &pcode_to_idx, &pos_c_vals, x_offset,
                    draw_y, msg_idx, &msg_text_heights, &bounder, &font_m,
                    pre_wrapped_widths.get(msg_idx).copied().unwrap_or(0.0),
                    msg_p1_levels.get(msg_idx).copied().unwrap_or(0),
                    msg_p2_levels.get(msg_idx).copied().unwrap_or(0),
                    note_p1_levels.get(msg_idx).copied().unwrap_or(0),
                    note_p2_levels.get(msg_idx).copied().unwrap_or(0),
                    &skin,
                    None,
                    false,
                );
            }
            // Inline destroy X: a separate eventsList element emitted right
            // after its carrying message (not per participant).
            if let Some((d_pi, d_top)) = inline_destroy.get(msg_idx).copied().flatten() {
                let dcx = pos_c_vals[d_pi] + x_offset;
                let d_bot = d_top + 2.0 * DESTROY_X_HALF_SIZE;
                svg.set_fill_color("none");
                svg.set_stroke_color(Some(COLOR_DESTROY));
                svg.set_stroke_width(DESTROY_X_STROKE_WIDTH, None);
                svg.svg_line(dcx - DESTROY_X_HALF_SIZE, d_top, dcx + DESTROY_X_HALF_SIZE, d_bot, 0.0);
                svg.svg_line(dcx - DESTROY_X_HALF_SIZE, d_bot, dcx + DESTROY_X_HALF_SIZE, d_top, 0.0);
            }
            // below this message's arrow; they stack cumulatively within the gap.
            let mut stack_y = 0.0_f64;
            for note in notes {
                if note.msg_index != msg_idx || note.participant.is_none() {
                    continue;
                }
                draw_standalone_note(
                    &mut svg, note, &pos_c_vals, &pcode_to_idx, &head_widths, x_offset,
                    y, stack_y, &bounder, &font_m, &skin,
                );
                let line_count = note.text.split("\\n").count().max(1) as f64;
                stack_y += STANDALONE_NOTE_H + (line_count - 1.0) * STANDALONE_NOTE_LINE_H;
            }
            msg_idx += 1;
        }
    }

    // ── Draw legend, caption, footer (below footbox) ─────────────────────
    if legend_text.is_some() || caption_text.is_some() || footer_text.is_some() {
        // JAR stacks legend/caption/footer banner-edge to banner-edge. The
        // legend starts footbox_bottom+13 (+14 when the footbox is hidden),
        // its bordered block is ribbon_h(14)+10 tall with baseline at
        // 5-pad + ascent.
        let legend_start_gap = if hide_footbox { 14.0 } else { 13.0 };
        let mut below_y = footbox_bottom + legend_start_gap;

        // Legend
        if let Some(legend_txt) = legend_text {
            let font_l = UFont::sans_serif(legend_font_size as i32);
            let text_w = bounder.calculate_dimension(&font_l, legend_txt).width();
            let rect_w = text_w + 10.0;
            let rect_h = legend_dim.map_or(29.068, |(_, h)| h);
            let rect_x = (centering_width - rect_w) / 2.0;
            let text_x = rect_x + 5.0;
            let text_y = below_y + 5.0 + ribbon_asc(legend_font_size);

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
            let rect_h = caption_dim.map_or(ribbon_h(caption_font_size), |(_, h)| h);
            let rect_x = (centering_width - rect_w) / 2.0;
            let text_y = below_y + ribbon_asc(caption_font_size);

            let line_str = caption_line.map(|n| n.to_string());
            let mut attrs = vec![("class", "caption")];
            if let Some(ref ls) = line_str {
                attrs.push(("data-source-line", ls.as_str()));
            }
            svg.open_group_with_attrs(&attrs);
            // Background rect only when a style fill exists.
            if let Some(bg) = caption_bg_color.as_deref() {
                svg.set_fill_color(bg);
                svg.set_stroke_color(None);
                svg.set_stroke_width(1.0, None);
                svg.svg_rectangle(rect_x, below_y, rect_w, rect_h, 0.0, 0.0, 0.0);
            }
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
            let rect_h = footer_dim.map_or(ribbon_h(footer_font_size), |(_, h)| h);
            let rect_x = (centering_width - rect_w) / 2.0;
            let text_y = below_y + ribbon_asc(footer_font_size);

            let line_str = footer_line.map(|n| n.to_string());
            let mut attrs = vec![("class", "footer")];
            if let Some(ls) = &line_str {
                attrs.push(("data-source-line", ls.as_str()));
            }
            svg.open_group_with_attrs(&attrs);
            // Background rect only when a style fill exists.
            if let Some(bg) = footer_bg_color.as_deref() {
                svg.set_fill_color(bg);
                svg.set_stroke_color(None);
                svg.set_stroke_width(1.0, None);
                svg.svg_rectangle(rect_x, below_y, rect_w, rect_h, 0.0, 0.0, 0.0);
            }
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

#[allow(clippy::fn_params_excessive_bools)]
fn render_sequence_svg_teoz(
    diagram: &SequenceDiagram,
    svg_title: Option<&str>,
    svg_desc: Option<&str>,
    title: Option<&str>,
    title_line: Option<usize>,
    hide_footbox: bool,
    notes: &[NoteInfo],
    groups: &[GroupInfo],
    dividers: &[DividerInfo],
    autonumber: bool,
    msg_activates: &[Vec<String>],
    msg_deactivates: &[Vec<String>],
    msg_parallel: &[bool],
    max_message_size: Option<f64>,
    msg_exo: &[Option<ExoType>],
    msg_hidden: &[bool],
    skin_rose: bool,
    teoz: bool,
    arrow_color: Option<&str>,
    header_text: Option<&str>,
    header_line: Option<usize>,
    footer_text: Option<&str>,
    footer_line: Option<usize>,
    legend_text: Option<&str>,
    caption_text: Option<&str>,
    caption_line: Option<usize>,
    style_rules: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    participant_source_lines: &[Option<usize>],
    msg_source_lines: &[Option<usize>],
    teoz_toks: &[TeozTok],
) -> String {
    let bounder = StringBounderSvg::new(FileFormat::Svg);
    let font_p = UFont::sans_serif(FONT_SIZE_PARTICIPANT);
    let font_m = UFont::sans_serif(FONT_SIZE_MESSAGE);

    let participants = diagram.participants();
    // ── teoz TimeHook vertical layout ───────────────────────────────────
    // Contact line and group frames are driven from this byte-exact (vs the
    // JAR) cumulative tile layout rather than the legacy Puma gauge below.
    let teoz_tree = build_teoz_tree(teoz_toks);
    let mut teoz_n_msgs = 0usize;
    let mut teoz_is_self: Vec<bool> = Vec::new();
    // Number of wrapped label rows per message. A self-arrow reserves text
    // height even for an empty label (one implicit line).
    let mut teoz_msg_rows: Vec<usize> = Vec::new();
    // A non-self arrow with no label collapses to ph=14/cpr=6/zzz=8
    // (ComponentRoseArrow textHeight=2); labeled arrows keep ph=31.706.
    let mut teoz_msg_empty: Vec<bool> = Vec::new();
    for ev in diagram.events() {
        if let SequenceEvent::Message(m) = ev {
            let exo = msg_exo.get(teoz_n_msgs).copied().flatten();
            teoz_is_self.push(exo.is_none() && m.p1().code() == m.p2().code());
            let label = m.label();
            teoz_msg_empty.push(label.is_empty());
            let rows = if label.is_empty() {
                1
            } else if let Some(max_w) = max_message_size {
                wrap_message_text(&bounder, &font_m, label, max_w).len()
            } else {
                label.split("\\n").count()
            };
            teoz_msg_rows.push(rows.max(1));
            teoz_n_msgs += 1;
        }
    }
    let teoz_layout = layout_teoz(&teoz_tree, teoz_n_msgs, &teoz_is_self, &teoz_msg_rows, &teoz_msg_empty);
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
    // Own preferred head component heights (each includes its trailing pixel).
    // Box: ComponentRoseParticipant height = rect 33.0679 + 1 = 34.0679.
    // Actor: ComponentRoseActor.getPreferredHeight = stickman(60) + text(19.0679) = 79.0679.
    // Heads are bottom aligned (LivingSpaces.drawHeads uses VerticalAlignment.BOTTOM)
    // and the lifeline starts at origin + max(own heights).
    const ACTOR_OWN_HEIGHT: f64 = 79.0679;
    // Actor text block: preferred height 19.0679; baseline offset 14.9659.
    const ACTOR_TEXT_HEIGHT: f64 = 19.0679;
    const ACTOR_TEXT_BASELINE: f64 = 14.9659;
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
    let mut own_heights: Vec<f64> = Vec::with_capacity(participants.len());
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
            own_heights.push(ACTOR_OWN_HEIGHT);
        } else {
            // Java: getPreferredWidth = getTextWidth + margin.left + margin.right + deltaShadow
            // getTextWidth = text_width + padding.left + padding.right; padding=(5,7,5,7)
            // Must add padding.left and padding.right separately to match Java's float addition order
            let hw = text_w + PADDING_H + PADDING_H;
            head_widths.push(hw);
            preferred_widths.push(hw + delta_shadow);
            own_heights.push(TEXT_BLOCK_HEIGHT + HEIGHT_EXTRA);
        }
    }
    // Heads are bottom aligned to a common height: LivingSpaces.getHeadHeight
    // returns max(own preferred heights). Each component's preferred height
    // already includes its trailing pixel (box = rect 33.0679 + 1; actor =
    // stickman 60 + text 19.0679), so the lifeline offset is exactly common_own.
    let common_own = own_heights
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let head_layout_height = common_own;
    // head_rect_height: tail contribution to footbox_bottom. The tail is placed
    // one pixel above footbox_y (tail_y = footbox_y - 1), so a box tail adds its
    // rectangle height (33.0679) while an actor tail adds its full 79.0679.
    let has_actor = is_actor.iter().any(|&a| a);
    let head_rect_height = if has_actor {
        ACTOR_OWN_HEIGHT
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
    // Autonumber: bold13 number per message and its prefix width
    // (number advance + 4px gap). The prefix joins the tile reservation only
    // when the message also has a label; the number is drawn regardless.
    let auto_font_bold = UFont::sans_serif(FONT_SIZE_MESSAGE).with_style(FontStyle::bold());
    let auto_num_strings: Vec<String> = (0..pre_wrapped_widths.len())
        .map(|i| (i + 1).to_string())
        .collect();
    let auto_num_widths: Vec<f64> = auto_num_strings
        .iter()
        .map(|s| bounder.calculate_dimension(&auto_font_bold, s).width())
        .collect();
    let auto_prefix: Vec<f64> = auto_num_widths.iter().map(|w| w + 4.0).collect();
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
            let text_w = if autonumber && !label.is_empty() {
                text_w + auto_prefix[spacing_msg_idx]
            } else {
                text_w
            };
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
        // teoz (JAR 1.2026.6): GroupingTile.addConstraints only delegates to
        // child tiles; the frame never pushes following/preceding participants.
        // Frame X extent wraps content but is not a participant constraint.
        if teoz {
            continue;
        }
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
        // Leftmost head renders at PAGE_MARGIN past its solver value; Java places
        // it at x=20 and the frame edge 10px left of it, so floor posB at 15.
        let indent_floor = plantuml_real::add_fixed(&xorigin, 15.0);
        plantuml_real::ensure_bigger_than(&pos_b[leftmost], &indent_floor);

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
        // teoz (JAR 1.2026.6): GroupingTile.addConstraints only delegates to
        // its child tiles; group frames never push participants. Horizontal
        // separation of parallel groups emerges from their member messages.
        if false {
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
        let msg_count_total: usize = diagram.events().iter()
            .filter(|e| matches!(e, SequenceEvent::Message(_))).count();
        // teoz (JAR 1.2026.6): TileParallel.addConstraints only delegates to its
        // member tiles; horizontal separation of parallel siblings emerges from
        // the member messages' own addConstraints (CommunicationTile width and
        // CommunicationTileSelf comp-width) plus base participant spacing —
        // there is NO cross-sibling disjoint constraint. Emit every tile as
        // non-parallel so the legacy heuristic below adds nothing.
        let mut mi = 0usize;
        while mi < msg_count_total {
            if let Some(gi) = groups.iter().position(|g| g.msg_start == mi && g.nesting_level == 0 && g.group_type != "else") {
                let g = &groups[gi];
                tiles.push((g.msg_start, g.msg_end, false));
                mi = g.msg_end;
            } else {
                tiles.push((mi, mi + 1, false));
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
        title_width = bold14_width(title_text);
        // Java: `DecorateEntityImage` stacks the bordered title block above the
        // raw diagram; when the block (`bold` width + TITLE_BLOCK_W_EXTRA) is
        // wider than the participant span, participants are shifted right by
        // half the excess so the title can center on the raw diagram width.
        let participant_span = pos_d_vals.last().copied().unwrap_or(0.0)
            - pos_b_vals.first().copied().unwrap_or(0.0);
        let excess = title_width + TITLE_BLOCK_W_EXTRA - participant_span;
        if excess > 0.0 {
            let shift = excess / 2.0;
            for v in &mut pos_b_vals {
                *v += shift;
            }
            for v in &mut pos_c_vals {
                *v += shift;
            }
            for v in &mut pos_d_vals {
                *v += shift;
            }
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
    let title_dim = title.map(|t| (bold14_width(t) + TITLE_BLOCK_W_EXTRA, 40.0679));
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
    let title_height = if title.is_some() { TITLE_SHIFT_Y } else { 0.0 };
    // Header adds extra Y offset: header_height + HEADER_GAP replaces STARTING_Y.
    let header_extra = if header_text.is_some() {
        HEADER_FONT_SIZE + HEADER_GAP - STARTING_Y
    } else {
        0.0
    };
    let y_offset = PAGE_MARGIN + STARTING_Y + header_extra + title_height;
    // teoz heads sit at page y=10 (Puma uses 5); the lifeline band follows.
    let head_y = if teoz { 10.0 + header_extra + title_height } else { y_offset };
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
    // Total height of standalone notes ("note right of X: ...") sitting in the
    // gap AFTER each message index. Standalone notes are independent tiles in
    // the Y gauge and stack cumulatively (each one-line note adds
    // STANDALONE_NOTE_H), unlike attached notes ("note right:") which only
    // grow the same message tile via a maximum.
    let mut standalone_gap_h: Vec<f64> = vec![0.0; diagram.events().iter().filter(|e| matches!(e, SequenceEvent::Message(_))).count().max(1)];
    for note in notes {
        if note.participant.is_none() {
            continue;
        }
        if note.msg_index >= standalone_gap_h.len() {
            continue;
        }
        let line_count = note.text.split("\\n").count().max(1) as f64;
        let h = STANDALONE_NOTE_H + (line_count - 1.0) * STANDALONE_NOTE_LINE_H;
        standalone_gap_h[note.msg_index] += h;
    }
    // Divider tiles in the gap before each following message. A divider is an
    // independent tile (like a standalone note): add its full preferred height
    // to the gap after message `before_msg - 1`.
    let mut divider_gap_h: Vec<f64> = vec![0.0; standalone_gap_h.len()];
    for div in dividers {
        if div.before_msg > 0 {
            let gi = div.before_msg - 1;
            if gi < divider_gap_h.len() {
                divider_gap_h[gi] += DIVIDER_TILE_H;
            }
        }
    }
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            let _has_text = !msg.label().is_empty();
            // Use precomputed wrapped lines
            let wrapped_lines = pre_wrapped_lines[msg_idx].clone();
            let line_count = wrapped_lines.len();
            msg_wrapped_lines.push(wrapped_lines);
            let text_h = (line_count as f64) * MSG_LINE_HEIGHT + MSG_PADDING_TOP + MSG_PADDING_BOTTOM;
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
                    let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
                    current_y += inc + ELSE_TILE_HEIGHT;
                } else {
                    // Parallel message not first in a group: use the cluster's chaining point
                    current_y = cluster_start_y;
                }
            } else if msg_idx == 0 {
                // First message overall
                if teoz {
                    // teoz: PlayingSpace.startingY = 8 (Puma uses MSG_STARTING_Y 10).
                    current_y = lifeline_y + 8.0 + MSG_PADDING_Y + text_h;
                    if is_first_in_group {
                        let curr_level = curr_group
                            .and_then(|gi| groups.get(gi))
                            .map_or(0, |g| g.nesting_level);
                        // teoz reserves 33.706 per header (dimEmpty 19.706 +
                        // MARGINY_MAGIC/2 10 + EXTERNAL_MARGINY 4); Puma reserves 26.706.
                        current_y += 33.706 * (curr_level as f64 + 1.0) + header_extra;
                    }
                } else {
                    current_y = lifeline_y + MSG_STARTING_Y + MSG_PADDING_Y + text_h;
                    if is_first_in_group {
                        let curr_level = curr_group
                            .and_then(|gi| groups.get(gi))
                            .map_or(0, |g| g.nesting_level);
                        group_header_offset = GROUP_HEADER_OFFSET * (curr_level as f64 + 1.0) + header_extra;
                        current_y += group_header_offset;
                    }
                }
            } else if is_first_in_group && is_else {
                // Else section: normal increment + else tile height (no group gap/header)
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
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
                    let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
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
                    let inc = if let Some(nh) = prev_note_h { nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra) } else { text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra };
                    current_y += inc;
                    group_header_offset = GROUP_HEADER_OFFSET + header_extra;
                    current_y += group_header_offset;
                }
            } else if let Some(nh) = prev_note_h {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                current_y += nh.max(text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra);
            } else {
                let prev_self_extra = if prev_is_self { SELF_ARROW_HEIGHT } else { 0.0 };
                current_y += text_h + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y + prev_self_extra;
            }
            // Standalone notes between the previous message and this one are
            // independent gauge tiles: always add the previous gap's total.
            if msg_idx > 0 && !is_parallel {
                current_y += standalone_gap_h[msg_idx - 1];
            }
            if msg_idx > 0 && !is_parallel {
                current_y += divider_gap_h[msg_idx - 1];
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
            prev_note_h = notes.iter().find(|n| n.msg_index == msg_idx && n.participant.is_none()).map(|note| {
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
                    let last_note = notes.iter().find(|n| n.msg_index == last_mi && n.participant.is_none());
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
    // ── teoz: override contact Ys with the byte-exact TimeHook layout ─────
    // Drawn contact = lifeline origin + tile hook + contactRelative (23.706).
    for (i, slot) in arrow_ys.iter_mut().enumerate() {
        if let Some(hook) = teoz_layout.msg_hook.get(i).copied() {
            if hook.is_finite() {
                // For self-arrows the drawn anchor is the TOP hline:
                // base + paddingY(4) + textHeight, which grows one line pitch
                // per wrapped row. Normal messages use the contact line.
                let cpr = if teoz_is_self.get(i).copied().unwrap_or(false) {
                    let n = teoz_msg_rows.get(i).copied().unwrap_or(1).max(1) as f64;
                    4.0 + TEOZ_SELF_TEXT_ONE + (n - 1.0) * TEOZ_SELF_LINE_INC
                } else if teoz_msg_empty.get(i).copied().unwrap_or(false) {
                    TEOZ_MSG_EMPTY_CPR
                } else {
                    TEOZ_MSG_CPR
                };
                *slot = lifeline_y + hook + cpr;
            }
        }
    }

    // teoz destroy crosses: an inline `!!` centers its X on the carrying
    // message's contact line at the destroyed participant's lifeline. A
    // standalone destroy sits on its own zero-height LifeEvent tile hook.
    // teoz never truncates the lifeline/hover rect at the cross.
    let teoz_destroys: Vec<(usize, f64)> = if teoz {
        let mut v = Vec::new();
        let mut li = 0usize;
        for ev in diagram.events() {
            if let SequenceEvent::LifeEvent(le) = ev {
                let ev_idx = li;
                li += 1;
                if le.is_destroy() {
                    let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                    let y = if let Some(m) = le.message_index() {
                        arrow_ys.get(m).copied().unwrap_or(lifeline_y)
                    } else {
                        let hook = teoz_layout.life_hook.get(ev_idx).copied().filter(|h| h.is_finite()).unwrap_or(0.0);
                        lifeline_y + hook
                    };
                    v.push((p_idx, y));
                }
            }
        }
        v
    } else {
        Vec::new()
    };

    // ── teoz native frame geometry ──────────────────────────────────────
    // Per-message drawn X extent (solver space, no x_offset).
    let mut teoz_msg_x: Vec<(f64, f64)> = Vec::new();
    {
        let mut mi = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                let exo = msg_exo.get(mi).copied().flatten();
                let is_self = exo.is_none() && msg.p1().code() == msg.p2().code();
                let is_reverse = if is_self {
                    msg.arrow_config().is_reverse_define()
                } else if exo.is_some() {
                    false
                } else {
                    pos_c_vals[p1_idx] > pos_c_vals[p2_idx]
                };
                let p1_c = pos_c_vals[p1_idx];
                let p2_c = pos_c_vals[p2_idx];
                let (mut d_min, mut d_max) = if let Some(exo_type) = exo {
                    let label_w = pre_wrapped_widths.get(mi).copied().unwrap_or(0.0);
                    let exo_w = label_w + 24.0;
                    match exo_type {
                        ExoType::ToRight | ExoType::FromRight => (p1_c, p1_c + exo_w),
                        ExoType::FromLeft | ExoType::ToLeft => (p2_c - exo_w, p2_c),
                    }
                } else if is_self {
                    // Frame extent uses CommunicationTileSelf.getMin/MaxX:
                    // compWidth = max(50, label + 2*MESSAGE_TEXT_X_OFFSET),
                    // plus 5*maxLevel livebox on the exposed edge. This is
                    // the tile's layout extent (not the drawn 42px loop).
                    let label_w = pre_wrapped_widths.get(mi).copied().unwrap_or(0.0);
                    let max_act = *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                    let comp_w = (label_w + 2.0 * MESSAGE_TEXT_X_OFFSET).max(50.0);
                    if is_reverse {
                        let live_adj = if max_act > 0.0 { ACTIVATION_BAR_EXPLICIT_OFFSET } else { 0.0 };
                        (p1_c - comp_w - live_adj, p1_c + ACTIVATION_BAR_EXPLICIT_OFFSET * max_act)
                    } else {
                        (p1_c, p1_c + comp_w + ACTIVATION_BAR_EXPLICIT_OFFSET * max_act)
                    }
                } else {
                    (p1_c.min(p2_c), p1_c.max(p2_c))
                };
                if let Some(note) = notes.iter().find(|n| n.msg_index == mi && n.participant.is_none()) {
                    let note_text_w = max_line_width(&bounder, &font_m, &note.text);
                    let note_comp_w = note_text_w
                        + NOTE_OLD_PADDING_X1
                        + NOTE_OLD_PADDING_X2
                        + 2.0 * NOTE_PADDING_X
                        + delta_shadow;
                    if is_self {
                        match note.position {
                            NotePosition::Left if is_reverse => d_min = d_min.min(d_min - note_comp_w),
                            NotePosition::Right if !is_reverse => d_max = d_max.max(p1_c + 42.0 + note_comp_w),
                            // CommunicationTileSelfNoteRight.getMaxX =
                            // tile.getMaxX()(posC2) + noteWidth. posC2 = posC
                            // + (boxWidth/2)*max activation depth (LiveBoxes
                            // .getMaxPosition). The note starts at posC2.
                            NotePosition::Right => {
                                let max_lvl =
                                    *max_participant_levels.get(p1_idx).unwrap_or(&0) as f64;
                                d_max = d_max.max(
                                    p1_c + ACTIVATION_BAR_EXPLICIT_OFFSET * max_lvl + note_comp_w,
                                );
                            }
                            NotePosition::Left => d_min = d_min.min(p1_c - note_comp_w),
                        }
                    } else {
                        match note.position {
                            NotePosition::Right => {
                                let pi = if is_reverse { p1_idx } else { p2_idx };
                                d_max = d_max.max(pos_c_vals[pi] + note_comp_w);
                            }
                            NotePosition::Left => {
                                let pi = if is_reverse { p2_idx } else { p1_idx };
                                d_min = d_min.min(pos_c_vals[pi] - note_comp_w);
                            }
                        }
                    }
                }
                teoz_msg_x.push((d_min, d_max));
                mi += 1;
            }
        }
    }
    // Header component width per gid (bold13 keyword + 45, plus the 11px
    // bracketed comment when present).
    let mut teoz_header_w: Vec<f64> = vec![0.0; teoz_layout.group_pref.len()];
    for (gid, slot) in teoz_header_w.iter_mut().enumerate() {
        let kw = &teoz_layout.group_kind[gid];
        let comment = &teoz_layout.group_comment[gid];
        let tab_label = if kw == "group" {
            if comment.is_empty() { "group".to_string() } else { comment.clone() }
        } else {
            kw.clone()
        };
        let mut w = max_line_width(&bounder, &font_bold, &tab_label) + 45.0;
        if kw != "group" && !comment.is_empty() {
            let cond = format!("[{comment}]");
            let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
            w += GROUP_TEXT_PADDING + max_line_width(&bounder, &font_small, &cond);
        }
        *slot = w;
    }
    // Per-LifeEvent X extent (LifeEventTile.getMin/MaxX), keyed to the same
    // LifeEvent subsequence as `teoz_layout.life_hook`. Level is the nesting
    // depth AFTER this event (getLevelAt IGNORE_FUTURE_ACTIVATE):
    // min = posC - (level > 0 ? 5 : 0), max = posC + 5*level.
    let mut teoz_life_x: Vec<(f64, f64)> = Vec::new();
    {
        let mut level = vec![0i32; participants.len()];
        for ev in diagram.events() {
            if let SequenceEvent::LifeEvent(le) = ev {
                let p_idx = pcode_to_idx.get(le.participant().code()).copied().unwrap_or(0);
                if le.is_activate() {
                    level[p_idx] += 1;
                } else if le.is_deactivate() || le.is_destroy() {
                    level[p_idx] = (level[p_idx] - 1).max(0);
                }
                let lvl = level[p_idx];
                let c = pos_c_vals[p_idx];
                let mn = if lvl > 0 { c - ACTIVATION_BAR_EXPLICIT_OFFSET } else { c };
                let mx = c + ACTIVATION_BAR_EXPLICIT_OFFSET * lvl as f64;
                teoz_life_x.push((mn, mx));
            }
        }
    }
    // Internal frame X extents per gid.
    let mut teoz_frame_x: Vec<(f64, f64)> = vec![(0.0, 0.0); teoz_layout.group_pref.len()];
    for node in &teoz_tree {
        if let TeozNode::Group { gid, sections, .. } = node {
            teoz_frame_group(*gid, &collect_section_children(sections), &teoz_msg_x, &teoz_life_x, &teoz_header_w, &mut teoz_frame_x);
        }
    }
    // Minimum descendant message ordinal per gid — the point in the message
    // stream at which the JAR enters the group (draws header + all elses).
    fn teoz_min_msg(nodes: &[TeozNode]) -> Option<usize> {
        let mut m: Option<usize> = None;
        for node in nodes {
            match node {
                TeozNode::Msg { idx, .. } => m = Some(m.map_or(*idx, |v| v.min(*idx))),
                TeozNode::Group { sections, .. } => {
                    if let Some(v) = teoz_min_msg(&collect_section_children(sections)) {
                        m = Some(m.map_or(v, |cur| cur.min(v)));
                    }
                }
                TeozNode::Divider { .. } | TeozNode::Life { .. } => {}
            }
        }
        m
    }
    fn teoz_fill_anchors(nodes: &[TeozNode], anchor: &mut [Option<usize>]) {
        for node in nodes {
            if let TeozNode::Group { gid, sections, .. } = node {
                let children = collect_section_children(sections);
                anchor[*gid] = teoz_min_msg(&children);
                teoz_fill_anchors(&children, anchor);
            }
        }
    }
    let mut teoz_group_anchor: Vec<Option<usize>> = vec![None; teoz_layout.group_pref.len()];
    teoz_fill_anchors(&teoz_tree, &mut teoz_group_anchor);

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
        let last_note = notes.iter().find(|n| n.msg_index == last_mi && n.participant.is_none());
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
        // Frame wraps the participant head rectangles (± 10px), verified
        // against the jar: frame left = leftmost head x - 10, frame right =
        // rightmost head (x+w) + 10.
        let mut hd_min = f64::MAX;
        let mut hd_max = f64::MIN;
        let mut msg_iter_idx = 0usize;
        for event in diagram.events() {
            if let SequenceEvent::Message(msg) = event {
                if msg_iter_idx >= group.msg_start && msg_iter_idx < group.msg_end {
                    let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
                    let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(0);
                    for &pi in &[p1_idx, p2_idx] {
                        hd_min = hd_min.min(pos_b_vals[pi]);
                        hd_max = hd_max.max(pos_b_vals[pi] + head_widths[pi]);
                    }
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
                    let mi_note = notes.iter().find(|n| n.msg_index == msg_iter_idx && n.participant.is_none());
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
        } else if teoz {
            // teoz GroupingTile: frame top = first child contact - arrow
            // startY(23.706) - header reserve(33.706) + EXTERNAL_MARGINY(4);
            // frame bottom = last child contact + tile tail(8).
            let first_contact = arrow_ys[group.msg_start];
            let last_mi = (group.msg_end - 1).min(arrow_ys.len().saturating_sub(1));
            let last_contact = arrow_ys[last_mi];
            let fy = first_contact - 23.706 - 33.706 * (group.nesting_level as f64 + 1.0) + 4.0 - p_extra;
            let fh = last_contact + 8.0 - fy;
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
            let fy = arrow_ys[group.msg_start] - GROUP_HEADER_OFFSET * headers_to_subtract as f64 - first_text_h + 3.0 - p_extra;
            let fh = body_height + GROUP_HEADER_HEIGHT + GROUP_MARGIN_Y_MAGIC / 2.0 + p_extra;
            (fy, fh)
        };
        let (frame_x, frame_width) = if teoz {
            // teoz GroupingTile: frameMin = min(child drawnMinX) - MARGINX(16);
            // frameMax = max(child drawnMaxX + 16, frameMin + headerCompW + 16),
            // headerCompW = bold13 label width + 45.
            let hdr_label = if group.group_type == "group" {
                if group.comment.is_empty() { "group".to_string() } else { group.comment.clone() }
            } else {
                group.group_type.clone()
            };
            let header_comp_w = max_line_width(&bounder, &font_bold, &hdr_label) + 45.0;
            let fx0 = min_x - GROUP_MARGIN_X;
            let right_from_children = max_x + GROUP_MARGIN_X;
            let right_from_header = fx0 + header_comp_w + GROUP_MARGIN_X;
            let frame_max = right_from_children.max(right_from_header);
            (fx0, frame_max - fx0)
        } else {
            (hd_min - 10.0, hd_max - hd_min + 20.0)
        };
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
        let tab_label_w = max_line_width(&bounder, &font_bold, &tab_label);
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
            let (efx, _efy, efw, _efh) = group_frames[gi];
            let new_left = pfx.min(efx);
            let new_right = (pfx + pfw).max(efx + efw);
            let new_width = new_right - new_left;
            let else_bottom = arrow_ys[groups[gi].msg_end - 1] + 8.0;
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
        let header_max = fx + header_widths[gi] + PAGE_MARGIN;
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
            let (efx, _efy, efw, _efh) = group_frames[gi];
            let new_left = pfx.min(efx);
            let new_right = (pfx + pfw).max(efx + efw);
            let new_width = new_right - new_left;
            // Legacy frame ends 8px below the else section's last arrow.
            let else_last = groups[gi].msg_end - 1;
            let else_bottom = arrow_ys[else_last] + 8.0;
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

    // Compute note heights for each message
    let mut note_heights: Vec<f64> = vec![0.0; arrow_ys.len()];
    for note in notes {
        if note.participant.is_none() && note.msg_index < note_heights.len() {
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
            let msg_lines = msg_text_heights.get(i).map_or(1, |&h| ((h - MSG_PADDING_TOP - MSG_PADDING_BOTTOM) / MSG_LINE_HEIGHT) as usize);
            let note_y_top = arrow_ys[i] - ARROW_Y_BASE - (msg_lines.saturating_sub(1) as f64) * MSG_LINE_HEIGHT;
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
    // Standalone notes after the last message extend the lifeline by their
    // total tile height (the Y gauge max + paddingY chain).
    let standalone_lifeline_bottom = normal_lifeline_bottom
        + standalone_gap_h.get(last_arrow_idx).copied().unwrap_or(0.0);
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
                // Lifeline runs to the last message's Y-gauge bottom.
                // teoz: the trailing EmptyTile after a group extends the body a
                // further 7px (contact + 32); Puma uses preferred height 25.
                let tail_gap = if teoz { 32.0 } else { java_preferred_height };
                let group_msg_ygauge_max = arrow_ys.iter().enumerate()
                    .filter(|&(mi, _)| mi >= g.msg_start && mi < g.msg_end)
                    .map(|(_, &y)| y + tail_gap)
                    .fold(frame_bottom + GROUP_MARGIN_Y, f64::max);
                group_msg_ygauge_max
            };
            val
        })
        .fold(0.0_f64, f64::max);
    // When parallel messages are present, cap normal_lifeline_bottom at msg_ygauge_max + PAGE_MARGIN
    // to avoid 1px overshoot from text_h=26 vs Java's 25 in self-messages.
    // Also cap at group_lifeline_bottom when the max-arrow-Y message is inside a parallel group,
    // because its height is already accounted for by the group's own lifeline_bottom.
    let max_arrow_in_parallel_group = groups.iter().any(|g| g.parallel && last_arrow_idx >= g.msg_start && last_arrow_idx < g.msg_end);
    let _normal_lifeline_bottom = if has_parallel {
        let capped = normal_lifeline_bottom.min(msg_ygauge_max + PAGE_MARGIN);
        if max_arrow_in_parallel_group && group_lifeline_bottom > 0.0 {
            capped.min(group_lifeline_bottom)
        } else {
            capped
        }
    } else {
        normal_lifeline_bottom
    };
    let lifeline_bottom = if teoz {
        lifeline_y + teoz_layout.final_y + 10.0
    } else {
        standalone_lifeline_bottom.max(note_lifeline_bottom).max(group_lifeline_bottom)
    };
    // Flush remaining pending activations (autoactivate without explicit deactivate)
    for (pi, stack) in pending_activations.iter().enumerate() {
        for &(start_y, level) in stack {
            activations.push((pi, start_y, lifeline_bottom, level));
        }
    }
    // ── teoz livebox (activation) segments ──────────────────────────────
    // LiveBoxesDrawer emits one white width-10 box per activation level,
    // anchored at the triggering message's teoz contact Y. Autoactivated
    // boxes (no explicit deactivate) run to the lifeline bottom. Each box is
    // drawn as a bare `<g><title/>…</g>` sibling right after its participant.
    #[derive(Clone, Copy)]
    enum TeozLiveOp {
        /// White activation rect for one level run (top screen Y, bottom, level).
        Rect(f64, f64, i32),
        /// Destroy cross centered at this screen Y (emitted inline at the run
        /// it closes, matching LiveBoxes.drawOneLevel.drawDestroyIfNeeded).
        Cross(f64),
    }
    let mut teoz_liveboxes: Vec<Vec<TeozLiveOp>> = vec![Vec::new(); participants.len()];
    // Screen Y of each destroy cross emitted inline at a level run.
    let mut teoz_inline_cross_y: Vec<f64> = Vec::new();
    if teoz {
        // Faithful port of LiveBoxes.getStairs / getLevelAt (JAR 1.2026.6).
        // We walk diagram.events() in order, attributing each event a gauge Y
        // (origin at the lifeline top): normal messages add one step at the
        // contact line for both endpoints; self messages add a single step at
        // the bottom (activate) / top (deactivate) hline; a LifeEvent adds a
        // step at its own (possibly parallel-cluster-adjusted) tile hook.
        let npart = participants.len();
        let pid = |code: &str| pcode_to_idx.get(code).copied().unwrap_or(0);
        struct Ev {
            is_msg: bool,
            mi: usize,
            li: usize,
            parts: Vec<usize>,
        }
        let mut evs: Vec<Ev> = Vec::new();
        let mut mi_run = 0usize;
        let mut li_run = 0usize;
        for ev in diagram.events() {
            match ev {
                SequenceEvent::Message(m) => {
                    let parts = if m.is_self_message() {
                        vec![pid(m.p1().code())]
                    } else {
                        vec![pid(m.p1().code()), pid(m.p2().code())]
                    };
                    evs.push(Ev { is_msg: true, mi: mi_run, li: 0, parts });
                    mi_run += 1;
                }
                SequenceEvent::LifeEvent(l) => {
                    evs.push(Ev {
                        is_msg: false,
                        mi: 0,
                        li: li_run,
                        parts: vec![pid(l.participant().code())],
                    });
                    li_run += 1;
                }
            }
        }
        // Effective attached message of an LE = nearest preceding message
        // (Java SequenceDiagram.activate binds it to the last EventWithDeactivate
        // unconditionally, including an arrow that does not touch the participant).
        let mut attach: Vec<Option<usize>> = Vec::with_capacity(evs.len());
        let mut last_msg: Option<usize> = None;
        for e in &evs {
            if e.is_msg {
                attach.push(None);
                last_msg = Some(e.mi);
            } else {
                attach.push(last_msg);
            }
        }
        // Parallel brother of a `&` message is the immediately preceding message.
        let par_brother = |mi: usize| -> Option<usize> {
            if mi > 0 && msg_parallel.get(mi).copied().unwrap_or(false) {
                Some(mi - 1)
            } else {
                None
            }
        };
        fn par_with(brother: &impl Fn(usize) -> Option<usize>, a: usize, b: usize) -> bool {
            if a == b {
                return true;
            }
            match brother(a) {
                Some(br) => par_with(brother, br, b),
                None => false,
            }
        }
        let msg_deals = |mi: usize, p: usize| -> bool {
            evs.iter().find(|e| e.is_msg && e.mi == mi).is_some_and(|e| e.parts.contains(&p))
        };
        // Gauge step Y per participant keyed by global event index, with the
        // +5 nudge for a deactivate landing on an already-used Y (LiveBoxes.addStep).
        let mut step_y: Vec<std::collections::HashMap<usize, f64>> = vec![std::collections::HashMap::new(); npart];
        let origin = lifeline_y;
        for (ei, e) in evs.iter().enumerate() {
            if e.is_msg {
                let mi = e.mi;
                if teoz_is_self.get(mi).copied().unwrap_or(false) {
                    // CommunicationTileSelf: one step on p1 only, gated on the
                    // presence of an attached activate / deactivate-or-destroy.
                    let has_act = evs.iter().enumerate().any(|(k, l)| {
                        !l.is_msg
                            && attach[k] == Some(mi)
                            && l.parts == e.parts
                            && diagram.events().get(k).is_some_and(|ev| {
                                matches!(ev, SequenceEvent::LifeEvent(le) if le.is_activate())
                            })
                    });
                    let has_off = evs.iter().enumerate().any(|(k, l)| {
                        !l.is_msg
                            && attach[k] == Some(mi)
                            && l.parts == e.parts
                            && diagram.events().get(k).is_some_and(|ev| {
                                matches!(ev, SequenceEvent::LifeEvent(le) if le.is_deactivate() || le.is_destroy())
                            })
                    });
                    let hook = teoz_layout.msg_hook.get(mi).copied().filter(|h| h.is_finite()).unwrap_or(0.0);
                    let n = teoz_msg_rows.get(mi).copied().unwrap_or(1).max(1) as f64;
                    let text_h = TEOZ_SELF_TEXT_ONE + (n - 1.0) * TEOZ_SELF_LINE_INC;
                    let top = hook + text_h + 4.0;
                    let bottom = hook + text_h + 17.0;
                    if has_act {
                        *step_y[e.parts[0]].entry(ei).or_insert(bottom) = bottom;
                    } else if has_off {
                        *step_y[e.parts[0]].entry(ei).or_insert(top) = top;
                    }
                } else {
                    let yg = arrow_ys.get(mi).copied().unwrap_or(lifeline_bottom) - origin;
                    for &p in &e.parts {
                        step_y[p].insert(ei, yg);
                    }
                }
            } else {
                let li = e.li;
                if let Some(hook) = teoz_layout.life_hook.get(li).copied().filter(|h| h.is_finite()) {
                    let p = e.parts[0];
                    let is_deact = matches!(
                        diagram.events().get(ei),
                        Some(SequenceEvent::LifeEvent(le)) if le.is_deactivate()
                    );
                    let mut y = hook;
                    if is_deact && step_y[p].values().any(|&v| (v - y).abs() < 1e-9) {
                        y += 5.0;
                    }
                    step_y[p].insert(ei, y);
                }
            }
        }
        // getLevelAt(event, CONSIDER_FUTURE_DEACTIVATE) for one participant.
        let level_at = |target: usize, p: usize| -> i32 {
            let mut level = 0i32;
            for (ei, e) in evs.iter().enumerate() {
                if !e.is_msg
                    && e.parts[0] == p {
                        if let Some(SequenceEvent::LifeEvent(le)) = diagram.events().get(ei) {
                            if le.is_activate() {
                                level += 1;
                            } else if le.is_deactivate() || le.is_destroy() {
                                level = (level - 1).max(0);
                            }
                        }
                    }
                if ei == target {
                    if e.is_msg {
                        let tmi = e.mi;
                        let mut seen_act = false;
                        let mut seen_deact = false;
                        for (k, l) in evs.iter().enumerate().skip(target + 1) {
                            if l.is_msg {
                                break;
                            }
                            let am = attach[k];
                            let linked = am == Some(tmi)
                                || am.is_some_and(|am_mi| par_with(&par_brother, am_mi, tmi));
                            if !linked || l.parts[0] != p || !msg_deals(tmi, p) {
                                continue;
                            }
                            if let Some(SequenceEvent::LifeEvent(le)) = diagram.events().get(k) {
                                if le.is_activate() {
                                    seen_act = true;
                                    if seen_deact {
                                        break;
                                    }
                                    level += 1;
                                } else if le.is_deactivate() || le.is_destroy() {
                                    seen_deact = true;
                                    if seen_act {
                                        break;
                                    }
                                    level = (level - 1).max(0);
                                }
                            }
                        }
                    }
                    return level.max(0);
                }
            }
            0
        };
        let total_h = lifeline_bottom - origin;
        for p in 0..npart {
            // getStairs walk producing monotonic (value, indent) steps.
            let mut raw: Vec<(f64, i32, bool)> = Vec::new();
            let mut position: Option<f64> = None;
            let mut last_message: Option<usize> = None;
            let mut seen_act = false;
            let mut seen_deact = false;
            let mut indent = 0i32;
            for (ei, e) in evs.iter().enumerate() {
                let potential = step_y[p].get(&ei).copied();
                if position.is_none() || last_message.is_none() {
                    position = potential;
                } else if !e.is_msg {
                    let deals_p = e.parts[0] == p;
                    if !deals_p {
                        continue;
                    }
                    let le_msg = attach[ei];
                    let coalesce = le_msg.is_some_and(|am| {
                        last_message.is_some_and(|lm| par_with(&par_brother, am, lm))
                    });
                    let le_act = matches!(diagram.events().get(ei), Some(SequenceEvent::LifeEvent(le)) if le.is_activate());
                    let le_deact = matches!(diagram.events().get(ei), Some(SequenceEvent::LifeEvent(le)) if le.is_deactivate() || le.is_destroy());
                    if !coalesce || (le_act && seen_deact) || (le_deact && seen_act) {
                        position = potential;
                    }
                    seen_act |= le_act;
                    seen_deact |= le_deact;
                } else {
                    position = potential;
                }
                if e.is_msg {
                    if let Some(lm) = last_message {
                        if par_with(&par_brother, e.mi, lm) {
                            continue;
                        }
                    }
                    seen_act = false;
                    seen_deact = false;
                    last_message = Some(e.mi);
                }
                let Some(pos) = position else { continue };
                indent = level_at(ei, p);
                // Step.isDestroy: a message immediately followed (skipping
                // notes) by a destroy LifeEvent on this participant.
                let is_destroy = e.is_msg
                    && evs.get(ei + 1).is_some_and(|n| {
                        !n.is_msg
                            && n.parts[0] == p
                            && matches!(diagram.events().get(ei + 1),
                                Some(SequenceEvent::LifeEvent(le)) if le.is_destroy())
                    });
                let val = pos.max(0.0);
                raw.push((val, indent, is_destroy));
            }
            // Stairs.addStep drops any step whose Y is below the last accepted Y.
            let mut stairs: Vec<(f64, i32, bool)> = Vec::new();
            for (val, ind, is_destr) in raw {
                if ind < 0 {
                    continue;
                }
                if let Some(&(last_y, _, _)) = stairs.last() {
                    if val < last_y {
                        continue;
                    }
                }
                stairs.push((val, ind, is_destr));
            }
            stairs.push((total_h, indent, false));
            let max_indent = stairs.iter().map(|&(_, i, _)| i).max().unwrap_or(0);
            // drawOneLevel per level 1..=maxIndent: one rect per level run, and
            // when the step that closes a run is a destroy message, the cross
            // is emitted immediately after that rect (drawDestroyIfNeeded).
            for level in 1..=max_indent {
                let mut pending = true;
                let mut start = 0.0;
                for (idx, &(val, ind, is_destr)) in stairs.iter().enumerate() {
                    if pending && ind == level {
                        start = val;
                        pending = false;
                    } else if !pending {
                        let has_next = idx + 1 < stairs.len();
                        if !has_next || ind < level {
                            teoz_liveboxes[p].push(TeozLiveOp::Rect(start + origin, val + origin, level));
                            if is_destr {
                                let cy = val + origin;
                                teoz_liveboxes[p].push(TeozLiveOp::Cross(cy));
                                teoz_inline_cross_y.push(cy);
                            }
                            pending = true;
                        }
                    }
                }
            }
        }
    }

    let lifeline_height = if arrow_ys.is_empty() {
        LIFELINE_HEIGHT_BASE
    } else {
        lifeline_bottom - lifeline_y
    };
    let footbox_y = lifeline_y + lifeline_height;
    let tail_y = footbox_y - 1.0;

    // ── Compute total dimensions ──────────────────────────────────────────
    // First pass: compute min_layout_left using solver values (without x_offset)
    // to determine how far left notes extend, then set x_offset accordingly.
    let mut min_layout_left = 0.0_f64;
    for note in notes {
        if note.participant.is_some() || note.msg_index >= arrow_ys.len() {
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
    // teoz derives the global translate from the ROOT PlayingSpace public
    // minimum: a bare message contributes its drawn min-X, a group tile its
    // public extent (internal fmin − 3). The legacy group_frames/loop scan
    // above under-counts encapsulated reverse-self tiles.
    let teoz_root_min = if teoz {
        let mut ml = 0.0_f64;
        for node in &teoz_tree {
            match node {
                TeozNode::Msg { idx, .. } => {
                    let (a, _) = teoz_msg_x.get(*idx).copied().unwrap_or((0.0, 0.0));
                    ml = ml.min(a);
                }
                TeozNode::Group { gid, .. } => {
                    let (fmin, _) = teoz_frame_x[*gid];
                    ml = ml.min(fmin - GROUP_EXTERNAL_MARGIN_X1);
                }
                TeozNode::Divider { .. } | TeozNode::Life { .. } => {}
            }
        }
        ml
    } else {
        0.0
    };
    let x_offset = if teoz {
        10.0 - teoz_root_min.min(0.0)
    } else {
        PAGE_MARGIN - min_left.min(0.0)
    };
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
        if note.participant.is_some() || note.msg_index >= arrow_ys.len() {
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
    // Standalone notes use a different (tile-level) geometry; compute their
    // right edges independently for the canvas width.
    let mut max_standalone_right = 0.0_f64;
    for note in notes {
        if note.participant.is_none() {
            continue;
        }
        let target = note.participant.as_deref().unwrap_or("");
        let code = target.split(',').next().unwrap_or(target);
        let p_idx = pcode_to_idx.get(code).copied().unwrap_or(0);
        let text_w = max_line_width(&bounder, &font_m, &note.text);
        let polygon_w = (text_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2).trunc();
        let polygon_x = (pos_c_vals[p_idx] + x_offset + NOTE_PADDING_X).round();
        max_standalone_right = max_standalone_right.max(polygon_x + polygon_w + 1.0);
    }
    let max_frame_right: f64 = group_frames
        .iter()
        .map(|&(fx, _, fw, _)| fx + fw + PAGE_MARGIN + x_offset)
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
    // `CommunicationTileSelf.getMaxX` = posC2 + compWidth (the loop's layout
    // reservation to the right; includes unused slack when the label is short).
    let self_tile_right: f64 = max_forward_comp_width
        .iter()
        .enumerate()
        .map(|(p, &cw)| {
            let pos_c2 = pos_c_vals[p]
                + 5.0 * max_participant_levels.get(p).copied().unwrap_or(0) as f64;
            pos_c2 + cw
        })
        .fold(0.0_f64, f64::max);
    let content_right = rightmost_x
        .max(title_rightmost)
        .max(max_note_right)
        .max(max_frame_right)
        .max(self_tile_right)
        .max(max_standalone_right);
    let has_extra_elements = header_text.is_some() || footer_text.is_some() || legend_text.is_some() || caption_text.is_some();
    let total_width = if has_extra_elements {
        content_right.max(max_element_width + PAGE_MARGIN * 2.0) + 2.0 * PAGE_MARGIN
    } else if title_width > 0.0 {
        content_right + 2.0 * PAGE_MARGIN + 1.0
    } else {
        content_right + 2.0 * PAGE_MARGIN
    };

    // Override partition frame dimensions to span the full diagram width
    let full_right = rightmost_x.max(max_note_right).max(max_frame_right);
    for (gi, group) in groups.iter().enumerate() {
        if group.group_type == "partition" {
            let (_, fy, _, fh) = group_frames[gi];
            let frame_x = PAGE_MARGIN - x_offset;
            let frame_width = full_right - PAGE_MARGIN;
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
        footbox_bottom + PAGE_MARGIN + height_extra + delta_shadow
    };
    let (total_width, total_height) = if teoz {
        // teoz page: TextBlock dims = body (maxX-minX) + 10 each side, then the
        // exporter's ensureVisible floors the far edge and adds its trailing
        // pixel plus margin. Width derives from solver extents; height from the
        // footbox bottom.
        // Root PlayingSpace extent: only top-level tiles count. A bare
        // message contributes its drawn extent; a group tile contributes
        // its public extent (internal frame min−3 / max+9). Messages nested
        // in a group are encapsulated and never escape the frame.
        let mut ps_max = f64::MIN;
        for node in &teoz_tree {
            match node {
                TeozNode::Msg { idx, .. } => {
                    let (_a, b) = teoz_msg_x.get(*idx).copied().unwrap_or((0.0, 0.0));
                    ps_max = ps_max.max(b);
                }
                TeozNode::Group { gid, .. } => {
                    let (_fmin, fmax) = teoz_frame_x[*gid];
                    ps_max = ps_max.max(fmax + GROUP_EXTERNAL_MARGIN_X2);
                }
                TeozNode::Divider { .. } | TeozNode::Life { .. } => {}
            }
        }
        ps_max = ps_max.max(pos_d_vals.last().copied().unwrap_or(0.0));
        let mut min_left = f64::MAX;
        for node in &teoz_tree {
            match node {
                TeozNode::Msg { idx, .. } => {
                    let (a, _b) = teoz_msg_x.get(*idx).copied().unwrap_or((0.0, 0.0));
                    min_left = min_left.min(a);
                }
                TeozNode::Group { gid, .. } => {
                    let (fmin, _fmax) = teoz_frame_x[*gid];
                    min_left = min_left.min(fmin - GROUP_EXTERNAL_MARGIN_X1);
                }
                TeozNode::Divider { .. } | TeozNode::Life { .. } => {}
            }
        }
        min_left = min_left.min(0.0);
        let w = (ps_max - min_left + 10.0).floor() + 11.0;
        let h = if hide_footbox {
            (lifeline_bottom).floor() + 11.0
        } else {
            // TextBlock bottom reserves the participant tail's shadow:
            // ComponentRoseParticipant prefHeight adds deltaShadow(rose=4)+1.
            // delta_shadow=0 reproduces the previous floor()+12 exactly. An
            // Actor tail has no trailing +1 pixel (pref 79.0679, not 80.0679).
            let trailing = if has_actor { 0.0 } else { 1.0 };
            (footbox_bottom + delta_shadow + trailing).floor() + 11.0
        };
        (w, h)
    } else {
        (total_width, total_height)
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
    // Centering width = rightmost_x + PAGE_MARGIN - 1 (matches Java layout).
    let centering_width = rightmost_x + PAGE_MARGIN - 1.0;
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
        let text_w = bold14_width(title_text);
        let rect_w = text_w + 10.0; // 5px padding each side
        let rect_h = 29.0679;
        let rect_y = PAGE_MARGIN + STARTING_Y + header_extra + 5.0;
        // Java centers the bordered title block on the raw diagram width; the
        // text sits 5px inside the block. With no header/footer/legend/caption
        // the block starts at x = PAGE_MARGIN (5); with extras it is centered.
        let (rect_x, text_x) = if has_extra_elements {
            let rx = (centering_width - rect_w) / 2.0;
            (rx, rx + 5.0)
        } else {
            (PAGE_MARGIN, PAGE_MARGIN + 5.0)
        };
        let text_y = rect_y + TITLE_TEXT_OFFSET;
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
        if teoz {
            // teoz frames/tabs/fills are emitted natively from the TimeHook
            // layout before the message loop; skip the legacy flat pass.
            continue;
        }
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

    // teoz draws every group backcolor as one background rect in a single
    // pre-pass (GroupingTile.drawCompBackground) before the lifelines; the
    // frame border is drawn later interleaved with the messages.
    if teoz {
        for (gid, bc) in teoz_layout.group_backcolor.iter().enumerate() {
            if let Some(bc) = bc {
                let (fmin, fmax) = teoz_frame_x[gid];
                let fx = fmin + x_offset;
                let fw = fmax - fmin;
                let fy = lifeline_y + teoz_layout.group_hook[gid];
                let fh = teoz_layout.group_pref[gid] - 10.0;
                svg.set_fill_color(bc);
                svg.set_stroke_color(Some(bc));
                svg.set_stroke_width(1.0, None);
                svg.svg_rectangle(fx, fy, fw, fh, 0.0, 0.0, 0.0);
            }
        }
    }

    // ── Draw explicit activation boxes (first pass, behind lifelines) ─────
    if !teoz {
        draw_activation_bars(&mut svg, &activations, participants, &pos_c_vals, x_offset);
    }

    // ── Draw lifelines (background) ───────────────────────────────────────
    // ── Draw lifelines + activation bars + destroy marks (per participant) ──
    for (i, p) in participants.iter().enumerate() {
        let cx = pos_c_vals[i] + x_offset;
        // Java's ComponentRoseLine draws the line at posB + (int)(head_width/2),
        // truncating the fractional part — NOT at posC = posB + head_width/2.
        // teoz draws lifelines in a width-1 area translated to the TRUE posC,
        // so the line is untruncated and the 8px hover rect starts at posC-3.5.
        let line_x = if teoz {
            cx
        } else {
            pos_b_vals[i] + ((head_widths[i] / 2.0) as i64) as f64 + x_offset
        };
        let ly = lifeline_y;

        // Check if this participant is destroyed
        let destroy_y = if teoz { None } else {
            destroys.iter().find(|(pi, _)| *pi == i).map(|(_, y)| *y)
        };
        let ll_height = if let Some(dy) = destroy_y {
            dy + DESTROY_X_HALF_SIZE - 1.0 - ly
        } else {
            lifeline_height
        };

        if teoz {
            // teoz: one bare <g><title>CODE</title>…</g>, no id/class wrappers.
            svg.open_group(None);
            svg.title(p.display());
        } else {
            // Wrapper <g class="participant-lifeline" ...>
            let uid = p.uid();
            let source_line = participant_source_lines.get(i).copied().flatten();
            let line_str = source_line.map(|n| n.to_string());
            let mut ll_attrs: Vec<(&str, String)> = vec![
                ("class", "participant-lifeline".to_string()),
                ("id", format!("{uid}-lifeline")),
                ("data-qualified-name", p.code().to_string()),
                ("data-entity-uid", uid.to_string()),
            ];
            if let Some(ls) = &line_str {
                ll_attrs.push(("data-source-line", ls.clone()));
            }
            let ll_attrs_final: Vec<(&str, &str)> = ll_attrs.iter()
                .map(|(k, v)| (*k, v.as_str())).collect();
            svg.open_group_with_attrs(&ll_attrs_final);

            // Inner <g> with <title>, rect, line
            svg.open_group(None);
            svg.title(p.display());
        }
        // Activation bar / hover rect (transparent)
        svg.set_fill_color(COLOR_ACTIVATION_BAR);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.svg_rectangle(
            cx - if teoz { 3.5 } else { ACTIVATION_BAR_OFFSET },
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
        svg.svg_line(line_x, ly, line_x, ly + ll_height, 0.0);

        svg.close_group();
        if !teoz {
            svg.close_group();
        }

        // teoz: liveboxes follow the lifeline as separate bare
        // `<g><title/>white-rect</g>` groups (LiveBoxesDrawer).
        if teoz {
            for op in &teoz_liveboxes[i] {
                match *op {
                    TeozLiveOp::Rect(sy, ey, level) => {
                        svg.open_group(None);
                        svg.title("");
                        svg.set_fill_color("#FFFFFF");
                        svg.set_stroke_color(Some(color_stroke));
                        svg.set_stroke_width(1.0, None);
                        svg.set_filter(skin.shadow_filter_id.as_deref());
                        let level_dx = (level as f64 - 1.0) * ACTIVATION_BAR_EXPLICIT_OFFSET;
                        svg.svg_rectangle(
                            cx - ACTIVATION_BAR_EXPLICIT_OFFSET + level_dx,
                            sy,
                            ACTIVATION_BAR_EXPLICIT_WIDTH,
                            ey - sy,
                            0.0,
                            0.0,
                            0.0,
                        );
                        svg.set_filter(None);
                        svg.close_group();
                    }
                    TeozLiveOp::Cross(dy) => {
                        svg.set_fill_color("none");
                        svg.set_stroke_color(Some(COLOR_DESTROY));
                        svg.set_stroke_width(DESTROY_X_STROKE_WIDTH, None);
                        svg.svg_line(cx - DESTROY_X_HALF_SIZE, dy - DESTROY_X_HALF_SIZE, cx + DESTROY_X_HALF_SIZE, dy + DESTROY_X_HALF_SIZE, 0.0);
                        svg.svg_line(cx - DESTROY_X_HALF_SIZE, dy + DESTROY_X_HALF_SIZE, cx + DESTROY_X_HALF_SIZE, dy - DESTROY_X_HALF_SIZE, 0.0);
                    }
                }
            }
        }

        // Draw destroy X marks for this participant. Inline destroys are
        // already emitted as Cross ops inside the livebox level stream.
        let cross_list: &[(usize, f64)] = if teoz { &teoz_destroys } else { &destroys };
        for &(pi, dy) in cross_list {
            if pi != i {
                continue;
            }
            if teoz && teoz_inline_cross_y.iter().any(|&y| (y - dy).abs() < 1e-6) {
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
    if teoz {
        // teoz emits ALL heads as bare rect+text pairs first, then ALL tails;
        // no participant-* wrappers.
        let draw_box = |svg: &mut SvgGraphics,
                        i: usize,
                        y: f64,
                        baseline: f64| {
            let x = pos_b_vals[i] + x_offset;
            let w = head_widths[i];
            svg.set_fill_color(color_back);
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(stroke_width_head, None);
            svg.set_filter(skin.shadow_filter_id.as_deref());
            svg.svg_rectangle(x, y, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
            svg.set_filter(None);
            let text_w = bounder.calculate_dimension(&font_p, participants[i].display()).width();
            let text_x = x + (w - text_w) / 2.0;
            svg.set_fill_color(COLOR_TEXT);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            svg.text(
                participants[i].display(),
                text_x,
                baseline,
                None,
                FONT_SIZE_PARTICIPANT,
                None,
                None,
                None,
                text_w,
                &indexmap::IndexMap::new(),
                None,
            );
        };
        // Actor chrome (ComponentRoseActor): text first, then the head
        // ellipse and the body/arms/legs stickman path. Head and tail use the
        // same 79.0679 band anchored at `y` (stickman on top, label below).
        let draw_actor = |svg: &mut SvgGraphics, i: usize, y: f64| {
            let cx = pos_c_vals[i] + x_offset;
            let text_w = bounder.calculate_dimension(&font_p, participants[i].display()).width();
            let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
            let text_y = y + STICKMAN_HEIGHT + ACTOR_TEXT_BASELINE;
            svg.set_fill_color(COLOR_TEXT);
            svg.set_stroke_color(None);
            svg.set_stroke_width(0.0, None);
            svg.text(
                participants[i].display(),
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
            let head_cy = y + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
            svg.set_fill_color(color_back);
            svg.set_stroke_color(Some(color_stroke));
            svg.set_stroke_width(STICKMAN_THICKNESS, None);
            svg.svg_ellipse(cx, head_cy, STICKMAN_HEAD_DIAM / 2.0, STICKMAN_HEAD_DIAM / 2.0, 0.0);
            let body_top = head_cy + STICKMAN_HEAD_DIAM / 2.0;
            let body_bottom = body_top + STICKMAN_BODY_LEN;
            let arms_y = body_top + 8.0;
            let legs_bottom = body_bottom + STICKMAN_LEGS_Y;
            svg.set_fill_color("none");
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
        };
        // Heads: rect at head_y, text baseline head_y + 21.9659.
        for (i, &actor) in is_actor.iter().enumerate() {
            if actor {
                let hy = head_y + (common_own - own_heights[i]);
                draw_actor(&mut svg, i, hy);
            } else {
                draw_box(&mut svg, i, head_y, head_y + PADDING_TOP + ASCENT_14);
            }
        }
        // Tails: rect at lifeline bottom (footbox_y), same 33.0679 band.
        if !hide_footbox {
            for (i, &actor) in is_actor.iter().enumerate() {
                if actor {
                    draw_actor(&mut svg, i, footbox_y);
                } else {
                    draw_box(&mut svg, i, footbox_y, footbox_y + PADDING_TOP + ASCENT_14);
                }
            }
        }
    }
    if !teoz {
    for (i, p) in participants.iter().enumerate() {
        let x = pos_b_vals[i] + x_offset;
        let w = head_widths[i];
        let cx = pos_c_vals[i] + x_offset;
        // Bottom align each head: offset its own origin so its bottom lands at
        // head_y + common_own (mirrors LivingSpaces.drawHeads BOTTOM).
        let hy = head_y + (common_own - own_heights[i]);
        // Wrapper <g class="participant participant-head" ...>
        let uid = p.uid();
        let source_line = participant_source_lines.get(i).copied().flatten();
        let line_str = source_line.map(|n| n.to_string());
        let mut head_attrs: Vec<(&str, String)> = vec![
            ("class", "participant participant-head".to_string()),
            ("id", format!("{uid}-head")),
            ("data-qualified-name", p.code().to_string()),
            ("data-entity-uid", uid.to_string()),
        ];
        if let Some(ref ls) = line_str {
            head_attrs.push(("data-source-line", ls.clone()));
        }
        let head_attrs_final: Vec<(&str, &str)> = head_attrs.iter()
            .map(|(k, v)| (*k, v.as_str())).collect();
        svg.open_group_with_attrs(&head_attrs_final);

        if is_actor[i] {
            // Actor: draw text first, then stickman (matching Java's ComponentRoseActor order)
            // Text below stickman
            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
            let text_y = hy + STICKMAN_HEIGHT + ACTOR_TEXT_BASELINE;

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
            let head_cy = hy + STICKMAN_THICKNESS + STICKMAN_HEAD_DIAM / 2.0;
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
            svg.svg_rectangle(x, hy, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
            svg.set_filter(None);

            let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
            let text_x = x + (w - text_w) / 2.0;
            let text_y = hy + PADDING_TOP + ASCENT_14;

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
        svg.close_group();

        // ── Draw footbox (tail) for this participant ──────────────────────
        if !hide_footbox {
            // Wrapper <g class="participant participant-tail" ...>
            let uid = p.uid();
            let source_line = participant_source_lines.get(i).copied().flatten();
            let line_str = source_line.map(|n| n.to_string());
            let mut tail_attrs: Vec<(&str, String)> = vec![
                ("class", "participant participant-tail".to_string()),
                ("id", format!("{uid}-tail")),
                ("data-qualified-name", p.code().to_string()),
                ("data-entity-uid", uid.to_string()),
            ];
            if let Some(ref ls) = line_str {
                tail_attrs.push(("data-source-line", ls.clone()));
            }
            let tail_attrs_final: Vec<(&str, &str)> = tail_attrs.iter()
                .map(|(k, v)| (*k, v.as_str())).collect();
            svg.open_group_with_attrs(&tail_attrs_final);
            if is_actor[i] {
                // Actor footbox: text above + stickman below (reversed from head)
                let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
                let text_x = cx - text_w / 2.0 - ACTOR_PADDING_H;
                let text_y = tail_y + ACTOR_TEXT_BASELINE;

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
                // Stickman drawn below the text block (ComponentRoseActor tail).
                let stickman_top = tail_y + ACTOR_TEXT_HEIGHT;
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
                svg.svg_rectangle(x, tail_y, w, TEXT_BLOCK_HEIGHT, head_round, head_round, 0.0);
                svg.set_filter(None);

                let text_w = bounder.calculate_dimension(&font_p, p.display()).width();
                let text_x = x + (w - text_w) / 2.0;
                let text_y = tail_y + PADDING_TOP + ASCENT_14;

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
            svg.close_group();
        }
    }
    }
    // ── Force SVG dimensions ────────────────────────────────────────────
    svg.set_hidden(true);
    // Background rectangle sets SVG dimensions via ensure_visible.
    // ensure_visible(total_width, ...) → max_x = total_width + 1 = SVG width.
    svg.svg_rectangle(0.0, 0.0, total_width, total_height, 0.0, 0.0, 0.0);
    svg.set_hidden(false);

    // ── Draw explicit activation boxes (second pass, over heads/lifelines) ─
    if !teoz {
        draw_activation_bars(&mut svg, &activations, participants, &pos_c_vals, x_offset);
    }

    // ── Draw group headers + messages + notes (interleaved) ──────────────
    let mut msg_idx = 0;
    for event in diagram.events() {
        if let SequenceEvent::Message(msg) = event {
            // ── teoz native group chrome (GroupingTile.drawU order) ──────
            if teoz {
                let mut anchored: Vec<usize> = (0..teoz_group_anchor.len())
                    .filter(|&g| teoz_group_anchor[g] == Some(msg_idx))
                    .collect();
                anchored.sort_unstable(); // pre-order gid = outer-before-inner
                for gid in anchored {
                    let (fmin, fmax) = teoz_frame_x[gid];
                    let fx = fmin + x_offset;
                    let fw = fmax - fmin;
                    let fy = lifeline_y + teoz_layout.group_hook[gid];
                    let fh = teoz_layout.group_pref[gid] - 10.0;

                    let kw = &teoz_layout.group_kind[gid];
                    let tab_label = if kw == "group" {
                        let c = &teoz_layout.group_comment[gid];
                        if c.is_empty() { "group".to_string() } else { c.clone() }
                    } else {
                        kw.clone()
                    };
                    let label_w = max_line_width(&bounder, &font_bold, &tab_label);
                    let tab_w = label_w + GROUP_TEXT_PADDING + 2.0 * GROUP_TEXT_PADDING;
                    let tab_right = fx + tab_w;

                    // Folded-corner tab.
                    svg.set_fill_color(COLOR_GROUP_HEADER);
                    svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                    svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                    let path = format!(
                        "M{x},{y} L{x2},{y} L{x2},{y2} L{x3},{y3} L{x},{y3} L{x},{y}",
                        x = format_number_path(fx),
                        y = format_number_path(fy),
                        x2 = format_number_path(tab_right),
                        y2 = format_number_path(fy + GROUP_TAB_HEIGHT - GROUP_TAB_CORNER),
                        x3 = format_number_path(tab_right - GROUP_TAB_CORNER),
                        y3 = format_number_path(fy + GROUP_TAB_HEIGHT),
                    );
                    svg.svg_path(&path, 0.0);

                    // Frame border.
                    svg.set_fill_color("none");
                    svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                    svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                    svg.svg_rectangle(fx, fy, fw, fh, 0.0, 0.0, 0.0);

                    // Tab keyword (13px bold).
                    svg.set_fill_color(COLOR_TEXT);
                    svg.set_stroke_color(None);
                    svg.set_stroke_width(0.0, None);
                    svg.text(
                        &tab_label,
                        fx + GROUP_TEXT_PADDING,
                        fy + GROUP_TAB_TITLE_BASELINE,
                        None,
                        FONT_SIZE_MESSAGE,
                        Some("700"),
                        None,
                        None,
                        label_w,
                        &indexmap::IndexMap::new(),
                        None,
                    );

                    // Bracket comment (11px bold).
                    if kw != "group" && !teoz_layout.group_comment[gid].is_empty() {
                        let cond = format!("[{}]", teoz_layout.group_comment[gid]);
                        let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
                        let cond_w = max_line_width(&bounder, &font_small, &cond);
                        svg.set_fill_color(COLOR_TEXT);
                        svg.set_stroke_color(None);
                        svg.set_stroke_width(0.0, None);
                        svg.text(
                            &cond,
                            fx + tab_w + GROUP_TEXT_PADDING,
                            fy + 13.759,
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

                    // Else separators (full-width dashed line + bracket label).
                    let elses = teoz_layout.group_else[gid].clone();
                    for (eh, elabel) in &elses {
                        let line_y = lifeline_y + eh + 11.0;
                        svg.set_fill_color("none");
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(1.0, Some([2.0, 2.0]));
                        svg.svg_line(fx, line_y, fx + fw, line_y, 0.0);
                        if !elabel.is_empty() {
                            let else_txt = format!("[{elabel}]");
                            let font_small = UFont::sans_serif(11).with_style(FontStyle::bold());
                            let ew = max_line_width(&bounder, &font_small, &else_txt);
                            svg.set_fill_color(COLOR_TEXT);
                            svg.set_stroke_color(None);
                            svg.set_stroke_width(0.0, None);
                            svg.text(
                                &else_txt,
                                fx + 5.0,
                                line_y + 13.7589,
                                None,
                                11,
                                Some("700"),
                                None,
                                None,
                                ew,
                                &indexmap::IndexMap::new(),
                                None,
                            );
                        }
                    }
                }
            }
            // Draw ALL group headers that start at this message index,
            // from outermost (lowest nesting) to innermost (highest nesting).
            let mut groups_at_msg: Vec<usize> = (0..groups.len())
                .filter(|&gi| groups[gi].msg_start == msg_idx && groups[gi].group_type != "else")
                .collect();
            groups_at_msg.sort_by_key(|&gi| groups[gi].nesting_level);
            for gi in groups_at_msg {
                if teoz { continue; }
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
                        let label_w = max_line_width(&bounder, &font_bold, &tab_label);
                        // Tab width = bold title width + 45 (15 left pad + 30 right).
                        let tab_w = label_w + GROUP_TEXT_PADDING + 2.0 * GROUP_TEXT_PADDING;

                        // Header tab path (folded corner). Fold drops
                        // height - cornersize = 9.706, diagonal closes to
                        // (tab_right - cornersize, tab bottom).
                        svg.set_fill_color(COLOR_GROUP_HEADER);
                        svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                        svg.set_stroke_width(GROUP_STROKE_WIDTH, None);
                        let tab_right = fx_off + tab_w;
                        let path = format!(
                            "M{x},{y} L{x2},{y} L{x2},{y2} L{x3},{y3} L{x},{y3} L{x},{y}",
                            x = format_number_path(fx_off),
                            y = format_number_path(fy),
                            x2 = format_number_path(tab_right),
                            y2 = format_number_path(fy + GROUP_TAB_HEIGHT - GROUP_TAB_CORNER),
                            x3 = format_number_path(tab_right - GROUP_TAB_CORNER),
                            y3 = format_number_path(fy + GROUP_TAB_HEIGHT),
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
                            fy + GROUP_TAB_TITLE_BASELINE,
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
                                fy + 13.759,
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
                if teoz { break; }
                if groups[gi].group_type != "else" || groups[gi].msg_start != msg_idx {
                    continue;
                }
                let (fx, _fy, fw, _fh) = group_frames[gi];
                let fx_off = fx + x_offset;
                let group = &groups[gi];
                let divider_y = arrow_ys[msg_idx.saturating_sub(1)] + 9.0;
                // Dashed divider line across the parent frame
                svg.set_fill_color("none");
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(1.0, Some([2.0, 2.0]));
                svg.svg_line(fx_off, divider_y, fx_off + fw, divider_y, 0.0);
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
                        divider_y + 11.759,
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
            // Divider (`== label ==`) preceding this message: band + label.
            if let Some(div) = dividers.iter().find(|d| d.before_msg == msg_idx) {
                // Band spans the global head range ±5.
                let mut hd_lo = f64::MAX;
                let mut hd_hi = f64::MIN;
                for pi in 0..participants.len() {
                    hd_lo = hd_lo.min(pos_b_vals[pi] + x_offset);
                    hd_hi = hd_hi.max(pos_b_vals[pi] + head_widths[pi] + x_offset);
                }
                let band_x = hd_lo - 5.0;
                let band_w = hd_hi - hd_lo + 10.0;
                let tile_top = arrow_ys[msg_idx - 1] + 8.0;
                let band_y = tile_top + DIVIDER_BAND_Y;

                // Band rect (fill + stroke same divider background).
                svg.set_fill_color(COLOR_DIVIDER_BG);
                svg.set_stroke_color(Some(COLOR_DIVIDER_BG));
                svg.set_stroke_width(1.0, None);
                svg.svg_rectangle(band_x, band_y, band_w, DIVIDER_BAND_H, 0.0, 0.0, 0.0);
                // Two black border lines at top and bottom of the band.
                svg.set_fill_color(COLOR_TEXT);
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(1.0, None);
                svg.svg_line(band_x, band_y, band_x + band_w, band_y, 0.0);
                svg.svg_line(band_x, band_y + DIVIDER_BAND_H, band_x + band_w, band_y + DIVIDER_BAND_H, 0.0);

                // Bold13 label rect (centered on the band).
                let text_w = max_line_width(&bounder, &font_bold, &div.text);
                let label_w = text_w + DIVIDER_LABEL_PAD_X;
                let label_x = band_x + (band_w - label_w) / 2.0;
                let label_y = tile_top + DIVIDER_LABEL_Y;
                svg.set_fill_color(COLOR_DIVIDER_BG);
                svg.set_stroke_color(Some(COLOR_GROUP_STROKE));
                svg.set_stroke_width(2.0, None);
                svg.svg_rectangle(label_x, label_y, label_w, DIVIDER_LABEL_H, 0.0, 0.0, 0.0);
                // Label text.
                svg.set_fill_color(COLOR_TEXT);
                svg.set_stroke_color(None);
                svg.set_stroke_width(0.0, None);
                svg.text(
                    &div.text,
                    label_x + 6.0,
                    tile_top + DIVIDER_TEXT_BASELINE,
                    None,
                    FONT_SIZE_MESSAGE,
                    Some("700"),
                    None,
                    None,
                    text_w,
                    &indexmap::IndexMap::new(),
                    None,
                );
            }

            let p1_idx = pcode_to_idx.get(msg.p1().code()).copied().unwrap_or(0);
            let p2_idx = pcode_to_idx.get(msg.p2().code()).copied().unwrap_or(1);
            let y = arrow_ys[msg_idx];

            // Draw arrow first, then note(s) for this message
            // Skip drawing for hidden messages (they occupy Y space but aren't drawn)
            let is_hidden = msg_hidden.get(msg_idx).copied().unwrap_or(false);
            if !is_hidden {
                // Wrapper <g class="message" ...> (legacy only; teoz emits the
                // polygon/line/text as bare siblings).
                if !teoz {
                    let msg_num_str = format!("msg{}", msg_idx + 1);
                    let p1_uid = participants[p1_idx].uid().to_string();
                    let p2_uid = participants[p2_idx].uid().to_string();
                    let source_line = msg_source_lines.get(msg_idx).copied().flatten();
                    let line_str = source_line.map(|n| n.to_string());
                    let mut msg_attrs: Vec<(&str, String)> = vec![
                        ("class", "message".to_string()),
                        ("id", msg_num_str),
                        ("data-entity-1", p1_uid),
                        ("data-entity-2", p2_uid),
                    ];
                    if let Some(ls) = &line_str {
                        msg_attrs.push(("data-source-line", ls.clone()));
                    }
                    let msg_attrs_final: Vec<(&str, &str)> = msg_attrs.iter()
                        .map(|(k, v)| (*k, v.as_str())).collect();
                    svg.open_group_with_attrs(&msg_attrs_final);
                }

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
                        teoz,
                        if autonumber { Some(auto_num_strings[msg_idx].as_str()) } else { None },
                        auto_num_widths.get(msg_idx).copied().unwrap_or(0.0),
                    );
                }
                if !teoz { svg.close_group(); }
            }
            // Attached notes ("note right:" / "note left:") share this message
            // tile and are positioned relative to the arrow.
            for note in notes {
                if note.msg_index != msg_idx || note.participant.is_some() {
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
                    if teoz && teoz_is_self.get(msg_idx).copied().unwrap_or(false) {
                        teoz_layout.msg_hook.get(msg_idx)
                            .filter(|h| h.is_finite())
                            .map(|h| lifeline_y + h)
                    } else {
                        None
                    },
                    teoz,
                );
            }
            // Standalone notes ("note right of P:") are independent tiles drawn
            // below this message's arrow; they stack cumulatively within the gap.
            let mut stack_y = 0.0_f64;
            for note in notes {
                if note.msg_index != msg_idx || note.participant.is_none() {
                    continue;
                }
                draw_standalone_note(
                    &mut svg, note, &pos_c_vals, &pcode_to_idx, &head_widths, x_offset,
                    y, stack_y, &bounder, &font_m, &skin,
                );
                let line_count = note.text.split("\\n").count().max(1) as f64;
                stack_y += STANDALONE_NOTE_H + (line_count - 1.0) * STANDALONE_NOTE_LINE_H;
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

/// Draws a standalone note tile (`note right of P: t`).
///
/// Geometry (measured against the reference jar, ported from
/// `sequencediagram/teoz/NoteTile.java` and
/// `skin/rose/ComponentRoseNote.java`):
/// - polygon x = round(posC + paddingX(5)),
/// - polygon is 27px tall for one line (textBlock 17 + 2*old padding 5),
/// - polygon width = trunc(textWidth + old padding 6 + old padding 15),
/// - text starts 6px right, baseline 18.897 below the polygon top.
fn draw_standalone_note(
    svg: &mut SvgGraphics,
    note: &NoteInfo,
    pos_c_vals: &[f64],
    pcode_to_idx: &std::collections::HashMap<String, usize>,
    head_widths: &[f64],
    x_offset: f64,
    arrow_y: f64,
    stack_y: f64,
    bounder: &dyn StringBounder,
    font_m: &UFont,
    skin: &SkinConfig,
) {
    let target = note.participant.as_deref().unwrap_or("");
    let (code, over_idx): (&str, Option<usize>) = match target.split_once(',') {
        Some((a, b)) => (
            a,
            pcode_to_idx.get(b.trim()).copied(),
        ),
        None => (target, None),
    };
    let p_idx = pcode_to_idx.get(code).copied().unwrap_or(0);

    let lines: Vec<&str> = note.text.split("\\n").collect();
    let mut max_line_w = 0.0_f64;
    for line in &lines {
        let w = bounder.calculate_dimension(font_m, line).width();
        if w > max_line_w {
            max_line_w = w;
        }
    }

    // Polygon top = this gap origin, which is arrow_y + 13 + cumulative stack.
    let path_top = arrow_y + 13.0 + stack_y;

    // Horizontal placement.
    //
    // "note over A, B" (OVER_SEVERAL, Puma2 `NoteBox`): the note spans from the
    // first participant's box left edge to the second participant's box right
    // edge, each `ParticipantBox` carrying an `outMargin` of 5 on both sides.
    //
    // - text_width  = pure_text + oldPadding 6 + oldPadding 15
    // - compPrefW   = text_width + 2 * paddingX(5)
    // - span        = (c2 + headW2/2) - (c1 - headW1/2)
    // - prefW       = max(compPrefW, span)
    // - x_start     = trunc((c1 + c2)/2 - prefW/2)
    // - polygon_x   = x_start + paddingX, polygon_w = trunc(prefW - 2*paddingX)
    // - text_x      = x_start + paddingX + 6 + (prefW - compPrefW)/2
    //
    // Ported from: `sequencediagram/graphic/NoteBox.java`,
    // `skin/rose/ComponentRoseNote.java`.
    let (polygon_x, polygon_w, text_x);
    if let Some(p2_idx) = over_idx {
        let c1 = pos_c_vals[p_idx] + x_offset;
        let c2 = pos_c_vals[p2_idx] + x_offset;
        let hw1 = head_widths[p_idx];
        let hw2 = head_widths[p2_idx];
        let text_width = max_line_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2;
        let comp_pref_w = text_width + 2.0 * NOTE_PADDING_X;

        // `ParticipantBox.getMaxX` includes one extra outMargin beyond
        // center+hw/2 (it uses 2*outMargin while center uses one).
        let span = (c2 + hw2 / 2.0 + NOTE_OUT_MARGIN) - (c1 - hw1 / 2.0);
        let pref_w = comp_pref_w.max(span);
        // Java `(int)` truncates toward zero.
        let x_start = (c1 / 2.0 + c2 / 2.0 - pref_w / 2.0).trunc();
        polygon_x = x_start + NOTE_PADDING_X;
        polygon_w = (pref_w - 2.0 * NOTE_PADDING_X).trunc();
        text_x = polygon_x + NOTE_OLD_PADDING_X1 + (pref_w - comp_pref_w) / 2.0;
    } else {
        polygon_w = (max_line_w + NOTE_OLD_PADDING_X1 + NOTE_OLD_PADDING_X2).trunc();
        polygon_x = (pos_c_vals[p_idx] + x_offset + NOTE_PADDING_X).round();
        text_x = polygon_x + NOTE_OLD_PADDING_X1;
    }

    // One line: 27 tall; each extra line adds 13.
    let polygon_h = 27.0 + (lines.len().saturating_sub(1) as f64) * MSG_LINE_HEIGHT;

    svg.set_fill_color(skin.color_note_back);
    svg.set_stroke_color(Some(skin.color_stroke));
    svg.set_stroke_width(0.5, None);
    svg.set_filter(skin.shadow_filter_id.as_deref());
    let path_d = format_note_path_abs(polygon_x, path_top, polygon_w, polygon_h, NOTE_CORNERSIZE);
    svg.svg_path(&path_d, 0.0);
    svg.set_filter(None);
    let corner_d = format_note_corner_abs(polygon_x, path_top, polygon_w, NOTE_CORNERSIZE);
    svg.svg_path(&corner_d, 0.0);

    svg.set_fill_color(COLOR_TEXT);
    svg.set_stroke_color(None);
    svg.set_stroke_width(0.0, None);
    for (line_idx, line) in lines.iter().enumerate() {
        let text_w = bounder.calculate_dimension(font_m, line).width();
        let text_y = path_top + 5.0 + ASCENT_13 + (line_idx as f64) * MSG_LINE_HEIGHT;
        svg.text(
            line, text_x, text_y, None, FONT_SIZE_MESSAGE, None, None, None,
            text_w, &indexmap::IndexMap::new(), None,
        );
    }
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
    bounder: &dyn StringBounder,
    font_m: &UFont,
    pre_wrapped_width: f64,
    msg_p1_level: i32,
    _msg_p2_level: i32,
    note_p1_level: i32,
    note_p2_level: i32,
    skin: &SkinConfig,
    // teoz: the owning tile's screen origin (lifeline origin + tile hook).
    // A self-message note shares the tile and is top-aligned there, rather
    // than being positioned relative to the arrow contact line.
    teoz_tile_origin: Option<f64>,
    teoz: bool,
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
    let msg_line_count = msg_text_heights.get(msg_idx).map_or(1, |&h| ((h - MSG_PADDING_TOP - MSG_PADDING_BOTTOM) / MSG_LINE_HEIGHT) as usize);

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

    // Legacy Puma ArrowAndNoteBox geometry for a note attached to a NON-SELF
    // message: the arrow and note share one tile sized boxH = max(arrowH,
    // noteH), each vertically centered. Polygon top is component-top (+5
    // oldPaddingY after the NoteBox translate); polygon height truncates the
    // note text height; the note X is an integer (segment pos cast to int).
    // Self-message notes use MessageSelfArrow/ArrowAndNoteBox with different
    // geometry and keep the legacy anchor below.
    let (poly_y_top, poly_height) = if teoz_tile_origin.is_none() && !is_self_msg {
        let k = lines.len() as f64;
        let note_text_h = 10.0 + k * MSG_LINE_HEIGHT;
        let note_h = note_text_h + 2.0 * NOTE_MARGIN_Y;
        let arrow_h = msg_text_heights.get(msg_idx).copied().unwrap_or(MSG_LINE_HEIGHT + 2.0)
            + ARROW_DELTA_Y + 2.0 * MSG_PADDING_Y;
        let box_h = arrow_h.max(note_h);
        let push_a = (box_h - arrow_h) / 2.0;
        let push_n = (box_h - note_h) / 2.0;
        let text_h = msg_text_heights.get(msg_idx).copied().unwrap_or(MSG_LINE_HEIGHT + 2.0);
        let top = y - push_a - text_h + push_n + 1.0;
        (top, note_text_h.trunc())
    } else {
        let top = match teoz_tile_origin {
            Some(origin) => origin + NOTE_MARGIN_Y,
            None => y - ARROW_Y_BASE - (msg_line_count.saturating_sub(1) as f64) * MSG_LINE_HEIGHT,
        };
        let h = if teoz_tile_origin.is_some() {
            ((lines.len() as f64) * MSG_LINE_HEIGHT).trunc() + 2.0 * NOTE_MARGIN_Y
        } else {
            (lines.len() as f64) * MSG_LINE_HEIGHT + 2.0 * NOTE_MARGIN_Y
        };
        (top, h)
    };
    let note_y_top = poly_y_top;
    let note_h = poly_height;

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
    // Legacy Puma casts the note segment X to int (NoteBox.getStartingX).
    let note_x = if teoz_tile_origin.is_none() && !is_self_msg {
        note_x.trunc()
    } else {
        note_x
    };
    // Draw note shape (folded corner rectangle) with absolute coordinates
    svg.set_fill_color(skin.color_note_back);
    svg.set_stroke_color(Some(skin.color_stroke));
    svg.set_stroke_width(if skin.skin_rose { 1.0 } else { STROKE_WIDTH_BOX }, None);
    let path_d = format_note_path_abs(note_x, note_y_top, polygon_w, note_h, NOTE_CORNERSIZE);
    svg.set_filter(skin.shadow_filter_id.as_deref());
    svg.svg_path(&path_d, 0.0);
    // Java's svgPath calls ensureVisible for each path coordinate + 2*deltaShadow.
    // Legacy rose reserves 2*5; teoz ComponentRoseNote uses deltaShadow 4.
    let delta_shadow_note_ensure = if skin.skin_rose {
        if teoz { 4.0 } else { 10.0 }
    } else {
        0.0
    };
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
        let text_y = if teoz_tile_origin.is_some() {
            note_y_top + NOTE_MARGIN_Y + ASCENT_13 + (line_idx as f64) * MSG_LINE_HEIGHT
        } else if is_self_msg {
            note_y_top + 15.111 + (line_idx as f64) * 13.0
        } else {
            // ComponentRoseNote: text translated to oldPaddingY(5), first
            // baseline = +5 + ascent(13.897), line pitch 17.706.
            note_y_top + NOTE_MARGIN_Y + ASCENT_13 + (line_idx as f64) * MSG_LINE_HEIGHT
        };
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
    bounder: &dyn StringBounder,
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
    // teoz: reverse arrows end at the level-adjusted source edge (`x1 - 1`);
    // the legacy engine keeps the older raw edge for level-1 sources.
    teoz: bool,
    auto_number: Option<&str>,
    auto_num_w: f64,
) {
    let x1_raw = pos_c[p1_idx] + x_offset;
    let x2_raw = pos_c[p2_idx] + x_offset;
    let is_self = p1_idx == p2_idx;
    let is_return = p1_idx > p2_idx;
    let is_reverse = msg.arrow_config().is_reverse_define();
    let is_dashed = msg.arrow_config().is_dotted();
    let ld = ACTIVATION_BAR_EXPLICIT_OFFSET; // LIVE_DELTA_SIZE = 5
    let max_level = level_ignore.max(level_considere) as f64;
    // Per-message arrow color: a `-[#color]` bracket sets it on the
    // ArrowConfiguration (CommandArrow.applyStyle); otherwise use the skin's
    // default arrow color.
    let arrow_color_string;
    let arrow_color: &str = match msg.arrow_config().color() {
        Some(c) => {
            arrow_color_string = c.to_svg(&plantuml_klimt::color::ColorMapper::identity());
            &arrow_color_string
        }
        None => skin.color_arrow,
    };

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
            svg.set_stroke_color(Some(arrow_color));
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

        svg.set_stroke_color(Some(arrow_color));
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
            svg.set_stroke_color(Some(arrow_color));
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
            svg.set_fill_color(arrow_color);
            svg.set_stroke_color(Some(arrow_color));
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

        svg.set_fill_color(arrow_color);
        svg.set_stroke_color(Some(arrow_color));
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
        // Tail at the source: when it is active at level 1, the arrow starts from
        // its own lifeline (posC - 1), not from the offset x1 (which moved -5 to
        // anchor the area). CommunicationTile reverse branch.
        let line_end_x = if teoz { x1 - 1.0 } else if msg_p1_level == 1 { x1_raw - 1.0 } else { x1 - 1.0 };
        svg.set_stroke_color(Some(arrow_color));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
        svg.svg_line(line_start, y, line_end_x, y, 0.0);
    } else {
        // Normal arrow: left-to-right, solid, right-pointing arrowhead
        let tip_x = x2 - ARROWHEAD_TIP_OFFSET;
        let base_x = tip_x - ARROWHEAD_SIZE;
        let half = ARROWHEAD_SIZE / 2.0 - 1.0;
        let is_async = msg.arrow_config().is_async2();

        svg.set_fill_color(arrow_color);
        svg.set_stroke_color(Some(arrow_color));
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
        svg.set_stroke_color(Some(arrow_color));
        svg.set_stroke_width(STROKE_WIDTH_ARROW, if is_dashed { Some([2.0, 2.0]) } else { None });
        svg.svg_line(x1, y, line_end, y, 0.0);
    }

    // Autonumber: bold number at the label slot, drawn even when the label
    // is empty (but the prefix only joins the width reservation with a label).
    if let Some(num) = auto_number {
        let auto_slot_x = if is_self {
            if is_reverse {
                x1 - 50.0 + MESSAGE_TEXT_X_OFFSET
            } else {
                x1 + ld * max_level + MESSAGE_TEXT_X_OFFSET
            }
        } else if is_return {
            x2 + MESSAGE_TEXT_X_OFFSET + ARROWHEAD_SIZE
        } else {
            x1 + MESSAGE_TEXT_X_OFFSET
        };
        let auto_num_y = y - MSG_LINE_HEIGHT - MSG_PADDING_TOP - MSG_PADDING_BOTTOM + ASCENT_13;
        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        svg.set_stroke_width(0.0, None);
        svg.text(
            num, auto_slot_x, auto_num_y, None, FONT_SIZE_MESSAGE, Some("700"),
            None, None, auto_num_w, &indexmap::IndexMap::new(), None,
        );
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
        // Shift the label right of the number only when autonumber is active.
        let text_x = if auto_number.is_some() {
            text_x + auto_num_w + 4.0
        } else {
            text_x
        };
        let text_y = y - (lines.len().max(1) as f64) * MSG_LINE_HEIGHT - MSG_PADDING_TOP - MSG_PADDING_BOTTOM + ASCENT_13;

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
        // PlantUML renders wrapped labels as a flat run of per-word text
        // blocks: each inter-word space is its own element with the regular
        // space advance (3.38px at 13pt), so the cursor advances by it even
        // though the space glyph has no ink.
        let space_w = bounder.calculate_dimension(font, " ").width();
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = text_y + (line_idx as f64) * MSG_LINE_HEIGHT;
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
                            // The separating space is its own nbsp element carrying
                            // the regular space advance; the cursor must move past it.
                            if use_exact {
                                svg.text_exact(
                                    "\u{00A0}", text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                    space_w, &indexmap::IndexMap::new(), None,
                                );
                            } else {
                                svg.text(
                                    "\u{00A0}", text_x + offset_x, line_y, None, FONT_SIZE_MESSAGE, None, None, None,
                                    space_w, &indexmap::IndexMap::new(), None,
                                );
                            }
                            offset_x += space_w;
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
    bounder: &dyn StringBounder,
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
        let text_y = y - (lines.len().max(1) as f64) * MSG_LINE_HEIGHT - MSG_PADDING_TOP - MSG_PADDING_BOTTOM + ASCENT_13;

        svg.set_fill_color(COLOR_TEXT);
        svg.set_stroke_color(None);
        for (line_idx, line) in lines.iter().enumerate() {
            let line_y = text_y + (line_idx as f64) * MSG_LINE_HEIGHT;
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
    /// Participant the note is attached to (`note right of Alice`).
    pub participant: Option<String>,
    /// Index of the message this note is attached to.
    pub msg_index: usize,
}

/// Parsed divider (`== label ==`).
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/Divider.java`.
#[derive(Clone)]
pub struct DividerInfo {
    /// Divider label text.
    pub text: String,
    /// Index of the message immediately following the divider.
    pub before_msg: usize,
}

/// Source-order structural token used to reconstruct the teoz tile tree.
///
/// teoz lays groups out as ONE frame containing sections (alt/else) and
/// messages, which the flat [`GroupInfo`] list cannot express (it emits one
/// frame per `else` section). Recorded during the text parse; only consumed by
/// the teoz render branch.
#[derive(Clone, Debug)]
pub enum TeozTok {
    /// A message tile: message ordinal (0-based) and whether it starts an
    /// `&`-parallel run.
    Msg { idx: usize, parallel: bool },
    /// `group`/`alt`/`opt`/`loop`/`par`/... opening. Keyword, comment/condition,
    /// whether the group itself is `&`-parallel, and optional `#color`.
    GroupOpen {
        kind: String,
        comment: String,
        parallel: bool,
        backcolor: Option<String>,
    },
    /// `else [condition]` separator inside the current group.
    Else { comment: String },
    /// `end` closing the innermost group.
    GroupEnd,
    /// `== divider ==` before the message with this ordinal.
    Divider { text: String, before_msg: usize },
    /// A standalone/inline life event (activate/deactivate/destroy) tile,
    /// zero-height in the Y gauge. `ev` indexes life events in source order
    /// (the `SequenceEvent::LifeEvent` subsequence of `diagram.events()`).
    Life { ev: usize },
}

/// A section of a teoz group: the optional else label and its child nodes.
#[derive(Clone, Default, Debug)]
struct TeozSection {
    /// Condition after `else`; empty for the first (head) section.
    else_comment: String,
    /// Child nodes in source order.
    children: Vec<TeozNode>,
}

/// A node in the reconstructed teoz tile tree.
#[derive(Clone, Debug)]
enum TeozNode {
    /// Message tile.
    Msg { idx: usize, parallel: bool },
    /// Group frame with one or more sections (head + elses).
    Group {
        /// Pre-order id assigned by [`build_teoz_tree`]; indexes layout data.
        gid: usize,
        kind: String,
        comment: String,
        parallel: bool,
        backcolor: Option<String>,
        sections: Vec<TeozSection>,
    },
    /// `== divider ==` tile.
    #[allow(dead_code)]
    Divider { text: String, before_msg: usize },
    /// Zero-height life-event tile; `ev` indexes the LifeEvent subsequence.
    Life { ev: usize },
}

/// Reconstructs the teoz tile tree from the source-order token stream.
/// A `GroupOpen` collects children (including nested groups and `else`
/// sections) up to its matching `GroupEnd`.
fn build_teoz_tree(toks: &[TeozTok]) -> Vec<TeozNode> {
    fn section(toks: &[TeozTok], pos: &mut usize, else_comment: String, next_gid: &mut usize) -> TeozSection {
        let mut children = Vec::new();
        while *pos < toks.len() {
            match &toks[*pos] {
                TeozTok::GroupEnd | TeozTok::Else { .. } => break,
                TeozTok::GroupOpen { .. } => {
                    if let Some(node) = group(toks, pos, next_gid) {
                        children.push(node);
                    }
                }
                TeozTok::Msg { idx, parallel } => {
                    children.push(TeozNode::Msg { idx: *idx, parallel: *parallel });
                    *pos += 1;
                }
                TeozTok::Divider { text, before_msg } => {
                    children.push(TeozNode::Divider { text: text.clone(), before_msg: *before_msg });
                    *pos += 1;
                }
                TeozTok::Life { ev } => {
                    children.push(TeozNode::Life { ev: *ev });
                    *pos += 1;
                }
            }
        }
        TeozSection { else_comment, children }
    }

    fn group(toks: &[TeozTok], pos: &mut usize, next_gid: &mut usize) -> Option<TeozNode> {
        let (kind, comment, parallel, backcolor) = match &toks[*pos] {
            TeozTok::GroupOpen { kind, comment, parallel, backcolor } => {
                (kind.clone(), comment.clone(), *parallel, backcolor.clone())
            }
            _ => return None,
        };
        let gid = *next_gid;
        *next_gid += 1;
        *pos += 1; // consume GroupOpen
        let mut sections = vec![section(toks, pos, String::new(), next_gid)];
        while *pos < toks.len() {
            if let TeozTok::Else { comment: ec } = &toks[*pos] {
                let ec = ec.clone();
                *pos += 1;
                sections.push(section(toks, pos, ec, next_gid));
            } else {
                break;
            }
        }
        if *pos < toks.len() && matches!(toks[*pos], TeozTok::GroupEnd) {
            *pos += 1; // consume GroupEnd
        }
        Some(TeozNode::Group { gid, kind, comment, parallel, backcolor, sections })
    }

    let mut root: Vec<TeozNode> = Vec::new();
    let mut pos = 0usize;
    let mut next_gid = 0usize;
    while pos < toks.len() {
        match &toks[pos] {
            TeozTok::GroupOpen { .. } => {
                if let Some(node) = group(toks, &mut pos, &mut next_gid) {
                    root.push(node);
                } else {
                    pos += 1;
                }
            }
            TeozTok::Msg { idx, parallel } => {
                root.push(TeozNode::Msg { idx: *idx, parallel: *parallel });
                pos += 1;
            }
            TeozTok::Divider { text, before_msg } => {
                root.push(TeozNode::Divider { text: text.clone(), before_msg: *before_msg });
                pos += 1;
            }
            TeozTok::Life { ev } => {
                root.push(TeozNode::Life { ev: *ev });
                pos += 1;
            }
            TeozTok::GroupEnd | TeozTok::Else { .. } => pos += 1,
        }
    }
    root
}

// ── teoz vertical layout (TimeHook cumulative model, JAR 1.2026.6) ──────────
// Component preferred heights / contact points for single-line 13px text.
const TEOZ_MSG_H: f64 = 31.706;
const TEOZ_MSG_CPR: f64 = 23.706;
/// Empty-label non-self arrow: textHeight collapses to 2, so prefHeight is
/// 2 + arrowDeltaY(4) + 2*paddingY(8) = 14 and contact point is 2 + 4 = 6.
const TEOZ_MSG_EMPTY_H: f64 = 14.0;
const TEOZ_MSG_EMPTY_CPR: f64 = 6.0;
const TEOZ_SELF_ZZZ: f64 = 18.5;
/// Text-block height for a one-line 13px self-message label.
const TEOZ_SELF_TEXT_ONE: f64 = 19.706;
/// Extra text-block height per additional wrapped line (line pitch 17.706).
const TEOZ_SELF_LINE_INC: f64 = 17.706;
const TEOZ_ELSE_H: f64 = 32.9819;
const TEOZ_EMPTY_H: f64 = 4.0;
const TEOZ_HEADER_H: f64 = 19.706;
/// Children start this far below a group's chaining hook (header + 10).
const TEOZ_HEADER_OFFSET: f64 = TEOZ_HEADER_H + 10.0;
/// A group reserves 20px (MARGINY_MAGIC) below the body in addition to header.
const TEOZ_GROUP_TAIL: f64 = 20.0;

/// Output of the teoz vertical (TimeHook) layout. All hook values are in the
/// gauge space whose origin maps to screen Y `head_height + 10`.
#[derive(Clone, Default, Debug)]
struct TeozLayoutOut {
    /// Tile hook per group (pre-order gid).
    group_hook: Vec<f64>,
    /// Preferred (reserved) height per group.
    group_pref: Vec<f64>,
    /// Body height per group (pref - header - tail).
    group_body: Vec<f64>,
    /// Else separator hooks (gauge space) and labels, per group.
    group_else: Vec<Vec<(f64, String)>>,
    /// Group keyword ("opt", "alt", "loop", …) per gid.
    group_kind: Vec<String>,
    /// Raw group comment (without brackets) per gid.
    group_comment: Vec<String>,
    /// Optional backcolor per gid.
    group_backcolor: Vec<Option<String>>,
    /// Tile hook per message ordinal (its contact-line anchor).
    msg_hook: Vec<f64>,
    /// Tile hook per life event (LifeEvent subsequence ordinal).
    life_hook: Vec<f64>,
    /// Final chained Y at the root level.
    final_y: f64,
}

/// A raw tile at one nesting level before `&`-parallel merging.
#[derive(Clone)]
enum TeozTile {
    /// 4px breathing spacer wrapped around a group.
    Empty,
    /// Else separator carrying its condition label.
    Else(String),
    /// Message by ordinal.
    Msg { idx: usize, parallel: bool },
    /// Group frame by pre-order gid.
    Group { gid: usize, parallel: bool },
    /// `== divider ==` (unused by the current fixtures).
    #[allow(dead_code)]
    Divider { text: String, before_msg: usize },
    /// Zero-height life-event tile; `ev` indexes the LifeEvent subsequence.
    Life { ev: usize },
}

/// A merged item walked during measure/place.
#[derive(Clone)]
enum TeozItem {
    Empty,
    Else(String),
    One(TeozTile),
    /// Contact-aligned `&`-parallel cluster.
    Many(Vec<TeozTile>),
}

fn teoz_msg_geom(idx: usize, is_self: &[bool], rows: &[usize], empty: &[bool]) -> (f64, f64, f64) {
    // (prefHeight, contactRelative, zzz = prefHeight - contactRelative)
    if is_self.get(idx).copied().unwrap_or(false) {
        // ComponentRoseSelfArrow: prefHeight = textHeight + arrowDeltaY(4)
        // + arrowOnlyHeight(13) + 2*paddingY(4) = textHeight + 25;
        // getYPoint (cpr) = (textHeight + textHeight + 13)/2 + paddingX.
        // paddingX is 0 so cpr = textHeight + 6.5. textHeight grows one line
        // (17.706) per wrapped row beyond the first; zzz stays 18.5.
        let n = rows.get(idx).copied().unwrap_or(1).max(1) as f64;
        let text_h = TEOZ_SELF_TEXT_ONE + (n - 1.0) * TEOZ_SELF_LINE_INC;
        (text_h + 25.0, text_h + 6.5, TEOZ_SELF_ZZZ)
    } else if empty.get(idx).copied().unwrap_or(false) {
        // No-label arrow: ComponentRoseArrow textHeight collapses to 2.
        (TEOZ_MSG_EMPTY_H, TEOZ_MSG_EMPTY_CPR, TEOZ_MSG_EMPTY_H - TEOZ_MSG_EMPTY_CPR)
    } else {
        (TEOZ_MSG_H, TEOZ_MSG_CPR, TEOZ_MSG_H - TEOZ_MSG_CPR)
    }
}

fn teoz_tile_parallel(t: &TeozTile) -> bool {
    match t {
        TeozTile::Msg { parallel, .. } | TeozTile::Group { parallel, .. } => *parallel,
        _ => false,
    }
}

fn teoz_tile_cpr(t: &TeozTile, is_self: &[bool], rows: &[usize], empty: &[bool]) -> f64 {
    match t {
        TeozTile::Msg { idx, .. } => teoz_msg_geom(*idx, is_self, rows, empty).1,
        _ => 0.0,
    }
}

fn teoz_tile_pref(t: &TeozTile, is_self: &[bool], rows: &[usize], gpref: &[f64], empty: &[bool]) -> f64 {
    match t {
        TeozTile::Empty => TEOZ_EMPTY_H,
        TeozTile::Else(_) => TEOZ_ELSE_H,
        TeozTile::Msg { idx, .. } => teoz_msg_geom(*idx, is_self, rows, empty).0,
        TeozTile::Group { gid, .. } => gpref.get(*gid).copied().unwrap_or(0.0),
        TeozTile::Divider { .. } => TEOZ_MSG_H,
        TeozTile::Life { .. } => 0.0,
    }
}

/// Appends raw tiles for a node list, wrapping each group in 4px EmptyTiles.
fn teoz_raw_nodes(nodes: &[TeozNode], out: &mut Vec<TeozTile>) {
    for n in nodes {
        match n {
            TeozNode::Msg { idx, parallel } => {
                out.push(TeozTile::Msg { idx: *idx, parallel: *parallel });
            }
            TeozNode::Group { gid, parallel, .. } => {
                out.push(TeozTile::Empty);
                out.push(TeozTile::Group { gid: *gid, parallel: *parallel });
                out.push(TeozTile::Empty);
            }
            TeozNode::Divider { text, before_msg } => {
                out.push(TeozTile::Divider { text: text.clone(), before_msg: *before_msg });
            }
            TeozNode::Life { ev } => {
                out.push(TeozTile::Life { ev: *ev });
            }
        }
    }
}

/// Builds the flat raw tile stream for a group body: sections joined by Else
/// separators (the first section has none).
fn teoz_raw_sections(sections: &[TeozSection], out: &mut Vec<TeozTile>) {
    for (si, sec) in sections.iter().enumerate() {
        if si > 0 {
            out.push(TeozTile::Else(sec.else_comment.clone()));
        }
        teoz_raw_nodes(&sec.children, out);
    }
}

/// Merges a raw stream into items, collapsing `&`-parallel runs into
/// contact-aligned clusters (mirrors GroupingTile.mergeParallel). A 4px
/// EmptyTile directly preceding a parallel run is dropped.
fn teoz_merge(raw: &[TeozTile]) -> Vec<TeozItem> {
    // Mirrors GroupingTile.mergeParallel. A parallel tile pulls the run of
    // trailing zero-height Life tiles AND the single anchor tile just before
    // them into its contact-aligned cluster; if that anchor is already the
    // open parallel cluster, only the trailing Life tiles join it. Life tiles
    // never break a pending cluster (they sit between a message and its `&`
    // brother). A 4px Empty tile directly before a parallel run is dropped.
    let mut result: Vec<TeozItem> = Vec::new();
    for tile in raw {
        if teoz_tile_parallel(tile) && !result.is_empty() {
            while matches!(result.last(), Some(TeozItem::Empty)) {
                result.pop();
            }
            // Count trailing Life tiles (capture starts at 1 = last element).
            let mut capture = 1usize;
            while result.len() >= capture
                && matches!(result[result.len() - capture], TeozItem::One(TeozTile::Life { .. }))
            {
                capture += 1;
            }
            // The tile just beyond the trailing Life run: if it is the open
            // parallel cluster itself, Java moves only the Life tiles.
            let boundary_is_cluster = result.len() >= capture
                && matches!(result[result.len() - capture], TeozItem::Many(_));
            if boundary_is_cluster {
                capture -= 1;
            }
            let start = result.len() - capture;
            let mut members: Vec<TeozTile> = Vec::new();
            for it in &result[start..] {
                if let TeozItem::One(t) = it {
                    members.push(t.clone());
                }
            }
            result.truncate(start);
            members.push(tile.clone());
            if let Some(TeozItem::Many(existing)) = result.last_mut() {
                existing.extend(members);
            } else {
                result.push(TeozItem::Many(members));
            }
        } else {
            result.push(match tile {
                TeozTile::Empty => TeozItem::Empty,
                TeozTile::Else(c) => TeozItem::Else(c.clone()),
                other => TeozItem::One(other.clone()),
            });
        }
    }
    result
}

/// Sums the merged stream's reserved height.
fn teoz_stream_pref(raw: &[TeozTile], is_self: &[bool], rows: &[usize], gpref: &[f64], empty: &[bool]) -> f64 {
    let mut total = 0.0;
    for item in teoz_merge(raw) {
        match item {
            TeozItem::Empty => total += TEOZ_EMPTY_H,
            TeozItem::Else(_) => total += TEOZ_ELSE_H,
            TeozItem::One(t) => total += teoz_tile_pref(&t, is_self, rows, gpref, empty),
            TeozItem::Many(members) => {
                let cpr = members.iter().map(|t| teoz_tile_cpr(t, is_self, rows, empty)).fold(0.0, f64::max);
                let tail = members
                    .iter()
                    .map(|t| teoz_tile_pref(t, is_self, rows, gpref, empty) - teoz_tile_cpr(t, is_self, rows, empty))
                    .fold(0.0, f64::max);
                total += cpr + tail;
            }
        }
    }
    total
}

fn teoz_count_groups(nodes: &[TeozNode]) -> usize {
    let mut n = 0;
    for node in nodes {
        if let TeozNode::Group { gid, sections, .. } = node {
            n = n.max(gid + 1);
            for sec in sections {
                n = n.max(teoz_count_groups(&sec.children));
            }
        }
    }
    n
}

/// Collects keyword/comment/backcolor per pre-order gid.
fn teoz_group_meta(
    nodes: &[TeozNode],
    kind: &mut [String],
    comment: &mut [String],
    backcolor: &mut [Option<String>],
) {
    for node in nodes {
        if let TeozNode::Group { gid, kind: k, comment: c, backcolor: bc, sections, .. } = node {
            kind[*gid].clone_from(k);
            comment[*gid].clone_from(c);
            backcolor[*gid].clone_from(bc);
            for sec in sections {
                teoz_group_meta(&sec.children, kind, comment, backcolor);
            }
        }
    }
}

/// Computes the internal frame extent of one group and recurses into nested
/// groups post-order. `gid` identifies this group; `nodes` is the flattened
/// stream of all its sections' children.
fn teoz_frame_group(
    gid: usize,
    nodes: &[TeozNode],
    msg_x: &[(f64, f64)],
    life_x: &[(f64, f64)],
    header_w: &[f64],
    frames: &mut [(f64, f64)],
) {
    // Recurse into nested groups first so their public margins are available.
    for node in nodes {
        if let TeozNode::Group { gid: cgid, sections, .. } = node {
            teoz_frame_group(*cgid, &collect_section_children(sections), msg_x, life_x, header_w, frames);
        }
    }
    // Gather non-else child contributions; nested groups expose internal ±(-3/+9).
    let mut mn = f64::MAX;
    let mut mx = f64::MIN;
    for node in nodes {
        match node {
            TeozNode::Msg { idx, .. } => {
                let (a, b) = msg_x.get(*idx).copied().unwrap_or((f64::MAX, f64::MIN));
                mn = mn.min(a - GROUP_MARGIN_X);
                mx = mx.max(b + GROUP_MARGIN_X);
            }
            TeozNode::Group { gid: cgid, .. } => {
                let (imin, imax) = frames[*cgid];
                mn = mn.min(imin - GROUP_EXTERNAL_MARGIN_X1 - GROUP_MARGIN_X);
                mx = mx.max(imax + GROUP_EXTERNAL_MARGIN_X2 + GROUP_MARGIN_X);
            }
            TeozNode::Life { ev } => {
                let (a, b) = life_x.get(*ev).copied().unwrap_or((f64::MAX, f64::MIN));
                mn = mn.min(a - GROUP_MARGIN_X);
                mx = mx.max(b + GROUP_MARGIN_X);
            }
            TeozNode::Divider { .. } => {}
        }
    }
    let header_right = mn + header_w.get(gid).copied().unwrap_or(0.0) + GROUP_MARGIN_X;
    mx = mx.max(header_right);
    frames[gid] = (mn, mx);
}

/// Flattens a group's sections' direct child node lists into one stream.
fn collect_section_children(sections: &[TeozSection]) -> Vec<TeozNode> {
    let mut out = Vec::new();
    for sec in sections {
        out.extend(sec.children.iter().cloned());
    }
    out
}

/// Post-order measure: fills every group's preferred height and records each
/// group's sections by pre-order gid.
fn teoz_measure_nodes(
    nodes: &[TeozNode],
    is_self: &[bool],
    rows: &[usize],
    empty: &[bool],
    gpref: &mut [f64],
    gsec: &mut Vec<Vec<TeozSection>>,
) {
    for node in nodes {
        if let TeozNode::Group { gid, sections, .. } = node {
            for sec in sections {
                teoz_measure_nodes(&sec.children, is_self, rows, empty, gpref, gsec);
            }
            if gsec.len() <= *gid {
                gsec.resize(gid + 1, Vec::new());
            }
            gsec[*gid].clone_from(sections);
            let mut raw = Vec::new();
            teoz_raw_sections(sections, &mut raw);
            let body = teoz_stream_pref(&raw, is_self, rows, gpref, empty);
            gpref[*gid] = TEOZ_HEADER_H + body + TEOZ_GROUP_TAIL;
        }
    }
}
fn teoz_place_tile(
    tile: &TeozTile,
    hook: f64,
    is_self: &[bool],
    rows: &[usize],
    empty: &[bool],
    gpref: &[f64],
    gsec: &[Vec<TeozSection>],
    out: &mut TeozLayoutOut,
) {
    match tile {
        TeozTile::Msg { idx, .. } => {
            if out.msg_hook.len() <= *idx {
                out.msg_hook.resize(idx + 1, f64::NAN);
            }
            out.msg_hook[*idx] = hook;
        }
        TeozTile::Group { gid, .. } => {
            teoz_place_group(*gid, hook, is_self, rows, empty, gpref, gsec, out);
        }
        TeozTile::Life { ev } => {
            if out.life_hook.len() <= *ev {
                out.life_hook.resize(ev + 1, f64::NAN);
            }
            out.life_hook[*ev] = hook;
        }
        TeozTile::Empty | TeozTile::Else(_) | TeozTile::Divider { .. } => {}
    }
}

/// Lays out a group body (sections joined by Else separators) starting at
/// `y0`, recording the group hook on the caller's behalf via [`teoz_place_tile`].
fn teoz_place_body(
    sections: &[TeozSection],
    y0: f64,
    gid: usize,
    is_self: &[bool],
    rows: &[usize],
    empty: &[bool],
    gpref: &[f64],
    gsec: &[Vec<TeozSection>],
    out: &mut TeozLayoutOut,
) -> f64 {
    let mut raw = Vec::new();
    teoz_raw_sections(sections, &mut raw);
    let mut y = y0;
    for item in teoz_merge(&raw) {
        match item {
            TeozItem::Empty => y += TEOZ_EMPTY_H,
            TeozItem::Else(label) => {
                out.group_else[gid].push((y, label));
                y += TEOZ_ELSE_H;
            }
            TeozItem::One(t) => {
                let pref = teoz_tile_pref(&t, is_self, rows, gpref, empty);
                teoz_place_tile(&t, y, is_self, rows, empty, gpref, gsec, out);
                y += pref;
            }
            TeozItem::Many(members) => {
                let cpr = members.iter().map(|t| teoz_tile_cpr(t, is_self, rows, empty)).fold(0.0, f64::max);
                let tail = members
                    .iter()
                    .map(|t| teoz_tile_pref(t, is_self, rows, gpref, empty) - teoz_tile_cpr(t, is_self, rows, empty))
                    .fold(0.0, f64::max);
                for m in &members {
                    teoz_place_tile(m, y + (cpr - teoz_tile_cpr(m, is_self, rows, empty)), is_self, rows, empty, gpref, gsec, out);
                }
                y += cpr + tail;
            }
        }
    }
    y
}

fn teoz_place_group(
    gid: usize,
    hook: f64,
    is_self: &[bool],
    rows: &[usize],
    empty: &[bool],
    gpref: &[f64],
    gsec: &[Vec<TeozSection>],
    out: &mut TeozLayoutOut,
) {
    out.group_hook[gid] = hook;
    out.group_pref[gid] = gpref[gid];
    out.group_body[gid] = gpref[gid] - TEOZ_HEADER_H - TEOZ_GROUP_TAIL;
    let fallback = Vec::new();
    let sections = gsec.get(gid).unwrap_or(&fallback);
    teoz_place_body(sections, hook + TEOZ_HEADER_OFFSET, gid, is_self, rows, empty, gpref, gsec, out);
}
/// Runs the full teoz vertical layout over the parsed token tree.
fn layout_teoz(tree: &[TeozNode], n_msgs: usize, is_self: &[bool], rows: &[usize], empty: &[bool]) -> TeozLayoutOut {
    let n_groups = teoz_count_groups(tree);
    let mut gpref = vec![0.0; n_groups];
    let mut gsec: Vec<Vec<TeozSection>> = vec![Vec::new(); n_groups];
    teoz_measure_nodes(tree, is_self, rows, empty, &mut gpref, &mut gsec);

    let mut out = TeozLayoutOut {
        msg_hook: vec![f64::NAN; n_msgs],
        group_hook: vec![0.0; n_groups],
        group_pref: gpref.clone(),
        group_body: gpref.iter().map(|p| p - TEOZ_HEADER_H - TEOZ_GROUP_TAIL).collect(),
        group_else: vec![Vec::new(); n_groups],
        group_kind: vec![String::new(); n_groups],
        group_comment: vec![String::new(); n_groups],
        group_backcolor: vec![None; n_groups],
        ..Default::default()
    };
    teoz_group_meta(tree, &mut out.group_kind, &mut out.group_comment, &mut out.group_backcolor);

    // Root placement.
    let mut raw = Vec::new();
    teoz_raw_nodes(tree, &mut raw);
    let mut y = 8.0;
    for item in teoz_merge(&raw) {
        match item {
            TeozItem::Empty => y += TEOZ_EMPTY_H,
            TeozItem::Else(_) => y += TEOZ_ELSE_H,
            TeozItem::One(t) => {
                let pref = teoz_tile_pref(&t, is_self, rows, &gpref, empty);
                teoz_place_tile(&t, y, is_self, rows, empty, &gpref, &gsec, &mut out);
                y += pref;
            }
            TeozItem::Many(members) => {
                let cpr = members.iter().map(|t| teoz_tile_cpr(t, is_self, rows, empty)).fold(0.0, f64::max);
                let tail = members
                    .iter()
                    .map(|t| teoz_tile_pref(t, is_self, rows, &gpref, empty) - teoz_tile_cpr(t, is_self, rows, empty))
                    .fold(0.0, f64::max);
                for m in &members {
                    teoz_place_tile(m, y + (cpr - teoz_tile_cpr(m, is_self, rows, empty)), is_self, rows, empty, &gpref, &gsec, &mut out);
                }
                y += cpr + tail;
            }
        }
    }
    out.final_y = y;
    out
}

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

/// Expands a 3-digit `#RGB` shorthand to the 6-digit `#RRGGBB` form the JAR
/// emits (e.g. `#FFA` -> `#FFFFAA`); longer strings pass through unchanged.
fn expand_hex_shorthand(color: &str) -> String {
    let bytes = color.as_bytes();
    if bytes.len() == 4 && bytes[0] == b'#'
        && bytes[1..].iter().all(u8::is_ascii_hexdigit)
    {
        let mut out = String::with_capacity(7);
        out.push('#');
        for b in &bytes[1..] {
            out.push(*b as char);
            out.push(*b as char);
        }
        out
    } else {
        color.to_string()
    }
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
    /// Dividers (`== label ==`).
    pub dividers: Vec<DividerInfo>,
    /// Whether `autonumber` is active.
    pub autonumber: bool,
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
    /// Whether `!pragma teoz true` is active (teoz sequence rendering branch).
    pub teoz: bool,
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
    /// Source line numbers for each participant (1-indexed, relative to @startuml).
    pub participant_source_lines: Vec<Option<usize>>,
    /// Source line numbers for each message (1-indexed, relative to @startuml).
    pub msg_source_lines: Vec<Option<usize>>,
    /// Source-order structural tokens for the teoz tile tree (empty for the
    /// legacy renderer).
    pub teoz_toks: Vec<TeozTok>,
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
            &self.dividers,
            self.autonumber,
            &self.msg_activates,
            &self.msg_deactivates,
            &self.msg_parallel,
            self.max_message_size,
            &self.msg_exo,
            &self.msg_hidden,
            self.skin_rose,
            self.teoz,
            self.arrow_color.as_deref(),
            self.header_text.as_deref(),
            self.header_line,
            self.footer_text.as_deref(),
            self.footer_line,
            self.legend_text.as_deref(),
            self.caption_text.as_deref(),
            self.caption_line,
            &self.style_rules,
            &self.participant_source_lines,
            &self.msg_source_lines,
            &self.teoz_toks,
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
    let mut dividers: Vec<DividerInfo> = Vec::new();
    let mut autonumber = false;
    let mut msg_count = 0usize;
    // Monotonic id for LifeEvent tiles (indexes the LifeEvent subsequence of
    // diagram.events()); one TeozTok::Life is emitted per diagram life event.
    let mut life_count = 0usize;
    // Group parsing state: stack of (msg_start, group_type, comment, nesting_level)
    let mut group_stack: Vec<(usize, String, String, usize, bool, Option<String>)> = Vec::new();
    // Source-order structural tokens for the teoz tile tree.
    let mut teoz_toks: Vec<TeozTok> = Vec::new();
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
    let mut msg_source_lines: Vec<Option<usize>> = Vec::new();
    // Track first source line for each participant code
    let mut pcode_to_source_line: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut next_msg_parallel = false;
    let mut autoactivate = false;
    let mut in_note_block: Option<(NotePosition, Vec<String>)> = None;
    let mut max_message_size: Option<f64> = None;
    let mut skin_rose = false;
    let mut teoz = false;
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
        // autonumber (may carry start/inc/format args we currently ignore)
        if trimmed == "autonumber" || trimmed.starts_with("autonumber ") {
            autonumber = true;
            continue;
        }
        // Divider: `== label ==` (CommandDivider regex `^==(.*)==$`)
        if trimmed.starts_with("==") && trimmed.ends_with("==") && trimmed.len() >= 4 {
            if let Some(inner) = trimmed.strip_prefix("==").and_then(|s| s.strip_suffix("==")) {
                let label = inner.trim();
                if !label.is_empty() {
                    dividers.push(DividerInfo { text: label.to_string(), before_msg: msg_count });
                    teoz_toks.push(TeozTok::Divider { text: label.to_string(), before_msg: msg_count });
                }
            }
            continue;
        }
        // `!pragma teoz true` selects the teoz rendering branch; tolerate
        // ASCII case differences and extra whitespace. Other pragmas are
        // ignored by the simplified renderer.
        if trimmed.starts_with("!pragma ") {
            let normalized: String = trimmed
                .to_ascii_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if normalized == "!pragma teoz true" {
                teoz = true;
            }
            continue;
        }
        if trimmed.starts_with('\'')
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
            if let Some(rest) = trimmed.strip_prefix("skinparam backgroundColor ") {
                if let Some(hex) = resolve_color(rest.trim()) {
                    style_rules
                        .entry("document".to_string())
                        .or_default()
                        .entry("BackGroundColor".to_string())
                        .or_insert(hex);
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

        if trimmed == "end" {
            teoz_toks.push(TeozTok::GroupEnd);
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
            teoz_toks.push(TeozTok::Else { comment: else_comment.clone() });
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
                (title, Some(expand_hex_shorthand(&color.to_ascii_uppercase())))
            } else {
                (raw_comment, None)
            };
            let is_parallel = next_msg_parallel;
            next_msg_parallel = false;
            teoz_toks.push(TeozTok::GroupOpen {
                kind: group_keyword.to_string(),
                comment: comment.clone(),
                parallel: is_parallel,
                backcolor: backcolor.clone(),
            });
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
            // An implicit participant first introduced by `activate` is
            // attributed to the activate line (Java LineLocation); a later
            // message must not relocate it. or_insert keeps an earlier explicit
            // declaration / message line.
            pcode_to_source_line.entry(pname.to_string()).or_insert(line_num - startuml_line.unwrap_or(0));
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
            teoz_toks.push(TeozTok::Life { ev: life_count });
            life_count += 1;
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
            teoz_toks.push(TeozTok::Life { ev: life_count });
            life_count += 1;
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
            teoz_toks.push(TeozTok::Life { ev: life_count });
            life_count += 1;
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
                pcode_to_source_line.entry(code.clone()).or_insert(line_num - startuml_line.unwrap_or(0));
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
                msg_source_lines.push(Some(line_num - startuml_line.unwrap_or(0)));
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
                            participant: None,
                            msg_index: msg_count - 1,
                        });
                    }
                }
            } else {
                in_note_block.as_mut().expect("checked above").1.push(trimmed.to_string());
            }
            continue;
        }
        // A line containing a colon is a one-line (inline) note and must not
        // open a multi-line "note ... end note" block; those are handled below.
        if !trimmed.contains(':')
            && (trimmed == "note right" || trimmed.starts_with("note right "))
        {
            in_note_block = Some((NotePosition::Right, Vec::new()));
            continue;
        }
        if !trimmed.contains(':')
            && (trimmed == "note left" || trimmed.starts_with("note left "))
        {
            in_note_block = Some((NotePosition::Left, Vec::new()));
            continue;
        }
        if !trimmed.contains(':')
            && (trimmed == "note over" || trimmed.starts_with("note over "))
        {
            in_note_block = Some((NotePosition::Right, Vec::new()));
            continue;
        }

        // Handle "note right:" and "note left:" (single-line notes)
        if let Some(val) = trimmed.strip_prefix("note right:") {
            if msg_count > 0 {
                notes.push(NoteInfo {
                    position: NotePosition::Right,
                    text: val.trim().to_string(),
                    participant: None,
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
                    participant: None,
                    msg_index: msg_count - 1,
                });
            }
            continue;
        }

        // Handle standalone notes: "note right of P: text", "note left of P: text",
        // and "note over P1, P2: text".
        // Ported from: command/note/sequence/FactorySequenceNoteCommand.java
        if let Some(rest) = trimmed.strip_prefix("note ") {
            let parse_note = |s: &str| -> Option<(NotePosition, String, Option<String>, String)> {
                let (pos, rest) = if let Some(r) = s.strip_prefix("right") {
                    (NotePosition::Right, r)
                } else if let Some(r) = s.strip_prefix("left") {
                    (NotePosition::Left, r)
                } else {
                    let r = s.strip_prefix("over")?;
                    (NotePosition::Right, r)
                };
                let rest = rest.trim_start();
                let rest = rest.strip_prefix("of").unwrap_or(rest);
                let rest = rest.trim_start();
                let colon = rest.find(':')?;
                let target = rest[..colon].trim().to_string();
                let text = rest[colon + 1..].trim();
                let (p1, p2) = if let Some((a, b)) = target.split_once(',') {
                    (a.trim().to_string(), Some(b.trim().to_string()))
                } else {
                    (target, None)
                };
                Some((pos, p1, p2, text.to_string()))
            };
            if let Some((position, p1, p2, text)) = parse_note(rest) {
                notes.push(NoteInfo {
                    position,
                    text,
                    participant: Some(if p2.is_some() {
                        format!("{p1},{}", p2.as_deref().unwrap_or(""))
                    } else {
                        p1
                    }),
                    msg_index: msg_count.saturating_sub(1),
                });
                continue;
            }
        }
        if let Some((p1_code, p2_code, label, mut arrow_config, inline_activate, inline_deactivate, inline_destroy, arrow_style)) = parse_arrow_line(trimmed) {
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
            // Track source lines for participants created by this arrow
            let source_line = line_num - startuml_line.unwrap_or(0);
            if !is_left_exo { pcode_to_source_line.entry(p1_code.clone()).or_insert(source_line); }
            if !is_right_exo { pcode_to_source_line.entry(p2_code.clone()).or_insert(source_line); }
            msg_source_lines.push(Some(source_line));
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
            if let Some(style) = &arrow_style {
                arrow_config = apply_arrow_style(arrow_config, style);
            }
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
            let n_inline_life = acts.len() + deacts.len() + dests.len();
            msg_activates.push(acts);
            let mut all_deacts = deacts.clone();
            all_deacts.extend(dests.iter().cloned());
            msg_deactivates.push(all_deacts);
            let in_parallel_group = group_stack.iter().any(|(_, _, _, _, is_par, _)| *is_par);
            msg_parallel.push(next_msg_parallel || in_parallel_group);
            msg_exo.push(exo);
            msg_hidden.push(trimmed.contains("[hidden]"));
            teoz_toks.push(TeozTok::Msg { idx: msg_count, parallel: next_msg_parallel });
            // Inline LifeEvents are appended to the diagram right after the
            // message (acts, then deacts, then destroys). Mirror that order as
            // zero-height Life tiles so hooks/parallel capture stay faithful.
            for _ in 0..n_inline_life {
                teoz_toks.push(TeozTok::Life { ev: life_count });
                life_count += 1;
            }
            next_msg_parallel = false;
            msg_count += 1;
        }
    }

    if diagram.participants().is_empty() {
        None
    } else {
        // Build participant_source_lines from pcode_to_source_line
        let participant_source_lines: Vec<Option<usize>> = diagram.participants().iter()
            .map(|p| pcode_to_source_line.get(p.code()).copied())
            .collect();
        Some(ParsedSequence {
            diagram,
            svg_title,
            svg_desc,
            title,
            title_line,
            hide_footbox,
            notes,
            groups,
            dividers,
            autonumber,
            msg_activates,
            msg_deactivates,
            msg_parallel,
            max_message_size,
            msg_exo,
            msg_hidden,
            skin_rose,
            teoz,
            arrow_color,
            header_text,
            header_line,
            footer_text,
            footer_line,
            legend_text,
            caption_text,
            caption_line,
            style_rules,
            participant_source_lines,
            msg_source_lines,
            teoz_toks,
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

/// Parsed fields of one sequence arrow line:
/// `(p1_code, p2_code, label, arrow_config, inline_activate,
/// inline_deactivate, inline_destroy, arrow_style)`.
type ParsedArrowLine = (
    String,
    String,
    String,
    plantuml_skin::ArrowConfiguration,
    bool,
    bool,
    bool,
    Option<String>,
);

/// Parses a `PlantUML` sequence arrow line, extracting participants, label, and arrow configuration.
///
/// Ported from: `net/sourceforge/plantuml/sequencediagram/command/CommandArrow.java`
///
/// Returns `(p1_code, p2_code, label, arrow_config, inline_activate, inline_deactivate, inline_destroy)`.
fn parse_arrow_line(line: &str) -> Option<ParsedArrowLine> {
    // Capture the [style] bracket content ("-[#red]->", "-[hidden]->"), then
    // remove the bracket from the notation for arrow-body parsing. The tokens
    // are returned and applied to the ArrowConfiguration by the caller,
    // matching Java CommandArrow.applyStyle (CommandArrow.java:482-502).
    let mut arrow_style: Option<String> = None;
    let line = if let Some(bracket_start) = line.find("-[") {
        if let Some(bracket_end) = line[bracket_start..].find(']') {
            arrow_style = Some(line[bracket_start + 2..bracket_start + bracket_end].to_string());
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
    candidates.sort_by_key(|&(s, e)| std::cmp::Reverse(e - s));

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
            arrow_style,
        ));
    }
    None
}

/// Returns whether a source line parses as a sequence arrow line.
/// Used by the sequence factory's context-sensitive keyword veto.
#[must_use]
pub fn parse_arrow_line_public(line: &str) -> bool {
    parse_arrow_line(line).is_some()
}

/// Applies the comma-separated tokens of an arrow bracket style (e.g. `[#red,dashed]`)
/// to an `ArrowConfiguration`.
///
/// Ported from: `net.sourceforge.plantuml.sequencediagram.command.CommandArrow.applyStyle`
/// (CommandArrow.java:482-502): `dashed`/`dotted` set a dotted body, `hidden`
/// hides the arrow, `bold` is ignored, and any other token names a color
/// (`#red`, `#FF0000`).
fn apply_arrow_style(
    mut config: plantuml_skin::ArrowConfiguration,
    style: &str,
) -> plantuml_skin::ArrowConfiguration {
    use plantuml_skin::ArrowBody;
    for token in style.split(',') {
        let s = token.trim();
        if s.is_empty() {
            continue;
        }
        if s.eq_ignore_ascii_case("dashed") || s.eq_ignore_ascii_case("dotted") {
            config = config.with_body(ArrowBody::Dotted);
        } else if s.eq_ignore_ascii_case("hidden") {
            config = config.with_body(ArrowBody::Hidden);
        } else if s.eq_ignore_ascii_case("bold") {
        } else {
            // Java HColorSet.getColor accepts "#red" (name) as well as "#FF0000"
            // (hex): strip the leading '#', look up a color name, else parse hex.
            let without_hash = s.strip_prefix('#').unwrap_or(s);
            let hex = color_name_to_hex(without_hash)
                .or_else(|| resolve_color(s))
                .unwrap_or_default();
            if let Some((r, g, b)) = parse_hex_color(&hex) {
                config = config.with_color(plantuml_klimt::color::HColor::rgb(r, g, b));
            }
        }
    }
    config
}

/// Computes the maximum line width for a (potentially multi-line) label.
/// Splits on literal `\n` and returns the width of the widest line.
fn max_line_width(bounder: &dyn StringBounder, font: &UFont, text: &str) -> f64 {
    text.split("\\n")
        .map(|line| bounder.calculate_dimension(font, line).width())
        .fold(0.0_f64, f64::max)
}

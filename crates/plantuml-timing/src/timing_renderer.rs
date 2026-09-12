//! Timing diagram SVG renderer.
//!
//! Ported from: `net/sourceforge/plantuml/timingdiagram/TimingDiagram.drawU()`.
//!
//! Renders timing signals as horizontal waveform lanes.

use plantuml_svg::{SvgGraphics, SvgOption};

use crate::timing_parser::{Signal, SignalType, TimingSource};

// ── Rendering constants ──────────────────────────────────────────────────

/// Left margin for signal labels.
const LABEL_WIDTH: f64 = 80.0;
/// Height of each signal lane.
const LANE_HEIGHT: f64 = 40.0;
/// Width per time unit.
const TIME_UNIT: f64 = 40.0;
/// Top margin.
const TOP_MARGIN: f64 = 20.0;
/// Left margin (after label area).
const LEFT_MARGIN: f64 = 10.0;
/// Stroke color.
const STROKE: &str = "#181818";
/// Fill color for high state.
/// Fill color for low state.
/// Text color.
const COLOR_TEXT: &str = "#000000";
/// Font size.
const FONT_SIZE: i32 = 12;
/// Font family.
const FONT_FAMILY: &str = "sans-serif";

/// Renders a timing diagram as SVG.
#[must_use]
pub fn render_timing_svg(source: &TimingSource, diagram_label: &str) -> String {
    let mut option = SvgOption::basic();
    option.set_title(diagram_label.to_string());

    let mut svg = SvgGraphics::new(0, option);

    if source.signals.is_empty() {
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), "#0000FF".to_string());
        svg.text(
            "Empty timing diagram",
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

    // Compute total time steps (max changes across all signals).
    let max_changes = source
        .signals
        .values()
        .map(|s| s.changes.len())
        .max()
        .unwrap_or(1);
    let total_width = (max_changes as f64 + 1.0).mul_add(TIME_UNIT, LABEL_WIDTH + LEFT_MARGIN);
    let _total_height = (source.signals.len() as f64).mul_add(LANE_HEIGHT, TOP_MARGIN) + 10.0;

    // Draw signal lanes.
    for (i, signal) in source.signals.values().enumerate() {
        let y = (i as f64).mul_add(LANE_HEIGHT, TOP_MARGIN);
        render_signal_lane(&mut svg, signal, y);
    }

    // Draw time axis at the bottom.
    let axis_y = (source.signals.len() as f64).mul_add(LANE_HEIGHT, TOP_MARGIN);
    svg.set_stroke_color(Some(STROKE));
    svg.set_stroke_width(1.0, None);
    svg.svg_line(
        LABEL_WIDTH + LEFT_MARGIN,
        axis_y,
        total_width,
        axis_y,
        0.0,
    );

    // Draw time markers.
    for t in 0..=max_changes {
        let x = (t as f64).mul_add(TIME_UNIT, LABEL_WIDTH + LEFT_MARGIN);
        svg.svg_line(x, axis_y, x, axis_y + 5.0, 0.0);
        let label = t.to_string();
        let label_w = text_width(&label, FONT_SIZE);
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
        svg.text(
            &label,
            x - label_w / 2.0,
            axis_y + 18.0,
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

    svg.create_xml()
}

/// Renders a single signal lane.
fn render_signal_lane(svg: &mut SvgGraphics, signal: &Signal, y: f64) {
    let lane_mid = y + LANE_HEIGHT / 2.0;
    let high_y = y + 5.0;
    let low_y = y + LANE_HEIGHT - 5.0;

    // Draw signal name label.
    let label_w = text_width(&signal.name, FONT_SIZE);
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        &signal.name,
        LEFT_MARGIN,
        f64::from(FONT_SIZE).mul_add(0.35, lane_mid),
        Some(FONT_FAMILY),
        FONT_SIZE,
        Some("normal"),
        Some("normal"),
        None,
        label_w,
        &attrs,
        None,
    );

    // Draw lane separator.
    svg.set_stroke_color(Some("#CCCCCC"));
    svg.set_stroke_width(0.5, None);
    svg.svg_line(LABEL_WIDTH, y + LANE_HEIGHT, LABEL_WIDTH + LEFT_MARGIN + 500.0, y + LANE_HEIGHT, 0.0);

    svg.set_stroke_color(Some(STROKE));
    svg.set_stroke_width(1.5, None);

    if signal.changes.is_empty() {
        return;
    }

    let x0 = LABEL_WIDTH + LEFT_MARGIN;

    match signal.signal_type {
        SignalType::Binary | SignalType::Digital => {
            render_binary_waveform(svg, signal, x0, high_y, low_y);
        }
        SignalType::Clock => {
            render_clock_waveform(svg, signal, x0, high_y, low_y);
        }
        SignalType::Analog => {
            render_analog_waveform(svg, signal, x0, high_y, low_y);
        }
        SignalType::Hexa => {
            render_hex_waveform(svg, signal, x0, lane_mid);
        }
    }
}

/// Renders a binary waveform (square wave).
fn render_binary_waveform(
    svg: &mut SvgGraphics,
    signal: &Signal,
    x0: f64,
    high_y: f64,
    low_y: f64,
) {
    let mut prev_y = if signal.changes[0].value == "1" || signal.changes[0].value == "high" {
        high_y
    } else {
        low_y
    };

    // Draw initial level.
    svg.svg_line(x0, prev_y, x0 + TIME_UNIT, prev_y, 0.0);

    for (i, change) in signal.changes.iter().enumerate().skip(1) {
        let x = (i as f64).mul_add(TIME_UNIT, x0);
        let new_y = if change.value == "1" || change.value == "high" {
            high_y
        } else {
            low_y
        };

        // Vertical transition.
        if (new_y - prev_y).abs() > f64::EPSILON {
            svg.svg_line(x, prev_y, x, new_y, 0.0);
        }
        // Horizontal level.
        svg.svg_line(x, new_y, x + TIME_UNIT, new_y, 0.0);
        prev_y = new_y;
    }
}

/// Renders a clock waveform (regular pulses).
fn render_clock_waveform(
    svg: &mut SvgGraphics,
    signal: &Signal,
    x0: f64,
    high_y: f64,
    low_y: f64,
) {
    for (i, _change) in signal.changes.iter().enumerate() {
        let x = (i as f64).mul_add(TIME_UNIT, x0);
        // Rising edge.
        svg.svg_line(x, low_y, x, high_y, 0.0);
        // High level (half period).
        svg.svg_line(x, high_y, x + TIME_UNIT / 2.0, high_y, 0.0);
        // Falling edge.
        svg.svg_line(x + TIME_UNIT / 2.0, high_y, x + TIME_UNIT / 2.0, low_y, 0.0);
        // Low level (half period).
        svg.svg_line(x + TIME_UNIT / 2.0, low_y, x + TIME_UNIT, low_y, 0.0);
    }
}

/// Renders an analog waveform (linear interpolation).
fn render_analog_waveform(
    svg: &mut SvgGraphics,
    signal: &Signal,
    x0: f64,
    high_y: f64,
    low_y: f64,
) {
    for (i, change) in signal.changes.iter().enumerate() {
        let x = (i as f64).mul_add(TIME_UNIT, x0);
        let value: f64 = change.value.parse().unwrap_or(0.0);
        let y = (1.0 - value).mul_add(low_y - high_y, high_y);

        if i > 0 {
            let prev_x = ((i - 1) as f64).mul_add(TIME_UNIT, x0);
            let prev_value: f64 = signal.changes[i - 1].value.parse().unwrap_or(0.0);
            let prev_y = (1.0 - prev_value).mul_add(low_y - high_y, high_y);
            svg.svg_line(prev_x, prev_y, x, y, 0.0);
        }
        svg.svg_line(x, y, x + TIME_UNIT, y, 0.0);
    }
}

/// Renders a hexadecimal waveform (value labels).
fn render_hex_waveform(
    svg: &mut SvgGraphics,
    signal: &Signal,
    x0: f64,
    lane_mid: f64,
) {
    for (i, change) in signal.changes.iter().enumerate() {
        let x = (i as f64).mul_add(TIME_UNIT, x0);
        let label_w = text_width(&change.value, FONT_SIZE);
        let mut attrs = indexmap::IndexMap::new();
        attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
        svg.text(
            &change.value,
            x + (TIME_UNIT - label_w) / 2.0,
            f64::from(FONT_SIZE).mul_add(0.35, lane_mid),
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

/// Approximate text width.
fn text_width(text: &str, font_size: i32) -> f64 {
    f64::from(font_size) * 0.6 * text.chars().count() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing_parser::parse_timing_source;

    #[test]
    fn test_render_timing_svg() {
        let lines = vec!["binary A", "binary B", "A = 0", "B = 0", "A = 1", "B = 1"];
        let source = parse_timing_source(&lines);
        let svg = render_timing_svg(&source, "(Timing)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("A"));
        assert!(svg.contains("B"));
    }

    #[test]
    fn test_render_empty_timing() {
        let source = TimingSource::default();
        let svg = render_timing_svg(&source, "(Timing)");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Empty"));
    }
}

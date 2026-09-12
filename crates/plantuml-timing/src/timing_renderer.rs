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

    // Compute max time value across all signals (for axis width).
    let max_time = source
        .signals
        .values()
        .filter_map(|s| s.changes.last().map(|c| c.time))
        .map(|t| t + 1.0)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);
    let total_width = max_time.mul_add(TIME_UNIT, LABEL_WIDTH + LEFT_MARGIN);
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

    // Draw time markers at integer positions.
    let max_tick = max_time.ceil() as i32;
    for t in 0..=max_tick {
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

    // Draw signal name label (prefer display_name if provided).
    let label = if signal.display_name.is_empty() {
        &signal.name
    } else {
        &signal.display_name
    };
    let label_w = text_width(label, FONT_SIZE);
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert("fill".to_string(), COLOR_TEXT.to_string());
    svg.text(
        label,
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
        SignalType::Binary | SignalType::Digital | SignalType::Robust | SignalType::Concise => {
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
    let mut prev_y = if is_high(&signal.changes[0].value) {
        high_y
    } else {
        low_y
    };

    // Draw initial level from first change time to next change.
    let first_x = signal.changes[0].time * TIME_UNIT + x0;
    let next_x = if signal.changes.len() > 1 {
        signal.changes[1].time * TIME_UNIT + x0
    } else {
        first_x + TIME_UNIT
    };
    svg.svg_line(first_x, prev_y, next_x, prev_y, 0.0);

    for (i, change) in signal.changes.iter().enumerate().skip(1) {
        let x = change.time * TIME_UNIT + x0;
        let new_y = if is_high(&change.value) {
            high_y
        } else {
            low_y
        };

        // Vertical transition.
        if (new_y - prev_y).abs() > f64::EPSILON {
            svg.svg_line(x, prev_y, x, new_y, 0.0);
        }
        // Horizontal level until next change (or +TIME_UNIT if last).
        let end_x = if i + 1 < signal.changes.len() {
            signal.changes[i + 1].time * TIME_UNIT + x0
        } else {
            x + TIME_UNIT
        };
        svg.svg_line(x, new_y, end_x, new_y, 0.0);
        prev_y = new_y;
    }
}

/// Returns true if the value represents a high state.
fn is_high(value: &str) -> bool {
    matches!(
        value.to_lowercase().as_str(),
        "1" | "high" | "true" | "on"
    )
}

/// Renders a clock waveform (regular pulses).
fn render_clock_waveform(
    svg: &mut SvgGraphics,
    signal: &Signal,
    x0: f64,
    high_y: f64,
    low_y: f64,
) {
    let period = signal.period.unwrap_or(1.0);
    let half = TIME_UNIT * period / 2.0;
    for change in &signal.changes {
        let x = change.time * TIME_UNIT + x0;
        // Rising edge.
        svg.svg_line(x, low_y, x, high_y, 0.0);
        // High level (half period).
        svg.svg_line(x, high_y, x + half, high_y, 0.0);
        // Falling edge.
        svg.svg_line(x + half, high_y, x + half, low_y, 0.0);
        // Low level (half period).
        svg.svg_line(x + half, low_y, x + TIME_UNIT * period, low_y, 0.0);
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
        let x = change.time * TIME_UNIT + x0;
        let value: f64 = change.value.parse().unwrap_or(0.0);
        let y = (1.0 - value).mul_add(low_y - high_y, high_y);

        if i > 0 {
            let prev_x = signal.changes[i - 1].time * TIME_UNIT + x0;
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
    for change in &signal.changes {
        let x = change.time * TIME_UNIT + x0;
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

//! Timing diagram source parser.
//!
//! Ported from: `net/sourceforge/plantuml/timingdiagram/TimingDiagram.java`
//! and `TimingFormat.java`.
//!
//! Parses timing diagram syntax: signal declarations and state changes.

use indexmap::IndexMap;

/// Type of timing signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    /// Binary signal (0/1).
    Binary,
    /// Hexadecimal signal.
    Hexa,
    /// Analog signal (continuous values).
    Analog,
    /// Clock signal (regular pulses).
    Clock,
    /// Digital signal (multi-valued).
    Digital,
}

/// A signal change at a specific time.
#[derive(Debug, Clone)]
pub struct SignalChange {
    /// Time value (relative).
    pub time: f64,
    /// New value (as string for flexibility: "0", "1", "HIGH", etc.).
    pub value: String,
}

/// A timing signal with its state changes.
#[derive(Debug, Clone)]
pub struct Signal {
    /// Signal name.
    pub name: String,
    /// Signal type.
    pub signal_type: SignalType,
    /// List of state changes in time order.
    pub changes: Vec<SignalChange>,
    /// Optional initial value.
    pub initial_value: Option<String>,
}

/// Parsed timing diagram source.
#[derive(Debug, Clone, Default)]
pub struct TimingSource {
    /// Signals keyed by name.
    pub signals: IndexMap<String, Signal>,
    /// Optional title.
    pub title: Option<String>,
    /// Optional caption.
    pub caption: Option<String>,
}

/// Parses timing diagram source lines.
///
/// Ported from: `TimingDiagram.parseTimingFormat()`.
#[must_use]
pub fn parse_timing_source(lines: &[&str]) -> TimingSource {
    let mut source = TimingSource::default();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // Skip @start/@end directives.
        if trimmed.starts_with("@start") || trimmed.starts_with("@end") {
            continue;
        }

        // Parse signal declarations: `binary A`, `clock A`, etc.
        if let Some(signal) = parse_signal_declaration(trimmed) {
            source.signals.insert(signal.name.clone(), signal);
            continue;
        }

        // Parse state changes: `A = 0`, `A = 1`, `A = "HIGH"`.
        if let Some((name, value)) = parse_state_change(trimmed) {
            if let Some(signal) = source.signals.get_mut(&name) {
                let time = signal.changes.len() as f64;
                signal.changes.push(SignalChange { time, value });
            }
            continue;
        }

        // Parse title.
        if let Some(rest) = trimmed.strip_prefix("title ") {
            source.title = Some(rest.trim_matches('"').to_string());
            continue;
        }

        // Parse caption.
        if let Some(rest) = trimmed.strip_prefix("caption ") {
            source.caption = Some(rest.trim_matches('"').to_string());

        }
    }

    source
}

/// Parses a signal declaration line (e.g. `binary A`, `clock B`).
fn parse_signal_declaration(line: &str) -> Option<Signal> {
    let (keyword, rest) = line.split_once(' ')?;
    let signal_type = match keyword.to_lowercase().as_str() {
        "binary" => SignalType::Binary,
        "hexa" | "hexadecimal" => SignalType::Hexa,
        "analog" => SignalType::Analog,
        "clock" => SignalType::Clock,
        "digital" => SignalType::Digital,
        _ => return None,
    };

    let name = rest.trim().trim_matches('"').to_string();
    if name.is_empty() {
        return None;
    }

    Some(Signal {
        name,
        signal_type,
        changes: Vec::new(),
        initial_value: None,
    })
}

/// Parses a state change line (e.g. `A = 0`, `A = "HIGH"`).
fn parse_state_change(line: &str) -> Option<(String, String)> {
    let (name, value) = line.split_once('=')?;
    let name = name.trim().to_string();
    let value = value.trim().trim_matches('"').to_string();
    if name.is_empty() || value.is_empty() {
        return None;
    }
    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_binary_signals() {
        let lines = vec!["binary A", "binary B", "A = 0", "B = 0", "A = 1", "B = 1"];
        let source = parse_timing_source(&lines);
        assert_eq!(source.signals.len(), 2);
        assert_eq!(source.signals["A"].signal_type, SignalType::Binary);
        assert_eq!(source.signals["A"].changes.len(), 2);
        assert_eq!(source.signals["A"].changes[0].value, "0");
        assert_eq!(source.signals["A"].changes[1].value, "1");
    }

    #[test]
    fn test_parse_clock_signal() {
        let lines = vec!["clock clk", "clk = 0", "clk = 1"];
        let source = parse_timing_source(&lines);
        assert_eq!(source.signals.len(), 1);
        assert_eq!(source.signals["clk"].signal_type, SignalType::Clock);
    }

    #[test]
    fn test_parse_title() {
        let lines = vec!["title \"My Timing Diagram\"", "binary A"];
        let source = parse_timing_source(&lines);
        assert_eq!(source.title, Some("My Timing Diagram".to_string()));
    }
}

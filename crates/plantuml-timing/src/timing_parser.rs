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
    /// Robust signal (multi-state with transitions).
    Robust,
    /// Concise signal (compact multi-state).
    Concise,
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
    /// Signal code (alias used to reference the signal in state changes).
    pub name: String,
    /// Display name (the quoted "full" name shown in rendering, if provided).
    pub display_name: String,
    /// Signal type.
    pub signal_type: SignalType,
    /// List of state changes in time order.
    pub changes: Vec<SignalChange>,
    /// Optional initial value.
    pub initial_value: Option<String>,
    /// Clock period (for clock signals).
    pub period: Option<f64>,
}

/// Parsed timing diagram source.
#[derive(Debug, Clone, Default)]
pub struct TimingSource {
    /// Signals keyed by code (alias).
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
    let mut current_time: f64 = 0.0;
    let mut last_signal_code: Option<String> = None;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // Skip @start/@end directives.
        if trimmed.starts_with("@start") || trimmed.starts_with("@end") {
            continue;
        }

        // Parse signal declarations: `binary A`, `binary "Full" as A`, `clock C`, etc.
        if let Some(signal) = parse_signal_declaration(trimmed) {
            last_signal_code = Some(signal.name.clone());
            source.signals.insert(signal.name.clone(), signal);
            continue;
        }

        // Parse time markers: `@0`, `@5`, `@10`, etc.
        if let Some(time) = parse_time_marker(trimmed) {
            current_time = time;
            continue;
        }

        // Parse state changes: `A = 0`, `A = "HIGH"`, `A is low`, `A is high`.
        if let Some((name, value)) = parse_state_change(trimmed) {
            if let Some(signal) = source.signals.get_mut(&name) {
                signal.changes.push(SignalChange {
                    time: current_time,
                    value,
                });
            }
            continue;
        }

        // Parse state change by time (uses last signal): `@0 is low` handled above by time marker.
        // Parse state change with `is` using last signal code: `is low` (implicit subject).
        if let Some(value) = parse_implicit_state_change(trimmed) {
            if let Some(code) = &last_signal_code {
                if let Some(signal) = source.signals.get_mut(code) {
                    signal.changes.push(SignalChange {
                        time: current_time,
                        value,
                    });
                }
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

/// Parses a signal declaration line.
///
/// Supports:
/// - `binary A` (simple code)
/// - `binary "Signal A" as A` (quoted display name + alias)
/// - `clock "clk" as C with period 10` (clock with period)
/// - `robust "Name" as R`, `concise "Name" as C`
fn parse_signal_declaration(line: &str) -> Option<Signal> {
    let (keyword, rest) = line.split_once(' ')?;
    let signal_type = match keyword.to_lowercase().as_str() {
        "binary" => SignalType::Binary,
        "hexa" | "hexadecimal" => SignalType::Hexa,
        "analog" => SignalType::Analog,
        "clock" => SignalType::Clock,
        "digital" => SignalType::Digital,
        "robust" => SignalType::Robust,
        "concise" => SignalType::Concise,
        _ => return None,
    };

    let rest = rest.trim();

    // Try to parse: "Display Name" as CODE [with period N]
    let (display_name, code, period) = if let Some(rest) = rest.strip_prefix('"') {
        // Quoted display name: "Full Name" as CODE
        let close = rest.find('"')?;
        let full = rest[..close].to_string();
        let after_quote = rest[close + 1..].trim();

        // Expect `as CODE`
        let code_part = after_quote.strip_prefix("as ")?.trim();

        // Check for `with period N` (clock signals)
        let (code, period) = if let Some(period_pos) = code_part.find(" with period ") {
            let code = code_part[..period_pos].trim().to_string();
            let period_str = code_part[period_pos + 13..].trim();
            let period = period_str.parse::<f64>().ok();
            (code, period)
        } else {
            (code_part.to_string(), None)
        };

        (full, code, period)
    } else {
        // Simple code: `binary A` (no display name)
        // Check for `with period N` (clock signals)
        let (code, period) = if let Some(period_pos) = rest.find(" with period ") {
            let code = rest[..period_pos].trim().to_string();
            let period_str = rest[period_pos + 13..].trim();
            let period = period_str.parse::<f64>().ok();
            (code, period)
        } else {
            (rest.to_string(), None)
        };
        (code.clone(), code, period)
    };

    let code = code.trim().trim_matches('"').to_string();
    if code.is_empty() {
        return None;
    }

    Some(Signal {
        name: code,
        display_name,
        signal_type,
        changes: Vec::new(),
        initial_value: None,
        period,
    })
}

/// Parses a time marker line: `@0`, `@5`, `@10`, etc.
fn parse_time_marker(line: &str) -> Option<f64> {
    let rest = line.strip_prefix('@')?;
    rest.trim().parse::<f64>().ok()
}

/// Parses a state change line.
///
/// Supports:
/// - `A = 0`, `A = "HIGH"` (equals syntax)
/// - `A is low`, `A is high`, `A is "HIGH"` (is syntax)
fn parse_state_change(line: &str) -> Option<(String, String)> {
    // Try `=` syntax: `A = 0`
    if let Some((name, value)) = line.split_once('=') {
        let name = name.trim().to_string();
        let value = value.trim().trim_matches('"').to_string();
        if name.is_empty() || value.is_empty() {
            return None;
        }
        return Some((name, value));
    }

    // Try `is` syntax: `A is low`
    if let Some((name, value)) = line.split_once(" is ") {
        let name = name.trim().to_string();
        let value = value.trim().trim_matches('"').to_string();
        if name.is_empty() || value.is_empty() {
            return None;
        }
        return Some((name, value));
    }

    None
}

/// Parses an implicit state change: `is low`, `is high` (uses last signal).
fn parse_implicit_state_change(line: &str) -> Option<String> {
    let rest = line.strip_prefix("is ")?;
    let value = rest.trim().trim_matches('"').to_string();
    if value.is_empty() {
        return None;
    }
    Some(value)
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

    #[test]
    fn test_parse_quoted_signal_with_alias() {
        let lines = vec!["binary \"Signal A\" as A"];
        let source = parse_timing_source(&lines);
        assert_eq!(source.signals.len(), 1);
        assert!(source.signals.contains_key("A"));
        assert_eq!(source.signals["A"].display_name, "Signal A");
    }

    #[test]
    fn test_parse_clock_with_period() {
        let lines = vec!["clock \"clk\" as C with period 10"];
        let source = parse_timing_source(&lines);
        assert_eq!(source.signals.len(), 1);
        assert!(source.signals.contains_key("C"));
        assert_eq!(source.signals["C"].display_name, "clk");
        assert_eq!(source.signals["C"].period, Some(10.0));
    }

    #[test]
    fn test_parse_time_marker_and_is_syntax() {
        let lines = vec![
            "binary \"Signal A\" as A",
            "@0",
            "A is low",
            "@5",
            "A is high",
            "@10",
            "A is low",
        ];
        let source = parse_timing_source(&lines);
        assert_eq!(source.signals.len(), 1);
        assert_eq!(source.signals["A"].changes.len(), 3);
        assert_eq!(source.signals["A"].changes[0].time, 0.0);
        assert_eq!(source.signals["A"].changes[0].value, "low");
        assert_eq!(source.signals["A"].changes[1].time, 5.0);
        assert_eq!(source.signals["A"].changes[1].value, "high");
        assert_eq!(source.signals["A"].changes[2].time, 10.0);
        assert_eq!(source.signals["A"].changes[2].value, "low");
    }

    #[test]
    fn test_parse_multiple_signals_with_time_markers() {
        let lines = vec![
            "clock \"clk\" as C with period 10",
            "binary \"Data\" as D",
            "@0",
            "D is low",
            "@5",
            "D is high",
            "@15",
            "D is low",
        ];
        let source = parse_timing_source(&lines);
        assert_eq!(source.signals.len(), 2);
        assert_eq!(source.signals["D"].changes.len(), 3);
        assert_eq!(source.signals["D"].changes[0].time, 0.0);
        assert_eq!(source.signals["D"].changes[1].time, 5.0);
        assert_eq!(source.signals["D"].changes[2].time, 15.0);
    }
}

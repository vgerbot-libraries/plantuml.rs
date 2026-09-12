//! Activity diagram source parser.
//!
//! Ported from: `net/sourceforge/plantuml/activitydiagram/ActivityDiagram.java`
//! and `Activity3Command.java`.
//!
//! Parses the new-style activity syntax (`start`, `:action;`, `if/else/endif`,
//! `while/endwhile`, `fork/end fork`).

/// Type of activity node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityNodeType {
    /// Start node (filled circle).
    Start,
    /// Stop node (filled circle inside circle).
    Stop,
    /// Action node (rounded rectangle with label).
    Action,
    /// Decision node (diamond with condition).
    If,
    /// Else branch.
    Else,
    /// End if.
    EndIf,
    /// While loop start.
    While,
    /// End while.
    EndWhile,
    /// Fork (parallel split).
    Fork,
    /// Another fork branch.
    ForkAgain,
    /// End fork.
    EndFork,
    /// Note attached to a node.
    Note,
}

/// An activity node in the flow.
#[derive(Debug, Clone)]
pub struct ActivityNode {
    /// Node type.
    pub node_type: ActivityNodeType,
    /// Label text (for actions, conditions, notes).
    pub label: String,
    /// Indentation level (for nested structures).
    pub level: usize,
}

/// Parsed activity diagram source.
#[derive(Debug, Clone, Default)]
pub struct ActivitySource {
    /// Ordered list of activity nodes.
    pub nodes: Vec<ActivityNode>,
}

/// Parses activity diagram source lines.
#[must_use]
pub fn parse_activity_source(lines: &[&str]) -> ActivitySource {
    let mut source = ActivitySource::default();
    let mut level = 0usize;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // Skip @start/@end directives.
        if trimmed.starts_with("@start") || trimmed.starts_with("@end") {
            continue;
        }

        // Start node.
        if trimmed == "start" {
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::Start,
                label: String::new(),
                level,
            });
            continue;
        }

        // Stop/end node.
        if trimmed == "stop" || trimmed == "end" {
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::Stop,
                label: String::new(),
                level,
            });
            continue;
        }

        // Action: `:label;` or `:label`
        if let Some(rest) = trimmed.strip_prefix(':') {
            let label = rest.trim_end_matches(';').trim().to_string();
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::Action,
                label,
                level,
            });
            continue;
        }

        // If condition: `if (cond) then (yes)` or `if (cond) then`
        if trimmed.starts_with("if ") {
            let label = extract_condition(trimmed);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::If,
                label,
                level,
            });
            level += 1;
            continue;
        }

        // Else: `else (no)` or `else`
        if trimmed.starts_with("else") {
            level = level.saturating_sub(1);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::Else,
                label: extract_paren_content(trimmed).unwrap_or_default(),
                level,
            });
            level += 1;
            continue;
        }

        // EndIf
        if trimmed == "endif" {
            level = level.saturating_sub(1);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::EndIf,
                label: String::new(),
                level,
            });
            continue;
        }

        // While: `while (cond) is (label)`
        if trimmed.starts_with("while ") {
            let label = extract_condition(trimmed);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::While,
                label,
                level,
            });
            level += 1;
            continue;
        }

        // EndWhile: `endwhile (label)` or `endwhile`
        if trimmed.starts_with("endwhile") {
            level = level.saturating_sub(1);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::EndWhile,
                label: extract_paren_content(trimmed).unwrap_or_default(),
                level,
            });
            continue;
        }

        // Fork
        if trimmed == "fork" {
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::Fork,
                label: String::new(),
                level,
            });
            level += 1;
            continue;
        }

        // Fork again
        if trimmed == "fork again" {
            level = level.saturating_sub(1);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::ForkAgain,
                label: String::new(),
                level,
            });
            level += 1;
            continue;
        }

        // End fork
        if trimmed == "end fork" || trimmed == "endfork" {
            level = level.saturating_sub(1);
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::EndFork,
                label: String::new(),
                level,
            });
            continue;
        }

        // Note: `note left : text` or `note right : text`
        if trimmed.starts_with("note ") {
            let label = trimmed.split_once(" : ").map_or("", |(_, t)| t).to_string();
            source.nodes.push(ActivityNode {
                node_type: ActivityNodeType::Note,
                label,
                level,
            });

        }
    }

    source
}

/// Extracts the condition from `if (cond) then` or `while (cond) is`.
fn extract_condition(line: &str) -> String {
    if let Some(start) = line.find('(') {
        if let Some(end) = line[start..].find(')') {
            return line[start + 1..start + end].trim().to_string();
        }
    }
    line.to_string()
}

/// Extracts content from parentheses: `else (no)` → `no`.
fn extract_paren_content(line: &str) -> Option<String> {
    let start = line.find('(')?;
    let end = line[start..].find(')')?;
    Some(line[start + 1..start + end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flow() {
        let lines = vec!["start", ":Do something;", "stop"];
        let source = parse_activity_source(&lines);
        assert_eq!(source.nodes.len(), 3);
        assert_eq!(source.nodes[0].node_type, ActivityNodeType::Start);
        assert_eq!(source.nodes[1].node_type, ActivityNodeType::Action);
        assert_eq!(source.nodes[1].label, "Do something");
        assert_eq!(source.nodes[2].node_type, ActivityNodeType::Stop);
    }

    #[test]
    fn test_parse_if_else() {
        let lines = vec![
            "start",
            "if (condition?) then (yes)",
            ":Do yes;",
            "else (no)",
            ":Do no;",
            "endif",
            "stop",
        ];
        let source = parse_activity_source(&lines);
        assert_eq!(source.nodes.len(), 7);
        assert_eq!(source.nodes[1].node_type, ActivityNodeType::If);
        assert_eq!(source.nodes[1].label, "condition?");
        assert_eq!(source.nodes[3].node_type, ActivityNodeType::Else);
        assert_eq!(source.nodes[6].node_type, ActivityNodeType::Stop);
    }

    #[test]
    fn test_parse_while() {
        let lines = vec![
            "start",
            "while (more data?) is (yes)",
            ":process;",
            "endwhile (no)",
            "stop",
        ];
        let source = parse_activity_source(&lines);
        assert_eq!(source.nodes.len(), 5);
        assert_eq!(source.nodes[1].node_type, ActivityNodeType::While);
        assert_eq!(source.nodes[3].node_type, ActivityNodeType::EndWhile);
    }
}

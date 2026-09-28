//! Activity diagram source parser.
//!
//! Ported from: `net/sourceforge/plantuml/activitydiagram3/ActivityDiagram3.java`
//! together with the `Instruction*` command objects (`InstructionIf`,
//! `InstructionWhile`, `InstructionSimple`, ...).
//!
//! Parses the new-style activity syntax into a recursive tree of
//! [`ActivityBlock`]s: actions, start/stop, `if/else/endif` and
//! `while/endwhile`, each carrying its nested branch/body blocks.

/// One statement in an activity flow.
///
/// Ported from the `Instruction` hierarchy under
/// `net/sourceforge/plantuml/activitydiagram3/`.
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityBlock {
    /// Start node (filled circle). Ported from `InstructionStart`.
    Start,
    /// Stop node (filled circle inside a circle). Ported from `InstructionStop`.
    Stop,
    /// Action (rounded rectangle). Ported from `InstructionSimple`.
    Action(String),
    /// Conditional with an optional else branch.
    ///
    /// Ported from `InstructionIf`; `otherwise` is `None` for an if without
    /// `else`.
    If {
        /// Condition shown inside the decision diamond.
        condition: String,
        /// Label on the then branch.
        then_label: Option<String>,
        /// Statements on the then branch.
        then_block: Vec<Self>,
        /// Label on the else branch, if present.
        else_label: Option<String>,
        /// Statements on the else branch, if present.
        else_block: Option<Vec<Self>>,
    },
    /// While loop.
    ///
    /// Ported from `InstructionWhile`.
    While {
        /// Condition shown inside the decision diamond.
        condition: String,
        /// Label on the looping (back) branch.
        yes_label: Option<String>,
        /// Loop body.
        body: Vec<Self>,
        /// Label on the exit branch.
        out_label: Option<String>,
    },
}

/// Parsed activity diagram: the ordered list of top-level statements.
///
/// Ported from the single `Swimlanes`/instruction list held by
/// `ActivityDiagram3`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActivitySource {
    /// Top-level statements in execution order.
    pub blocks: Vec<ActivityBlock>,
}

/// Parses activity diagram source lines.
///
/// `lines` are the raw diagram lines, typically with the `@startuml`/
/// `@enduml` wrappers already removed.
#[must_use]
pub fn parse_activity_source(lines: &[&str]) -> ActivitySource {
    let cleaned: Vec<&str> = lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('\'')
                && !l.starts_with("@start")
                && !l.starts_with("@end")
        })
        .collect();

    let mut cursor = Cursor { lines: &cleaned, pos: 0 };
    let blocks = parse_sequence(&mut cursor, &[]);
    ActivitySource { blocks }
}

/// Indexed view over the trimmed source lines.
struct Cursor<'a> {
    lines: &'a [&'a str],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Returns the current line without consuming it.
    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.pos).copied()
    }

    /// Consumes and returns the current line.
    fn next(&mut self) -> Option<&'a str> {
        let line = self.lines.get(self.pos).copied();
        if line.is_some() {
            self.pos += 1;
        }
        line
    }
}

/// Parses statements until a terminator keyword (`else`, `endif`, `endwhile`)
/// or end of input is reached.
///
/// Branch/body blocks share this parser; `terminators` lists the keywords that
/// close the current block (they are left unconsumed for the caller).
fn parse_sequence(cursor: &mut Cursor, terminators: &[&str]) -> Vec<ActivityBlock> {
    let mut blocks = Vec::new();
    while let Some(line) = cursor.peek() {
        if terminators.iter().any(|t| line.starts_with(t)) {
            break;
        }
        cursor.next();

        if line == "start" {
            blocks.push(ActivityBlock::Start);
        } else if line == "stop" || line == "end" {
            blocks.push(ActivityBlock::Stop);
        } else if let Some(rest) = line.strip_prefix(':') {
            let label = rest.trim_end_matches(';').trim().to_string();
            blocks.push(ActivityBlock::Action(label));
        } else if line.starts_with("if ") {
            blocks.push(parse_if(cursor, line));
        } else if line.starts_with("while ") {
            blocks.push(parse_while(cursor, line));
        }
    }
    blocks
}

/// Parses an `if` statement whose header line has already been consumed.
fn parse_if(cursor: &mut Cursor, header: &str) -> ActivityBlock {
    let condition = first_paren(header).unwrap_or_else(|| header.to_string());
    let then_label = second_paren(header);
    let then_block = parse_sequence(cursor, &["else", "endif"]);

    let mut else_label = None;
    let mut else_block = None;
    if let Some(else_line) = cursor.peek() {
        if else_line.starts_with("else") {
            cursor.next();
            else_label = first_paren(else_line);
            else_block = Some(parse_sequence(cursor, &["endif"]));
        }
    }
    // Consume the closing `endif`.
    if cursor.peek().is_some_and(|l| l == "endif") {
        cursor.next();
    }

    ActivityBlock::If {
        condition,
        then_label,
        then_block,
        else_label,
        else_block,
    }
}

/// Parses a `while` statement whose header line has already been consumed.
fn parse_while(cursor: &mut Cursor, header: &str) -> ActivityBlock {
    let condition = first_paren(header).unwrap_or_else(|| header.to_string());
    let yes_label = second_paren(header);
    let body = parse_sequence(cursor, &["endwhile"]);

    let mut out_label = None;
    if let Some(end_line) = cursor.next() {
        if end_line.starts_with("endwhile") {
            out_label = first_paren(end_line);
        }
    }

    ActivityBlock::While {
        condition,
        yes_label,
        body,
        out_label,
    }
}

/// Returns the content of the first parenthesised group: `if (c) then` → `c`.
fn first_paren(line: &str) -> Option<String> {
    paren_group(line, 0)
}

/// Returns the content of the second parenthesised group:
/// `if (c) then (yes)` → `yes`.
fn second_paren(line: &str) -> Option<String> {
    let first_close = line.find(')')?;
    paren_group(&line[first_close..], 0)
}

/// Finds the first `( ... )` in `s` and returns its trimmed content.
fn paren_group(s: &str, _from: usize) -> Option<String> {
    let start = s.find('(')? + 1;
    let end = s[start..].find(')')? + start;
    Some(s[start..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flow() {
        let source = parse_activity_source(&["start", ":Do something;", "stop"]);
        assert_eq!(
            source.blocks,
            vec![
                ActivityBlock::Start,
                ActivityBlock::Action("Do something".to_string()),
                ActivityBlock::Stop,
            ]
        );
    }

    #[test]
    fn test_parse_if_else() {
        let source = parse_activity_source(&[
            "start",
            "if (condition?) then (yes)",
            ":Take yes path;",
            "else (no)",
            ":Take no path;",
            "endif",
            "stop",
        ]);
        assert_eq!(source.blocks.len(), 3);
        match &source.blocks[1] {
            ActivityBlock::If {
                condition,
                then_label,
                then_block,
                else_label,
                else_block,
            } => {
                assert_eq!(condition, "condition?");
                assert_eq!(then_label.as_deref(), Some("yes"));
                assert_eq!(
                    then_block,
                    &[ActivityBlock::Action("Take yes path".to_string())]
                );
                assert_eq!(else_label.as_deref(), Some("no"));
                assert_eq!(
                    else_block.as_deref(),
                    Some(&[ActivityBlock::Action("Take no path".to_string())][..])
                );
            }
            other => panic!("expected if, got {other:?}"),
        }
        assert_eq!(source.blocks[2], ActivityBlock::Stop);
    }

    #[test]
    fn test_parse_while() {
        let source = parse_activity_source(&[
            "start",
            "while (more data?) is (yes)",
            ":Process item;",
            "endwhile (no)",
            "stop",
        ]);
        assert_eq!(source.blocks.len(), 3);
        match &source.blocks[1] {
            ActivityBlock::While {
                condition,
                yes_label,
                body,
                out_label,
            } => {
                assert_eq!(condition, "more data?");
                assert_eq!(yes_label.as_deref(), Some("yes"));
                assert_eq!(body, &[ActivityBlock::Action("Process item".to_string())]);
                assert_eq!(out_label.as_deref(), Some("no"));
            }
            other => panic!("expected while, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_nested_if_in_while() {
        let source = parse_activity_source(&[
            "while (c?) is (y)",
            "if (x?) then (a)",
            ":one;",
            "else (b)",
            ":two;",
            "endif",
            "endwhile (n)",
        ]);
        match &source.blocks[0] {
            ActivityBlock::While { body, .. } => assert_eq!(body.len(), 1),
            other => panic!("expected while, got {other:?}"),
        }
    }
}

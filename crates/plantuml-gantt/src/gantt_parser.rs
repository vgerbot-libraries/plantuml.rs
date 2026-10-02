//! Gantt diagram source parser.
//!
//! Ported from the command layer of
//! `net/sourceforge/plantuml/gantt/` (see `gantt/command/`,
//! `GanttTaskTable`, `TaskImpl`, `GanttDiagram`).
//!
//! Time is tracked as an integer day index relative to the diagram
//! project start, exactly like Java's model: every task/milestone start
//! and end lands on a whole day.


/// A Gantt task or milestone.
#[derive(Debug, Clone)]
pub struct GanttTask {
    /// Task name (display label).
    pub name: String,
    /// Zero-based day index of the start.
    pub start: i64,
    /// Duration in whole days (0 for milestones).
    pub duration: i64,
    /// Whether this is a milestone (duration = 0, rendered as a diamond).
    pub is_milestone: bool,
}

/// A dependency between two tasks: `source` must finish before `dest`.
#[derive(Debug, Clone)]
pub struct TaskDependency {
    /// Name of the predecessor task.
    pub from: String,
    /// Name of the successor task.
    pub to: String,
}

/// Parsed Gantt diagram source.
#[derive(Debug, Clone, Default)]
pub struct GanttSource {
    /// Tasks in declaration order.
    pub tasks: Vec<GanttTask>,
    /// Dependencies in declaration order.
    pub dependencies: Vec<TaskDependency>,
    /// Number of days between the project start and the 1st displayed day.
    pub project_offset: i64,
}

impl GanttSource {
    /// Looks up a task by its display name.
    #[must_use]
    pub fn task_by_name(&self, name: &str) -> Option<&GanttTask> {
        self.tasks.iter().find(|t| t.name == name)
    }
}
/// Parses Gantt diagram source lines.
#[must_use]
pub fn parse_gantt_source(lines: &[&str]) -> GanttSource {
    let mut source = GanttSource::default();
    let mut project_started = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('\'')
            || trimmed.starts_with("@start")
            || trimmed.starts_with("@end")
        {
            continue;
        }

        let lower = trimmed.to_lowercase();
        if lower.starts_with("project starts") {
            if let Some(offset) = parse_project_start(trimmed) {
                source.project_offset = offset;
                project_started = true;
            }
            continue;
        }

        // `[Name] lasts N days` / `task [Name] lasts N days`.
        // Without an explicit date the task starts on day 0 (project start).
        if let Some((name, duration)) = parse_last_task(trimmed) {
            source.tasks.push(GanttTask {
                name,
                start: 0,
                duration,
                is_milestone: false,
            });
            continue;
        }

        // `[Name] happens <date>` / `milestone [Name] happens <date>`
        if let Some((name, when)) = parse_happens(trimmed) {
            let start = if project_started {
                when - source.project_offset
            } else {
                when
            };
            source.tasks.push(GanttTask {
                name,
                start,
                duration: 0,
                is_milestone: true,
            });
            continue;
        }

        // `[B] starts at [A]'s end`
        if let Some((to, from)) = parse_starts_at_end(trimmed) {
            source.dependencies.push(TaskDependency { from, to });
        }
    }

    // Exact-start dependencies move the successor to the predecessor end.
    for dep in &source.dependencies {
        let end = source
            .task_by_name(&dep.from)
            .map_or(0, |t| t.start + t.duration);
        if let Some(to_task) = source.tasks.iter_mut().find(|t| t.name == dep.to) {
            if to_task.start < end {
                to_task.start = end;
            }
        }
    }

    source
}

/// Extracts the first `[...]` token and returns the inner text.
fn bracket_token(s: &str) -> Option<String> {
    let start = s.find('[')? + 1;
    let end = s[start..].find(']')? + start;
    Some(s[start..end].trim().to_string())
}

/// Parses `<name> lasts N days` (with optional leading `task `).
/// Returns `(name, duration)` or `None`.
fn parse_last_task(line: &str) -> Option<(String, i64)> {
    let rest = line.strip_prefix("task ").unwrap_or(line);
    let idx = rest.find(" lasts ")?;
    let name = bracket_token(&rest[..idx])?;
    let duration = parse_days(rest[idx + " lasts ".len()..].trim())?;
    Some((name, duration))
}

/// Parses `<name> happens <time expression>` (with optional leading `milestone `).
/// Returns `(name, absolute day index)`.
fn parse_happens(line: &str) -> Option<(String, i64)> {
    let rest = line.strip_prefix("milestone ").unwrap_or(line);
    let idx = rest.find(" happens ")?;
    let name = bracket_token(&rest[..idx])?;
    let when = parse_day_anchor(rest[idx + " happens ".len()..].trim())?;
    Some((name, when))
}

/// Parses `[B] starts at [A]'s end` → `(B, A)`.
fn parse_starts_at_end(line: &str) -> Option<(String, String)> {
    let lower = line.to_lowercase();
    if !lower.contains(" starts at ") || !lower.contains("'s end") {
        return None;
    }
    let mut iter = line.match_indices('[');
    let b_start = iter.next()?.0 + 1;
    let b_end = line[b_start..].find(']')? + b_start;
    let a_start = iter.next()?.0 + 1;
    let a_end = line[a_start..].find(']')? + a_start;
    Some((
        line[b_start..b_end].trim().to_string(),
        line[a_start..a_end].trim().to_string(),
    ))
}

/// Parses a project-start line and returns the day index of the start date.
fn parse_project_start(line: &str) -> Option<i64> {
    let s = line
        .trim_start_matches(|c: char| c.is_alphabetic() || c.is_whitespace())
        .trim();
    // After removing the leading "Project starts" words.
    let s = line.split_once("starts").map_or(s, |(_, r)| r.trim());
    parse_day_anchor(s)
}

/// Parses an anchor expression into an absolute day index.
///
/// Supports ISO dates (`2020-01-11`) and natural language
/// (`the 1st of january 2020`).
fn parse_day_anchor(s: &str) -> Option<i64> {
    if let Some(date) = parse_iso_date(s) {
        return Some(date);
    }
    parse_natural_date(s)
}

/// Parses a natural language date like "the 1st of january 2020".
fn parse_natural_date(s: &str) -> Option<i64> {
    let day = s.split_whitespace().find_map(parse_ordinal_day)?;
    let month = s.split_whitespace().find_map(parse_month_name)?;
    let year = s
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| p.len() == 4)
        .find_map(|p| p.parse::<i32>().ok())?;
    Some(date_to_day_index(year, month, day))
}

/// Parses an ordinal day number: "1st", "2nd", "3rd", "15th", etc.
fn parse_ordinal_day(s: &str) -> Option<u32> {
    let digits = s
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    let n: u32 = digits.parse().ok()?;
    if (1..=31).contains(&n) {
        Some(n)
    } else {
        None
    }
}

/// Parses a month name (case-insensitive) to 1-12.
fn parse_month_name(s: &str) -> Option<u32> {
    const MONTHS: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let lower = s.to_lowercase();
    MONTHS
        .iter()
        .position(|m| lower.starts_with(m))
        .map(|i| u32::try_from(i).unwrap_or_default() + 1)
}

/// Converts a calendar date into a day index (1970-01-01 = 0).
#[must_use]
pub fn date_to_day_index(year: i32, month: u32, day: u32) -> i64 {
    let mut total: i64 = 0;
    for y in 1970..year {
        total += if is_leap_year(y) { 366 } else { 365 };
    }
    const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        total += i64::from(DAYS[m as usize - 1]);
        if m == 2 && is_leap_year(year) {
            total += 1;
        }
    }
    total + i64::from(day) - 1
}

/// Returns true if the year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Parses an ISO date: "2020-01-11" into a day index.
fn parse_iso_date(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s
        .split(|c: char| c == '-' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(date_to_day_index(year, month, day))
}

/// Parses a duration like `5 days`, `3 weeks`, `1 month`.
fn parse_days(s: &str) -> Option<i64> {
    let mut words = s.split_whitespace();
    let n: i64 = words.next()?.parse().ok()?;
    let unit = words.next()?.to_lowercase();
    let factor = if unit.starts_with("day") {
        1
    } else if unit.starts_with("week") {
        7
    } else if unit.starts_with("month") {
        30
    } else {
        return None;
    };
    Some(n * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_tasks() {
        let source = parse_gantt_source(&[
            "Project starts the 1st of january 2020",
            "[Task A] lasts 5 days",
            "[Task B] lasts 3 days",
        ]);
        assert_eq!(source.tasks.len(), 2);
        assert_eq!(source.tasks[0].start, 0);
        assert_eq!(source.tasks[0].duration, 5);
        assert_eq!(source.tasks[1].start, 0);
        assert_eq!(source.tasks[1].duration, 3);
        assert_eq!(source.project_offset, date_to_day_index(2020, 1, 1));
    }

    #[test]
    fn parses_start_at_end_dependency() {
        let source = parse_gantt_source(&[
            "Project starts the 1st of january 2020",
            "[Task A] lasts 5 days",
            "[Task B] lasts 3 days",
            "[Task B] starts at [Task A]'s end",
        ]);
        assert_eq!(source.tasks[1].start, 5);
    }

    #[test]
    fn parses_iso_milestone() {
        let source = parse_gantt_source(&[
            "Project starts the 1st of january 2020",
            "[M1] happens 2020-01-11",
        ]);
        assert!(source.tasks[1].is_milestone);
        assert_eq!(source.tasks[1].start, 10);
    }
}

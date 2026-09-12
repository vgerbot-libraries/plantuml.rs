//! Gantt diagram source parser.
//!
//! Ported from: `net/sourceforge/plantuml/ganttdiagram/GanttDiagram.java`.

use indexmap::IndexMap;

/// A Gantt task.
#[derive(Debug, Clone)]
pub struct GanttTask {
    /// Task name (display label).
    pub name: String,
    /// Start day (0-based).
    pub start: f64,
    /// Duration in days.
    pub duration: f64,
    /// Optional color.
    pub color: Option<String>,
    /// Whether this is a milestone (duration = 0).
    pub is_milestone: bool,
}

/// A dependency between tasks.
#[derive(Debug, Clone)]
pub struct TaskDependency {
    /// Source task name.
    pub from: String,
    /// Target task name.
    pub to: String,
}

/// Parsed Gantt diagram source.
#[derive(Debug, Clone, Default)]
pub struct GanttSource {
    /// Tasks keyed by name.
    pub tasks: IndexMap<String, GanttTask>,
    /// Dependencies.
    pub dependencies: Vec<TaskDependency>,
    /// Optional title.
    pub title: Option<String>,
    /// Project start day (default 0).
    pub project_start: f64,
}

/// Parses Gantt diagram source lines.
#[must_use]
pub fn parse_gantt_source(lines: &[&str]) -> GanttSource {
    let mut source = GanttSource::default();
    let mut current_day = 0.0;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // Skip @start/@end directives.
        if trimmed.starts_with("@start") || trimmed.starts_with("@end") {
            continue;
        }

        // Parse title.
        if let Some(rest) = trimmed.strip_prefix("title ") {
            source.title = Some(rest.trim_matches('"').to_string());
            continue;
        }

        // Parse project start: `Project starts the 1st of january 2020` or `project starts N`.
        let lower = trimmed.to_lowercase();
        if lower.starts_with("project starts") {
            if let Some(days) = parse_project_start(trimmed) {
                source.project_start = days;
                current_day = days;
            }
            continue;
        }

        // Parse task declaration: `[Task1] lasts 5 days` or `task [Task1] lasts 5 days`.
        if let Some(task) = parse_task_line(trimmed, current_day) {
            current_day = task.start + task.duration;
            source.tasks.insert(task.name.clone(), task);
            continue;
        }

        // Parse milestone: `milestone [M1] happens at 5 days` or `[M1] happens at 5 days`.
        if let Some(milestone) = parse_milestone_line(trimmed) {
            source.tasks.insert(milestone.name.clone(), milestone);
            continue;
        }

        // Parse dependency: `[Task1] depends on [Task2]`
        if let Some(dep) = parse_dependency_line(trimmed) {
            source.dependencies.push(dep);
        }
    }

    // Adjust task starts based on dependencies.
    for dep in &source.dependencies {
        let dep_end = source.tasks.get(&dep.from).map_or(0.0, |t| t.start + t.duration);
        if let Some(to_task) = source.tasks.get_mut(&dep.to) {
            if to_task.start < dep_end {
                to_task.start = dep_end;
            }
        }
    }

    source
}

/// Parses a task line: `[Name] lasts N days` or `task [Name] lasts N days`.
fn parse_task_line(line: &str, current_day: f64) -> Option<GanttTask> {
    // Strip optional `task ` prefix.
    let rest = line.strip_prefix("task ").unwrap_or(line);

    let (name_part, duration_part) = rest.split_once(" lasts ")?;
    let name = extract_bracket_name(name_part);
    let duration = parse_days(duration_part.trim())?;
    Some(GanttTask {
        name,
        start: current_day,
        duration,
        color: None,
        is_milestone: false,
    })
}

/// Parses a milestone line: `milestone [Name] happens at N days` or `[Name] happens at N days`.
fn parse_milestone_line(line: &str) -> Option<GanttTask> {
    // Strip optional `milestone ` prefix.
    let rest = line.strip_prefix("milestone ").unwrap_or(line);

    let (name_part, time_part) = rest.split_once(" happens at ")?;
    let name = extract_bracket_name(name_part);
    let time = parse_days(time_part.trim())?;
    Some(GanttTask {
        name,
        start: time,
        duration: 0.0,
        color: None,
        is_milestone: true,
    })
}

/// Parses a dependency line: `[Task1] depends on [Task2]`.
fn parse_dependency_line(line: &str) -> Option<TaskDependency> {
    let (from, rest) = line.split_once(" depends on ")?;
    let from = extract_bracket_name(from);
    let to = extract_bracket_name(rest);
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some(TaskDependency { from, to })
}

/// Extracts the name from `[Name]`, trimming brackets and whitespace.
fn extract_bracket_name(s: &str) -> String {
    s.trim().trim_matches('[').trim_matches(']').trim().to_string()
}

/// Parses a project start line.
///
/// Supports:
/// - `Project starts the 1st of january 2020` (natural language date)
/// - `Project starts 2020-07-01` (ISO date)
/// - `Project starts N` (numeric day offset)
fn parse_project_start(line: &str) -> Option<f64> {
    let rest = line.trim();
    // Case-insensitive strip "project starts".
    let rest = if rest.to_lowercase().starts_with("project starts") {
        rest["project starts".len()..].trim()
    } else {
        return None;
    };

    // Try natural language: "the 1st of january 2020"
    if let Some(day) = parse_natural_date(rest) {
        return Some(day);
    }

    // Try ISO date: "2020-07-01"
    if let Some(day) = parse_iso_date(rest) {
        return Some(day);
    }

    // Try numeric: "5" or "5 days"
    parse_days(rest).or_else(|| rest.trim().parse::<f64>().ok())
}

/// Parses a natural language date like "the 1st of january 2020".
/// Returns a day offset from January 1, 2000 (epoch-like base).
fn parse_natural_date(s: &str) -> Option<f64> {
    // Pattern: the Nth of Month YYYY
    let s = s.trim();
    let s = s.strip_prefix("the ").unwrap_or(s);

    // Split into day part, "of", month year.
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    // Expected: ["1st", "of", "january", "2020"]
    if parts[1].to_lowercase() != "of" {
        return None;
    }

    let day = parse_ordinal_day(parts[0])?;
    let month = parse_month_name(parts[2])?;
    let year: i32 = parts[3].parse().ok()?;

    // Convert to day offset from year 2000.
    Some(date_to_day_offset(year, month, day) as f64)
}

/// Parses an ordinal day number: "1st", "2nd", "3rd", "4th", "15th", etc.
fn parse_ordinal_day(s: &str) -> Option<u32> {
    let s = s.trim();
    // Strip ordinal suffix.
    let digits = s
        .trim_end_matches("st")
        .trim_end_matches("nd")
        .trim_end_matches("rd")
        .trim_end_matches("th");
    digits.parse().ok()
}

/// Parses a month name (case-insensitive) to 1-12.
fn parse_month_name(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "january" | "jan" => Some(1),
        "february" | "feb" => Some(2),
        "march" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "may" => Some(5),
        "june" | "jun" => Some(6),
        "july" | "jul" => Some(7),
        "august" | "aug" => Some(8),
        "september" | "sep" | "sept" => Some(9),
        "october" | "oct" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}

/// Converts a year/month/day to a day offset from 2000-01-01.
fn date_to_day_offset(year: i32, month: u32, day: u32) -> i64 {
    // Simple cumulative day calculation from 2000-01-01.
    let mut total: i64 = 0;

    // Add days for full years.
    for y in 2000..year {
        total += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for full months in current year.
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        total += days_in_month[(m - 1) as usize];
        if m == 2 && is_leap_year(year) {
            total += 1;
        }
    }

    // Add days in current month (day - 1 since offset starts at 1st).
    total += (day as i64) - 1;

    total
}

/// Returns true if the year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Parses an ISO date: "2020-07-01".
fn parse_iso_date(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    Some(date_to_day_offset(year, month, day) as f64)
}

/// Parses a duration like `5 days`, `3 weeks`, `1 month`.
fn parse_days(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(rest) = s.strip_suffix("days") {
        rest.trim().parse::<f64>().ok()
    } else if let Some(rest) = s.strip_suffix("day") {
        rest.trim().parse::<f64>().ok()
    } else if let Some(rest) = s.strip_suffix("weeks") {
        rest.trim().parse::<f64>().ok().map(|w| w * 5.0)
    } else if let Some(rest) = s.strip_suffix("week") {
        rest.trim().parse::<f64>().ok().map(|w| w * 5.0)
    } else if let Some(rest) = s.strip_suffix("months") {
        rest.trim().parse::<f64>().ok().map(|m| m * 20.0)
    } else if let Some(rest) = s.strip_suffix("month") {
        rest.trim().parse::<f64>().ok().map(|m| m * 20.0)
    } else {
        s.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tasks() {
        let lines = vec![
            "task [Task1] lasts 5 days",
            "task [Task2] lasts 3 days",
        ];
        let source = parse_gantt_source(&lines);
        assert_eq!(source.tasks.len(), 2);
        assert_eq!(source.tasks["Task1"].duration, 5.0);
        assert_eq!(source.tasks["Task2"].start, 5.0); // starts after Task1
    }

    #[test]
    fn test_parse_milestone() {
        let lines = vec!["milestone [M1] happens at 10 days"];
        let source = parse_gantt_source(&lines);
        assert_eq!(source.tasks.len(), 1);
        assert!(source.tasks["M1"].is_milestone);
        assert_eq!(source.tasks["M1"].start, 10.0);
    }

    #[test]
    fn test_parse_dependency() {
        let lines = vec![
            "task [Task1] lasts 5 days",
            "task [Task2] lasts 3 days",
            "[Task2] depends on [Task1]",
        ];
        let source = parse_gantt_source(&lines);
        assert_eq!(source.dependencies.len(), 1);
        // Task2 should start after Task1 ends (day 5)
        assert_eq!(source.tasks["Task2"].start, 5.0);
    }

    #[test]
    fn test_parse_task_without_prefix() {
        let lines = vec![
            "Project starts the 1st of january 2020",
            "[Task A] lasts 5 days",
            "[Task B] lasts 3 days",
        ];
        let source = parse_gantt_source(&lines);
        assert_eq!(source.tasks.len(), 2);
        assert_eq!(source.tasks["Task A"].duration, 5.0);
        assert_eq!(source.tasks["Task B"].start, source.tasks["Task A"].start + 5.0);
    }

    #[test]
    fn test_parse_project_start_natural_date() {
        let lines = vec![
            "Project starts the 1st of january 2020",
            "[Task A] lasts 5 days",
        ];
        let source = parse_gantt_source(&lines);
        // Jan 1 2020 should be a non-zero day offset from 2000-01-01.
        assert!(source.project_start > 0.0);
    }

    #[test]
    fn test_parse_project_start_iso_date() {
        let lines = vec!["Project starts 2020-07-01", "[Task A] lasts 5 days"];
        let source = parse_gantt_source(&lines);
        assert!(source.project_start > 0.0);
    }

    #[test]
    fn test_parse_milestone_without_prefix() {
        let lines = vec!["[M1] happens at 10 days"];
        let source = parse_gantt_source(&lines);
        assert_eq!(source.tasks.len(), 1);
        assert!(source.tasks["M1"].is_milestone);
        assert_eq!(source.tasks["M1"].start, 10.0);
    }

    #[test]
    fn test_parse_dependency_without_prefix() {
        let lines = vec![
            "[Task A] lasts 5 days",
            "[Task B] lasts 3 days",
            "[Task B] depends on [Task A]",
        ];
        let source = parse_gantt_source(&lines);
        assert_eq!(source.dependencies.len(), 1);
        assert_eq!(source.tasks["Task B"].start, 5.0);
    }
}

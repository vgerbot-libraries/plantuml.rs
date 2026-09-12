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

        // Parse project start.
        if let Some(rest) = trimmed.strip_prefix("project starts") {
            if let Some(days) = parse_days(rest) {
                source.project_start = days;
                current_day = days;
            }
            continue;
        }

        // Parse task declaration: `task [Task1] lasts 5 days`
        if let Some(task) = parse_task_line(trimmed, current_day) {
            current_day = task.start + task.duration;
            source.tasks.insert(task.name.clone(), task);
            continue;
        }

        // Parse milestone: `milestone [M1] happens at 5 days`
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

/// Parses a task line: `task [Name] lasts N days`.
fn parse_task_line(line: &str, current_day: f64) -> Option<GanttTask> {
    let rest = line.strip_prefix("task ")?;
    let (name_part, duration_part) = rest.split_once(" lasts ")?;
    let name = name_part.trim().trim_matches('[').trim_matches(']').trim().to_string();
    let duration = parse_days(duration_part.trim())?;
    Some(GanttTask {
        name,
        start: current_day,
        duration,
        color: None,
        is_milestone: false,
    })
}

/// Parses a milestone line: `milestone [Name] happens at N days`.
fn parse_milestone_line(line: &str) -> Option<GanttTask> {
    let rest = line.strip_prefix("milestone ")?;
    let (name_part, time_part) = rest.split_once(" happens at ")?;
    let name = name_part.trim().trim_matches('[').trim_matches(']').trim().to_string();
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
    let from = from.trim().trim_matches('[').trim_matches(']').trim().to_string();
    let to = rest.trim().trim_matches('[').trim_matches(']').trim().to_string();
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some(TaskDependency { from, to })
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
}

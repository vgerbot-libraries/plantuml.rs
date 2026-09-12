//! Gantt diagram for plantuml.rs.
//!
//! Ported from: `net/sourceforge/plantuml/ganttdiagram/` (19 files).
//!
//! Gantt diagrams render project tasks as horizontal bars on a timeline.

pub mod gantt_diagram;
pub mod gantt_parser;
pub mod gantt_renderer;

pub use gantt_diagram::GanttDiagram;
pub use gantt_parser::{parse_gantt_source, GanttTask, GanttSource, TaskDependency};

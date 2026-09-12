//! Activity diagram for plantuml.rs.
//!
//! Ported from: `net/sourceforge/plantuml/activitydiagram/` (100+ files).
//!
//! Activity diagrams render flowchart-like processes with actions,
//! decisions, loops, and forks.

pub mod activity_diagram;
pub mod activity_parser;
pub mod activity_renderer;

pub use activity_diagram::ActivityDiagram;
pub use activity_parser::{parse_activity_source, ActivityNode, ActivityNodeType, ActivitySource};

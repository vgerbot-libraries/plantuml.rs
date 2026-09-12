//! Mindmap and WBS diagram support for plantuml.rs.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/mindmap/` — Mindmap diagram (19 files)
//! - `net/sourceforge/plantuml/wbs/` — WBS diagram (14 files)
//!
//! Both are tree-based diagrams with org-mode `*` prefix syntax.
//! Mindmap uses left-right branching with curved connectors.
//! WBS uses top-down layout with orthogonal connectors.

pub mod idea;
pub mod mindmap_diagram;
pub mod mindmap_renderer;
pub mod wbs_diagram;
pub mod wbs_element;
pub mod wbs_renderer;

pub use mindmap_diagram::MindMapDiagram;
pub use wbs_diagram::WbsDiagram;

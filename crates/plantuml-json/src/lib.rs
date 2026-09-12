//! JSON and YAML diagram rendering for plantuml.rs.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/jsondiagram/` — JSON diagram rendering
//! - `net/sourceforge/plantuml/yaml/` — YAML diagram (converts to JSON)
//!
//! Uses `serde_json` (with `preserve_order`) instead of porting the Java
//! JSON parser, and `serde_yaml` for YAML parsing.
//!
//! The `JsonDiagram` implements `plantuml_core::Diagram`. Factory logic
//! (parsing `@startjson`/`@startyaml` blocks) lives in `plantuml-engine`
//! to avoid circular dependencies.

pub mod json_diagram;
pub mod json_renderer;
pub mod yaml_support;
pub use yaml_support::{parse_yaml_to_json, parse_json};
pub use json_diagram::JsonDiagram;
pub use json_renderer::{render_json_svg, Highlight};

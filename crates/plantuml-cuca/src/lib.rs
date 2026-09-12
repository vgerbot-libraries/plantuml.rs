//! CucaDiagram foundation for plantuml.rs.
//!
//! Ported from:
//! - `net/atmp/CucaDiagram.java` — base diagram with entity/link management
//! - `net/sourceforge/plantuml/cucadiagram/` — body rendering (28 files)
//! - `net/sourceforge/plantuml/sdot/` — Smetana layout engine (10 files)
//! - `net/sourceforge/plantuml/command/` — shared commands
//!
//! This crate provides the shared infrastructure for Class, Object, State,
//! and Description (Component/Deployment/UseCase) diagrams.
//!
//! The Rust implementation uses a simplified layout algorithm instead of
//! the full Smetana graph layout engine.

pub mod cuca_diagram;
pub mod cuca_layout;
pub mod cuca_renderer;
pub mod entity_link_parser;

pub use cuca_diagram::CucaDiagram;
pub use cuca_layout::LayoutNode;
pub use entity_link_parser::{ParsedEntity, ParsedLink, parse_entity_link_source};

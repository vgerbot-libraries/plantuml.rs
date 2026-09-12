//! `TitledDiagram` — shared base for all non-sequence diagram types.
//!
//! Ported from: `net/sourceforge/plantuml/TitledDiagram.java`
//!
//! Provides common state and operations used by all diagram types that
//! have a title, caption, legend, header, footer, and skin parameters.
//! Concrete diagram types embed a `TitledDiagram` via composition and
//! delegate to it for shared functionality.

pub mod titled_diagram;

pub use titled_diagram::TitledDiagram;

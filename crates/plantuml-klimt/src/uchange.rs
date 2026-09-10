//! `UChange` — marker trait for drawing state changes.
//!
//! Ported from: net/sourceforge/plantuml/klimt/drawing/UChange.java
//!
//! Java has `UChangeColor`, `UChangeStroke`, `UChangeFont`, `UTranslate`,
//! `UClip`, etc. They all implement the empty `UChange` marker. Concrete
//! changes are ported as needed by backends.

/// Marker trait for a drawing-state change applied to a `UGraphic`.
pub trait UChange: std::fmt::Debug {}

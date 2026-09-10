//! `UShape` — marker trait for drawable shapes.
//!
//! Ported from: net/sourceforge/plantuml/klimt/shape/UShape.java
//!
//! In Java `UShape` is an empty marker interface implemented by `URectangle`,
//! `UEllipse`, `ULine`, `UPolygon`, `UPath`, `UText`, `UImage`, etc. In Rust
//! we keep the same marker-trait pattern; concrete shapes are ported with
//! their drivers in later phases.

/// Marker trait for any drawable shape.
pub trait UShape: std::fmt::Debug + std::any::Any {
    /// Returns this shape as a `&dyn Any` for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Macro to implement `UShape` with `as_any` for a concrete shape type.
#[macro_export]
macro_rules! impl_ushape {
    ($t:ty) => {
        impl $crate::ushape::UShape for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

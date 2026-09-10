//! PlantUML entity model — Entity, Link, and related types.
//!
//! Ported from: `net/sourceforge/plantuml/abel/` and `net/sourceforge/plantuml/cucadiagram/` packages.

pub mod bodier;
pub mod entity;
pub mod entity_factory;
pub mod group_type;
pub mod leaf_type;
pub mod link;

pub use bodier::Bodier;
pub use entity::Entity;
pub use entity_factory::EntityFactory;
pub use group_type::GroupType;
pub use leaf_type::LeafType;
pub use link::Link;

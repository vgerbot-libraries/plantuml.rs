//! `PlantUML` entity model — Entity, Link, and related types.
//!
//! Ported from `net/sourceforge/plantuml/abel/` and `net/sourceforge/plantuml/cucadiagram/` packages.

pub mod bodier;
pub mod cuca_note;
pub mod display_positioned;
pub mod entity;
pub mod entity_factory;
pub mod group_type;
pub mod leaf_type;
pub mod link;
pub mod link_arg;
pub mod link_arrow;
pub mod position;
pub mod tip;
pub mod together;

pub use bodier::Bodier;
pub use cuca_note::{CucaNote, NoteLinkStrategy};
pub use display_positioned::DisplayPositioned;
pub use entity::Entity;
pub use entity_factory::EntityFactory;
pub use group_type::GroupType;
pub use leaf_type::LeafType;
pub use link::Link;
pub use link_arg::LinkArg;
pub use link_arrow::LinkArrow;
pub use position::Position;
pub use tip::Tip;
pub use together::Together;

// Re-export Display from klimt (canonical implementation).
pub use plantuml_klimt::Display;

//! Bodier trait — body management for entities.
//!
//! Ported from: `net/sourceforge/plantuml/cucadiagram/Bodier.java`

use crate::entity::Entity;

/// Trait for managing the body of an entity (methods, fields, etc.).
///
/// The body is the content inside a class-like entity — methods, fields,
/// and other members. Different body types exist for different entity
/// types (simple, class-like, map, JSON, etc.).
///
/// Ported from: `net/sourceforge/plantuml/cucadiagram/Bodier.java`
pub trait Bodier {
    /// Returns the entity that owns this body.
    ///
    /// Ported from: `Bodier.getEntity()`.
    fn get_entity(&self) -> &Entity;

    /// Sets the entity that owns this body.
    ///
    /// Ported from: `Bodier.setEntity(Entity)`.
    fn set_entity(&mut self, entity: Entity);

    /// Returns `true` if the body has methods or fields.
    ///
    /// Ported from: `Bodier.hasMethodsOrFields()`.
    fn has_methods_or_fields(&self) -> bool;

    /// Returns `true` if the body is empty.
    ///
    /// Ported from: `Bodier.isEmpty()`.
    fn is_empty(&self) -> bool {
        !self.has_methods_or_fields()
    }
}

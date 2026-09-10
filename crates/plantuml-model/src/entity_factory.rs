//! Entity factory trait — interface for creating entities.
//!
//! Ported from: `net/sourceforge/plantuml/abel/EntityFactory.java`

use crate::entity::{Entity, Quark};
use crate::group_type::GroupType;
use crate::leaf_type::LeafType;

/// Factory for creating entities in a diagram.
///
/// Ported from: `net/sourceforge/plantuml/abel/EntityFactory.java`
pub trait EntityFactory {
    /// Creates a new leaf entity with the given name and type.
    ///
    /// Ported from: `EntityFactory.createLeaf(Quark, LeafType)`.
    fn create_leaf(&mut self, quark: Quark, leaf_type: LeafType) -> Entity;

    /// Creates a new group entity with the given name and type.
    ///
    /// Ported from: `EntityFactory.createGroup(Quark, GroupType)`.
    fn create_group(&mut self, quark: Quark, group_type: GroupType) -> Entity;

    /// Creates a new entity from a code string.
    ///
    /// Ported from: `EntityFactory.createEntityWithCodeString(String, String)`.
    fn create_entity_with_code_string(
        &mut self,
        code: &str,
        display: &str,
    ) -> Entity;

    /// Returns `true` if an entity with the given UID exists.
    ///
    /// Ported from: `EntityFactory.isEntityWithCodeString(String)`.
    fn is_entity_with_code_string(&self, code: &str) -> bool;

    /// Gets an entity by code string.
    ///
    /// Ported from: `EntityFactory.getEntityWithCodeString(String)`.
    fn get_entity_with_code_string(&self, code: &str) -> Option<&Entity>;
}

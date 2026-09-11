//! Link — relationship between two entities.
//!
//! Ported from: `net/sourceforge/plantuml/abel/Link.java`

use crate::entity::Entity;
use std::sync::Arc;

/// Placeholder for `LinkType`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkType;

/// Placeholder for `LinkArg`.
#[derive(Debug, Clone, Default)]
pub struct LinkArg {
    label: String,
}

impl LinkArg {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into() }
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Placeholder for `LinkArrow`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LinkArrow;

/// Placeholder for `LinkConstraint`.
#[derive(Debug, Clone, Default)]
pub struct LinkConstraint;

/// Placeholder for `CucaNote`.
#[derive(Debug, Clone, Default)]
pub struct CucaNote;

/// Placeholder for Url.
#[derive(Debug, Clone, Default)]
pub struct Url;

/// Placeholder for Stereotype.
#[derive(Debug, Clone, Default)]
pub struct Stereotype;

/// Placeholder for `StyleBuilder`.
#[derive(Debug, Clone, Default)]
pub struct StyleBuilder;

/// Placeholder for `LineLocation`.
#[derive(Debug, Clone, Default)]
pub struct LineLocation {
    pub file: Option<String>,
    pub line: u32,
}

/// Placeholder for `CucaDiagram`.
#[derive(Debug, Clone, Default)]
pub struct CucaDiagramRef;

/// Relationship between two entities.
///
/// Ported from: `net/sourceforge/plantuml/abel/Link.java`
#[derive(Debug, Clone)]
#[allow(dead_code, clippy::struct_field_names)]
pub struct Link {
    entity1: Arc<Entity>,
    entity2: Arc<Entity>,
    port1: Option<String>,
    port2: Option<String>,
    link_arg: LinkArg,
    uid: String,
    note: Option<CucaNote>,
    invis: bool,
    weight: f64,
    constraint: bool,
    inverted: bool,
    link_arrow: LinkArrow,
    opale: bool,
    horizontal_solitary: bool,
    sametail: Option<String>,
    style_builder: StyleBuilder,
    stereotype: Option<Stereotype>,
    location: LineLocation,
    url: Option<Url>,
    link_constraint: Option<LinkConstraint>,
    code_line: LineLocation,
}

impl Link {
    /// Creates a new link between two entities.
    ///
    /// Ported from: `Link(LineLocation, CucaDiagram, StyleBuilder, Entity, Entity, LinkType, LinkArg)`.
    #[must_use]
    pub fn new(
        entity1: Arc<Entity>,
        entity2: Arc<Entity>,
        link_arg: LinkArg,
        location: LineLocation,
    ) -> Self {
        Self {
            entity1,
            entity2,
            port1: None,
            port2: None,
            link_arg,
            uid: String::new(),
            note: None,
            invis: false,
            weight: 0.0,
            constraint: true,
            inverted: false,
            link_arrow: LinkArrow,
            opale: false,
            horizontal_solitary: false,
            sametail: None,
            style_builder: StyleBuilder,
            stereotype: None,
            location,
            url: None,
            link_constraint: None,
            code_line: LineLocation::default(),
        }
    }

    /// Returns the first entity.
    ///
    /// Ported from: `Link.getEntity1()`.
    #[must_use]
    pub fn get_entity1(&self) -> &Entity {
        &self.entity1
    }

    /// Returns the second entity.
    ///
    /// Ported from: `Link.getEntity2()`.
    #[must_use]
    pub fn get_entity2(&self) -> &Entity {
        &self.entity2
    }

    /// Returns the port name for entity1, if any.
    ///
    /// Ported from: `Link.getPortName1()`.
    #[must_use]
    pub fn get_port_name1(&self) -> Option<&str> {
        self.port1.as_deref()
    }

    /// Returns the port name for entity2, if any.
    ///
    /// Ported from: `Link.getPortName2()`.
    #[must_use]
    pub fn get_port_name2(&self) -> Option<&str> {
        self.port2.as_deref()
    }

    /// Returns the UID.
    ///
    /// Ported from: `Link.getUid()`.
    #[must_use]
    pub fn get_uid(&self) -> &str {
        &self.uid
    }

    /// Sets the UID.
    pub fn set_uid(&mut self, uid: impl Into<String>) {
        self.uid = uid.into();
    }

    /// Returns `true` if the link is invisible.
    ///
    /// Ported from: `Link.isInvis()`.
    #[must_use]
    pub const fn is_invis(&self) -> bool {
        self.invis
    }

    /// Sets the invisible flag.
    ///
    /// Ported from: `Link.setInvis()`.
    pub const fn set_invis(&mut self, invis: bool) {
        self.invis = invis;
    }

    /// Returns the weight.
    ///
    /// Ported from: `Link.getWeight()`.
    #[must_use]
    pub const fn get_weight(&self) -> f64 {
        self.weight
    }

    /// Sets the weight.
    ///
    /// Ported from: `Link.setWeight()`.
    pub const fn set_weight(&mut self, weight: f64) {
        self.weight = weight;
    }

    /// Returns `true` if the link is a constraint.
    ///
    /// Ported from: `Link.isConstraint()`.
    #[must_use]
    pub const fn is_constraint(&self) -> bool {
        self.constraint
    }

    /// Sets the constraint flag.
    ///
    /// Ported from: `Link.setConstraint()`.
    pub const fn set_constraint(&mut self, constraint: bool) {
        self.constraint = constraint;
    }

    /// Returns `true` if the link is inverted.
    #[must_use]
    pub const fn is_inverted(&self) -> bool {
        self.inverted
    }

    /// Returns the link arrow.
    ///
    /// Ported from: `Link.getLinkArrow()`.
    #[must_use]
    pub const fn get_link_arrow(&self) -> LinkArrow {
        self.link_arrow
    }

    /// Sets the link arrow.
    ///
    /// Ported from: `Link.setLinkArrow()`.
    pub const fn set_link_arrow(&mut self, link_arrow: LinkArrow) {
        self.link_arrow = link_arrow;
    }

    /// Returns `true` if the link is opale (transparent style).
    #[must_use]
    pub const fn is_opale(&self) -> bool {
        self.opale
    }

    /// Sets the opale flag.
    ///
    /// Ported from: `Link.setOpale()`.
    pub const fn set_opale(&mut self, opale: bool) {
        self.opale = opale;
    }

    /// Returns `true` if the link is horizontally solitary.
    #[must_use]
    pub const fn is_horizontal_solitary(&self) -> bool {
        self.horizontal_solitary
    }

    /// Returns the sametail, if any.
    #[must_use]
    pub fn get_sametail(&self) -> Option<&str> {
        self.sametail.as_deref()
    }

    /// Sets the sametail.
    pub fn set_sametail(&mut self, sametail: impl Into<String>) {
        self.sametail = Some(sametail.into());
    }

    /// Returns the label.
    ///
    /// Ported from: `Link.getLabel()`.
    #[must_use]
    pub fn get_label(&self) -> &str {
        self.link_arg.label()
    }

    /// Returns the link arg.
    ///
    /// Ported from: `Link.getLinkArg()`.
    #[must_use]
    pub const fn get_link_arg(&self) -> &LinkArg {
        &self.link_arg
    }

    /// Returns the note, if any.
    #[must_use]
    pub const fn get_note(&self) -> Option<&CucaNote> {
        self.note.as_ref()
    }

    /// Adds a note to the link.
    ///
    /// Ported from: `Link.addNote()`.
    pub const fn add_note(&mut self, note: CucaNote) {
        self.note = Some(note);
    }

    /// Returns the URL, if any.
    #[must_use]
    pub const fn get_url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    /// Sets the URL.
    pub const fn set_url(&mut self, url: Url) {
        self.url = Some(url);
    }

    /// Returns `true` if the link has a URL.
    #[must_use]
    pub const fn has_url(&self) -> bool {
        self.url.is_some()
    }

    /// Returns the stereotype, if any.
    #[must_use]
    pub const fn get_stereotype(&self) -> Option<&Stereotype> {
        self.stereotype.as_ref()
    }

    /// Sets the stereotype.
    pub const fn set_stereotype(&mut self, stereotype: Stereotype) {
        self.stereotype = Some(stereotype);
    }

    /// Returns `true` if the link is hidden.
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        self.invis
    }

    /// Returns `true` if the link is removed.
    #[must_use]
    pub const fn is_removed(&self) -> bool {
        false
    }

    /// Returns the location.
    #[must_use]
    pub const fn get_location(&self) -> &LineLocation {
        &self.location
    }

    /// Returns `true` if this link connects the two given entities (in either order).
    ///
    /// Ported from: `Link.isBetween(Entity, Entity)`.
    #[must_use]
    pub fn is_between(&self, e1: &Entity, e2: &Entity) -> bool {
        (self.entity1.get_name() == e1.get_name() && self.entity2.get_name() == e2.get_name())
            || (self.entity1.get_name() == e2.get_name() && self.entity2.get_name() == e1.get_name())
    }

    /// Returns the other entity in the link (not the one given).
    ///
    /// Ported from: `Link.getOther(Entity)`.
    #[must_use]
    pub fn get_other(&self, entity: &Entity) -> Option<&Entity> {
        if self.entity1.get_name() == entity.get_name() {
            Some(&self.entity2)
        } else if self.entity2.get_name() == entity.get_name() {
            Some(&self.entity1)
        } else {
            None
        }
    }

    /// Returns `true` if this link contains the given entity.
    ///
    /// Ported from: `Link.contains(Entity)`.
    #[must_use]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.entity1.get_name() == entity.get_name()
            || self.entity2.get_name() == entity.get_name()
    }

    /// Returns the link constraint, if any.
    #[must_use]
    pub const fn get_link_constraint(&self) -> Option<&LinkConstraint> {
        self.link_constraint.as_ref()
    }

    /// Sets the link constraint.
    pub const fn set_link_constraint(&mut self, constraint: LinkConstraint) {
        self.link_constraint = Some(constraint);
    }
}

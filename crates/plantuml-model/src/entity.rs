//! Entity — central entity model for all UML diagrams.
//!
//! Ported from: `net/sourceforge/plantuml/abel/Entity.java`

use crate::cuca_note::CucaNote;
use crate::group_type::GroupType;
use crate::leaf_type::LeafType;
use crate::position::Position;
use crate::tip::Tip;
use crate::together::Together;
use plantuml_klimt::Display;
use std::collections::HashMap;

/// Placeholder for Bodier — body management.
#[derive(Debug, Clone, Default)]
pub struct BodierRef;

/// Placeholder for Quark — entity identity in namespace.
#[derive(Debug, Clone)]
pub struct Quark {
    name: String,
}

impl Quark {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}


/// Placeholder for Stereotype.
#[derive(Debug, Clone, Default)]
pub struct Stereotype;

/// Placeholder for `USymbol`.
#[derive(Debug, Clone, Default)]
pub struct USymbol;

/// Placeholder for Colors.
#[derive(Debug, Clone, Default)]
pub struct Colors;

/// Placeholder for Url.
#[derive(Debug, Clone, Default)]
pub struct Url;


/// Placeholder for `VisibilityModifier`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VisibilityModifier;

/// Placeholder for Neighborhood.
#[derive(Debug, Clone, Default)]
pub struct Neighborhood;

/// Placeholder for Margins.
#[derive(Debug, Clone, Default)]
pub struct Margins;

/// Placeholder for `IEntityImage`.
#[derive(Debug, Clone, Default)]
pub struct IEntityImage;

/// Placeholder for `LineLocation`.
#[derive(Debug, Clone, Default)]
pub struct LineLocation {
    pub file: Option<String>,
    pub line: u32,
}

/// Placeholder for `StyleBuilder`.
#[derive(Debug, Clone, Default)]
pub struct StyleBuilder;

/// Placeholder for `CucaDiagram`.
#[derive(Debug, Clone, Default)]
pub struct CucaDiagramRef;

/// Central entity model for all UML diagrams.
///
/// An entity can be either a leaf (class, state, usecase, etc.) or a
/// group (package, composite state, etc.). The `leaf_type` and
/// `group_type` fields are mutually exclusive — one is `Some` and the
/// other is `None`.
///
/// Ported from: `net/sourceforge/plantuml/abel/Entity.java`
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Entity {
    quark: Quark,
    uid: String,
    display: Display,
    leaf_type: Option<LeafType>,
    group_type: Option<GroupType>,
    stereotype: Option<Stereotype>,
    generic: Option<String>,
    symbol: Option<USymbol>,
    url: Option<Url>,
    bodier: BodierRef,
    legend: Option<Display>,
    margins: Margins,
    xposition: i32,
    raw_layout: i32,
    location: LineLocation,
    notes_top: Vec<CucaNote>,
    notes_bottom: Vec<CucaNote>,
    together: Option<Together>,
    packed: bool,
    is_static: bool,
    visibility: Option<VisibilityModifier>,
    neighborhood: Neighborhood,
    colors: Colors,
    #[allow(clippy::zero_sized_map_values)]
    tips: HashMap<String, Tip>,
    style_builder: StyleBuilder,
    hidden: bool,
    removed: bool,
    concurrent_separator: char,
}

impl Entity {
    /// Creates a new leaf entity.
    ///
    /// Ported from: `Entity(StyleBuilder, LineLocation, Quark, CucaDiagram, Bodier, LeafType, int)`.
    #[must_use]
    #[allow(clippy::zero_sized_map_values)]
    pub fn new_leaf(
        quark: Quark,
        leaf_type: LeafType,
        location: LineLocation,
    ) -> Self {
        Self {
            quark,
            uid: String::new(),
            display: Display::default(),
            leaf_type: Some(leaf_type),
            group_type: None,
            stereotype: None,
            generic: None,
            symbol: None,
            url: None,
            bodier: BodierRef,
            legend: None,
            margins: Margins,
            xposition: 0,
            raw_layout: 0,
            location,
            notes_top: Vec::new(),
            notes_bottom: Vec::new(),
            together: None,
            packed: false,
            is_static: false,
            visibility: None,
            neighborhood: Neighborhood,
            colors: Colors,
            tips: HashMap::new(),
            style_builder: StyleBuilder,
            hidden: false,
            removed: false,
            concurrent_separator: '\0',
        }
    }

    /// Creates a new group entity.
    ///
    /// Ported from: `Entity(StyleBuilder, LineLocation, Quark, CucaDiagram, Bodier, GroupType, int)`.
    #[must_use]
    #[allow(clippy::zero_sized_map_values)]
    pub fn new_group(
        quark: Quark,
        group_type: GroupType,
        location: LineLocation,
    ) -> Self {
        Self {
            quark,
            uid: String::new(),
            display: Display::default(),
            leaf_type: None,
            group_type: Some(group_type),
            stereotype: None,
            generic: None,
            symbol: None,
            url: None,
            bodier: BodierRef,
            legend: None,
            margins: Margins,
            xposition: 0,
            raw_layout: 0,
            location,
            notes_top: Vec::new(),
            notes_bottom: Vec::new(),
            together: None,
            packed: false,
            is_static: false,
            visibility: None,
            neighborhood: Neighborhood,
            colors: Colors,
            tips: HashMap::new(),
            style_builder: StyleBuilder,
            hidden: false,
            removed: false,
            concurrent_separator: '\0',
        }
    }

    /// Returns the leaf type, if this is a leaf entity.
    ///
    /// Ported from: `Entity.getLeafType()`.
    #[must_use]
    pub const fn get_leaf_type(&self) -> Option<LeafType> {
        self.leaf_type
    }

    /// Returns the group type, if this is a group entity.
    ///
    /// Ported from: `Entity.getGroupType()`.
    #[must_use]
    pub const fn get_group_type(&self) -> Option<GroupType> {
        self.group_type
    }

    /// Returns `true` if this is a group entity.
    ///
    /// Ported from: `Entity.isGroup()`.
    #[must_use]
    pub const fn is_group(&self) -> bool {
        self.group_type.is_some()
    }

    /// Returns the quark (identity in namespace).
    ///
    /// Ported from: `Entity.getQuark()`.
    #[must_use]
    pub const fn get_quark(&self) -> &Quark {
        &self.quark
    }

    /// Returns the entity name (from quark).
    ///
    /// Ported from: `Entity.getName()`.
    #[must_use]
    pub fn get_name(&self) -> &str {
        self.quark.name()
    }

    /// Returns the UID.
    ///
    /// Ported from: `Entity.getUid()`.
    #[must_use]
    pub fn get_uid(&self) -> &str {
        &self.uid
    }

    /// Sets the UID.
    ///
    /// Ported from: `Entity.setUid()`.
    pub fn set_uid(&mut self, uid: impl Into<String>) {
        self.uid = uid.into();
    }

    /// Returns the display.
    ///
    /// Ported from: `Entity.getDisplay()`.
    #[must_use]
    pub const fn get_display(&self) -> &Display {
        &self.display
    }

    /// Sets the display.
    ///
    /// Ported from: `Entity.setDisplay()`.
    pub fn set_display(&mut self, display: Display) {
        self.display = display;
    }

    /// Returns the stereotype, if any.
    ///
    /// Ported from: `Entity.getStereotype()`.
    #[must_use]
    pub const fn get_stereotype(&self) -> Option<&Stereotype> {
        self.stereotype.as_ref()
    }

    /// Sets the stereotype.
    ///
    /// Ported from: `Entity.setStereotype()`.
    pub const fn set_stereotype(&mut self, stereotype: Stereotype) {
        self.stereotype = Some(stereotype);
    }

    /// Returns `true` if the entity is hidden.
    ///
    /// Ported from: `Entity.isHidden()`.
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Sets the hidden flag.
    pub const fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    /// Returns `true` if the entity is removed.
    ///
    /// Ported from: `Entity.isRemoved()`.
    #[must_use]
    pub const fn is_removed(&self) -> bool {
        self.removed
    }

    /// Sets the removed flag.
    pub const fn set_removed(&mut self, removed: bool) {
        self.removed = removed;
    }

    /// Returns the x-position.
    ///
    /// Ported from: `Entity.getXposition()`.
    #[must_use]
    pub const fn get_xposition(&self) -> i32 {
        self.xposition
    }

    /// Sets the x-position.
    ///
    /// Ported from: `Entity.setXposition()`.
    pub const fn set_xposition(&mut self, xposition: i32) {
        self.xposition = xposition;
    }

    /// Returns the `USymbol`, if any.
    ///
    /// Ported from: `Entity.getUSymbol()`.
    #[must_use]
    pub const fn get_u_symbol(&self) -> Option<&USymbol> {
        self.symbol.as_ref()
    }

    /// Sets the `USymbol`.
    ///
    /// Ported from: `Entity.setUSymbol()`.
    pub const fn set_u_symbol(&mut self, symbol: USymbol) {
        self.symbol = Some(symbol);
    }

    /// Returns the colors.
    ///
    /// Ported from: `Entity.getColors()`.
    #[must_use]
    pub const fn get_colors(&self) -> &Colors {
        &self.colors
    }

    /// Returns the URL, if any.
    ///
    /// Ported from: `Entity.getUrl()`.
    #[must_use]
    pub const fn get_url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    /// Adds a URL.
    ///
    /// Ported from: `Entity.addUrl()`.
    pub const fn add_url(&mut self, url: Url) {
        self.url = Some(url);
    }

    /// Returns `true` if the entity has a URL.
    ///
    /// Ported from: `Entity.hasUrl()`.
    #[must_use]
    pub const fn has_url(&self) -> bool {
        self.url.is_some()
    }

    /// Adds a note to the top.
    ///
    /// Ported from: `Entity.addNote()`.
    pub fn add_note_top(&mut self, note: CucaNote) {
        self.notes_top.push(note);
    }

    /// Adds a note to the bottom.
    ///
    /// Ported from: `Entity.addNote()`.
    pub fn add_note_bottom(&mut self, note: CucaNote) {
        self.notes_bottom.push(note);
    }

    /// Returns the top notes.
    #[must_use]
    pub fn get_notes_top(&self) -> &[CucaNote] {
        &self.notes_top
    }

    /// Returns the bottom notes.
    #[must_use]
    pub fn get_notes_bottom(&self) -> &[CucaNote] {
        &self.notes_bottom
    }

    /// Returns `true` if this is the root group.
    ///
    /// Ported from: `Entity.isRoot()`.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.group_type == Some(GroupType::Root)
    }

    /// Returns `true` if this entity is a leaf and has no links.
    ///
    /// Ported from: `Entity.isAloneAndUnlinked()`.
    #[must_use]
    pub const fn is_alone_and_unlinked(&self) -> bool {
        self.leaf_type.is_some() && !self.has_url()
    }

    /// Returns the location.
    ///
    /// Ported from: `Entity.getLocation()`.
    #[must_use]
    pub const fn get_location(&self) -> &LineLocation {
        &self.location
    }

    /// Mutates the entity to a different leaf type.
    ///
    /// Ported from: `Entity.muteToType(LeafType)`.
    pub const fn mute_to_type(&mut self, leaf_type: LeafType) {
        self.leaf_type = Some(leaf_type);
        self.group_type = None;
    }

    /// Mutates the entity to a different group type.
    ///
    /// Ported from: `Entity.muteToGroupType(GroupType)`.
    pub const fn mute_to_group_type(&mut self, group_type: GroupType) {
        self.group_type = Some(group_type);
        self.leaf_type = None;
    }

    /// Mutates the entity to a leaf type with a `USymbol`.
    ///
    /// Ported from: `Entity.muteToType(LeafType, USymbol)`.
    pub const fn mute_to_type_with_symbol(&mut self, leaf_type: LeafType, symbol: USymbol) {
        self.mute_to_type(leaf_type);
        self.symbol = Some(symbol);
    }

    /// Returns `true` if the entity can be packed.
    ///
    /// Ported from: `Entity.canBePacked()`.
    #[must_use]
    pub const fn can_be_packed(&self) -> bool {
        self.packed
    }

    /// Returns the visibility modifier, if any.
    #[must_use]
    pub const fn get_visibility(&self) -> Option<VisibilityModifier> {
        self.visibility
    }

    /// Returns the generic type parameter, if any.
    #[must_use]
    pub fn get_generic(&self) -> Option<&str> {
        self.generic.as_deref()
    }

    /// Sets the generic type parameter.
    pub fn set_generic(&mut self, generic: impl Into<String>) {
        self.generic = Some(generic.into());
    }

    /// Adds a note to the entity at the given position.
    ///
    /// Ported from: `Entity.addNote(Display, Position, Colors)`.
    pub fn add_note(&mut self, note: Display, position: Position, colors: Colors) {
        let cuca_note = CucaNote::build(note, position, colors);
        match position {
            Position::Top => self.notes_top.push(cuca_note),
            Position::Bottom => self.notes_bottom.push(cuca_note),
            _ => {} // Left/Right notes handled differently in Java
        }
    }

    /// Returns notes at the given position.
    ///
    /// Ported from: `Entity.getNotes(Position)`.
    #[must_use]
    pub fn get_notes(&self, position: Position) -> &[CucaNote] {
        match position {
            Position::Top => &self.notes_top,
            Position::Bottom => &self.notes_bottom,
            _ => &[],
        }
    }

    /// Returns the concurrent separator character.
    ///
    /// Ported from: `Entity.getConcurrentSeparator()`.
    #[must_use]
    pub const fn get_concurrent_separator(&self) -> char {
        self.concurrent_separator
    }

    /// Sets the concurrent separator character.
    ///
    /// Ported from: `Entity.setConcurrentSeparator(char)`.
    pub const fn set_concurrent_separator(&mut self, separator: char) {
        self.concurrent_separator = separator;
    }

    /// Adds a tip for a member.
    ///
    /// Ported from: `Entity.putTip(String, Tip)`.
    pub fn put_tip(&mut self, key: impl Into<String>, tip: crate::tip::Tip) {
        self.tips.insert(key.into(), tip);
    }

    /// Returns all tips.
    ///
    /// Ported from: `Entity.getTips()`.
    #[must_use]
    pub const fn get_tips(&self) -> &HashMap<String, crate::tip::Tip> {
        &self.tips
    }

    /// Sets the together grouping.
    ///
    /// Ported from: `Entity.setTogether(Together)`.
    pub fn set_together(&mut self, together: crate::together::Together) {
        self.together = Some(together);
    }

    /// Returns the together grouping, if any.
    ///
    /// Ported from: `Entity.getTogether()`.
    #[must_use]
    pub const fn get_together(&self) -> Option<&crate::together::Together> {
        self.together.as_ref()
    }
}

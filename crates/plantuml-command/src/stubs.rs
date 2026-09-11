#![allow(dead_code)]
//! Stub types for unported Java dependencies.
//!
//! These are placeholder types for Java classes that haven't been ported yet.
//! They will be replaced with real implementations in later phases.

use std::sync::OnceLock;

/// Placeholder for `net.sourceforge.plantuml.utils.BlocLines`.
///
/// Represents a block of input lines for command parsing.
/// Will be properly ported from `BlocLines.java` in a later phase.
#[derive(Debug, Clone)]
pub struct BlocLines {
    lines: Vec<String>,
}

impl BlocLines {
    /// Creates a new `BlocLines` from a list of strings.
    #[must_use]
    pub const fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Returns the number of lines.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns `true` if there are no lines.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the first line, if any.
    #[must_use]
    pub fn first(&self) -> Option<&str> {
        self.lines.first().map(String::as_str)
    }

    /// Returns an iterator over the lines.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(String::as_str)
    }

    /// Returns the lines as a slice.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

/// Placeholder for `net.sourceforge.plantuml.AbstractDiagram`.
///
/// Will be properly ported in a later phase.
#[derive(Debug, Clone)]
pub struct AbstractDiagram;

/// Placeholder for `net.sourceforge.plantuml.klimt.display.Display`.
///
/// Rich text display. Will be properly ported from klimt in a later phase.
#[derive(Debug, Clone, Default)]
pub struct Display {
    text: String,
}

impl Display {
    /// Creates a new `Display` from a string.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Returns the display text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

/// Placeholder for `net.sourceforge.plantuml.utils.LineLocation`.
///
/// Tracks source file and line number for error reporting.
#[derive(Debug, Clone, Default)]
pub struct LineLocation {
    pub file: Option<String>,
    pub line: u32,
}

impl LineLocation {
    /// Creates a new `LineLocation`.
    #[must_use]
    pub const fn new(file: Option<String>, line: u32) -> Self {
        Self { file, line }
    }
}

/// Placeholder for `net.sourceforge.plantuml.utils.StringLocated`.
///
/// A string with location information.
#[derive(Debug, Clone)]
pub struct StringLocated {
    string: String,
    location: LineLocation,
}

impl StringLocated {
    /// Creates a new `StringLocated`.
    #[must_use]
    pub fn new(string: impl Into<String>, location: LineLocation) -> Self {
        Self {
            string: string.into(),
            location,
        }
    }

    /// Returns the string content.
    #[must_use]
    pub fn get_string(&self) -> &str {
        &self.string
    }

    /// Returns the location.
    #[must_use]
    pub const fn get_location(&self) -> &LineLocation {
        &self.location
    }
}

/// Placeholder for `net.sourceforge.plantuml.klimt.color.NoSuchColorException`.
///
/// Thrown when a color name cannot be resolved.
#[derive(Debug, Clone)]
pub struct NoSuchColorException {
    message: String,
}

impl NoSuchColorException {
    /// Creates a new `NoSuchColorException`.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for NoSuchColorException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NoSuchColorException {}

/// Placeholder for `net.sourceforge.plantuml.PSystemError`.
///
/// Error diagram that renders error messages.
#[derive(Debug, Clone)]
pub struct PSystemError {
    message: String,
}

impl PSystemError {
    /// Creates a new `PSystemError`.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.message
    }
}

/// Placeholder for `net.sourceforge.plantuml.style.StyleBuilder`.
///
/// Builds style objects. Will be properly ported from the style package.
#[derive(Debug, Clone, Default)]
pub struct StyleBuilder;

/// Placeholder for `net.sourceforge.plantuml.klimt.color.Colors`.
///
/// Color collection for entities. Will be properly ported from klimt.
#[derive(Debug, Clone, Default)]
pub struct Colors;

/// Placeholder for `net.sourceforge.plantuml.decoration.LinkType`.
///
/// Link/edge type. Will be properly ported from the decoration package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkType;

/// Placeholder for `net.sourceforge.plantuml.abel.LinkArg`.
///
/// Link arguments (label, quantifiers, etc.).
#[derive(Debug, Clone, Default)]
pub struct LinkArg;

/// Placeholder for `net.sourceforge.plantuml.decoration.USymbol`.
///
/// Symbol type for entities. Will be properly ported from the decoration package.
#[derive(Debug, Clone, Default)]
pub struct USymbol;

/// Placeholder for `net.sourceforge.plantuml.stereo.Stereotype`.
///
/// Entity stereotype. Will be properly ported from the stereo package.
#[derive(Debug, Clone, Default)]
pub struct Stereotype;

/// Placeholder for `net.sourceforge.plantuml.url.Url`.
///
/// URL attached to entities/links. Will be properly ported from the url package.
#[derive(Debug, Clone, Default)]
pub struct Url;

/// Placeholder for `net.sourceforge.plantuml.cucadiagram.CucaNote`.
///
/// Note attached to entities/links.
#[derive(Debug, Clone, Default)]
pub struct CucaNote;

/// Placeholder for `net.sourceforge.plantuml.cucadiagram.LinkConstraint`.
///
/// Constraint attached to links.
#[derive(Debug, Clone, Default)]
pub struct LinkConstraint;

/// Placeholder for `net.sourceforge.plantuml.abel.LinkArrow`.
///
/// Arrow direction on a link.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LinkArrow;

/// Placeholder for `net.sourceforge.plantuml.abel.Together`.
///
/// Together grouping for entities.
#[derive(Debug, Clone, Default)]
pub struct Together;

/// Placeholder for `net.sourceforge.plantuml.abel.Tip`.
///
/// Tip attached to entity members.
#[derive(Debug, Clone, Default)]
pub struct Tip;

/// Placeholder for `net.sourceforge.plantuml.skin.VisibilityModifier`.
///
/// Visibility modifier (public, private, etc.).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VisibilityModifier;

/// Placeholder for `net.sourceforge.plantuml.abel.Neighborhood`.
///
/// Entity neighborhood information.
#[derive(Debug, Clone, Default)]
pub struct Neighborhood;

/// Placeholder for `net.sourceforge.plantuml.klimt.geom.Margins`.
///
/// Entity margins.
#[derive(Debug, Clone, Default)]
pub struct Margins;

/// Placeholder for `net.sourceforge.plantuml.svek.IEntityImage`.
///
/// Entity image for svek layout.
#[derive(Debug, Clone, Default)]
pub struct IEntityImage;

/// Placeholder for `net.sourceforge.plantuml.cucadiagram.Bodier`.
///
/// Body management trait. Will be properly ported.
#[derive(Debug, Clone, Default)]
pub struct Bodier;

/// Placeholder for `net.sourceforge.plantuml.plasma.Quark`.
///
/// Quark tree node for entity namespace.
#[derive(Debug, Clone)]
pub struct Quark {
    name: String,
}

impl Quark {
    /// Creates a new `Quark` with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the quark name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Placeholder for `net.sourceforge.plantuml.plasma.Plasma`.
///
/// Quark tree namespace.
#[derive(Debug, Clone, Default)]
pub struct Plasma;

/// Placeholder for `net.sourceforge.plantuml.skin.ISkinParam`.
///
/// Skin parameters. Will be properly ported from the skin package.
#[derive(Debug, Clone, Default)]
pub struct ISkinParam;

/// Placeholder for `net.sourceforge.plantuml.klimt.creole.TextBlock`.
///
/// Already in plantuml-core but needs the full trait.
#[derive(Debug, Clone, Default)]
pub struct TextBlock;

/// Lazy-initialized value placeholder.
///
/// Ported from `Lazy<T>` pattern in Java.
pub struct Lazy<T> {
    cell: OnceLock<T>,
    init: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone> Lazy<T> {
    /// Creates a new `Lazy` with the given initializer.
    #[must_use]
    pub fn new<F>(init: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            cell: OnceLock::new(),
            init: Box::new(init),
        }
    }

    /// Gets the value, initializing if needed.
    #[must_use]
    pub fn get(&self) -> T {
        self.cell.get_or_init(&self.init).clone()
    }
}

impl<T> std::fmt::Debug for Lazy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lazy").finish_non_exhaustive()
    }
}

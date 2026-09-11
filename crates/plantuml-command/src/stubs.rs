#![allow(dead_code)]
//! Stub types for unported Java dependencies.
//!
//! These are placeholder types for Java classes that haven't been ported yet.
//! They will be replaced with real implementations in later phases.

use std::sync::OnceLock;

/// A block of input lines for command parsing.
///
/// Ported from: net/sourceforge/plantuml/utils/BlocLines.java
#[derive(Debug, Clone)]
pub struct BlocLines {
    lines: Vec<StringLocated>,
}

impl BlocLines {
    /// Creates an empty `BlocLines`.
    #[must_use]
    pub const fn empty() -> Self {
        Self { lines: Vec::new() }
    }

    /// Creates a `BlocLines` from a list of `StringLocated`.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(lines: Vec<StringLocated>) -> Self {
        Self { lines }
    }

    /// Creates a single-line `BlocLines` from one `StringLocated`.
    ///
    /// Ported from: `BlocLines.single(StringLocated)`.
    #[must_use]
    pub fn single(line: &StringLocated) -> Self {
        Self {
            lines: vec![line.clone()],
        }
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
    pub fn first(&self) -> Option<&StringLocated> {
        self.lines.first()
    }

    /// Returns an iterator over the lines.
    pub fn iter(&self) -> impl Iterator<Item = &StringLocated> {
        self.lines.iter()
    }

    /// Returns the lines as a slice.
    #[must_use]
    pub fn lines(&self) -> &[StringLocated] {
        &self.lines
    }

    /// Returns the string content of each line.
    #[must_use]
    pub fn get_strings(&self) -> Vec<&str> {
        self.lines.iter().map(StringLocated::get_string).collect()
    }

    /// Adds a line and returns a new `BlocLines`.
    ///
    /// Ported from: `BlocLines.add(StringLocated)`.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add(mut self, line: StringLocated) -> Self {
        self.lines.push(line);
        self
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

// Re-export `StringLocated` from the preprocessor crate — the canonical
// implementation with full location tracking.
pub use plantuml_preproc::StringLocated;

// Re-export `LineLocation` from the preprocessor crate.
pub use plantuml_preproc::stubs::LineLocation;

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

// Re-export `PSystemError` from plantuml-core — the canonical implementation.
pub use plantuml_core::PSystemError;

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

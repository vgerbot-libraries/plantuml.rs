//! Diagram trait — the top-level diagram abstraction.
//!
//! Ported from: net/sourceforge/plantuml/core/Diagram.java
//!
//! The Java interface extends `WarningHandler` and references several types
//! (`ImageData`, `UmlSource`, `Display`, `ParserPass`, `InstallationRequirement`)
//! that belong to later phases. Here we define the minimal trait surface that
//! foundation crates can depend on; concrete methods are added as their
//! dependencies are ported.

use std::io::Write;

use crate::file_format_option::FileFormatOption;

/// A rendered diagram, produced by a diagram-type factory.
pub trait Diagram {
    /// Export image `num` (0-indexed) to `os` in the given format.
    fn export_diagram(
        &self,
        os: &mut dyn Write,
        num: usize,
        file_format: &FileFormatOption,
    ) -> Result<(), crate::PlantumlError>;

    /// Number of images this diagram produces (usually 1).
    fn get_nb_images(&self) -> usize;

    /// Short human-readable description of the diagram type.
    fn get_description(&self) -> Option<String>;

    /// Warning or error message, if any.
    fn get_warning_or_error(&self) -> Option<String>;

    /// Whether the diagram contains clickable URLs.
    fn has_url(&self) -> bool;

    /// The root cause of any failure, if the diagram is an error diagram.
    fn root_cause(&self) -> Option<String>;
}

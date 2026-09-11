//! `UmlSource` — the textual source of a diagram.
//!
//! Ported from: net/sourceforge/plantuml/core/UmlSource.java
//!
//! Wraps the preprocessed `StringLocated` lines and the set of candidate
//! `DiagramType`s derived from the `@start<keyword>` directive.

use std::collections::HashSet;

use plantuml_core::DiagramType;
use plantuml_preproc::StringLocated;

/// The textual source of a diagram, starting with `@start<keyword>` and
/// ending with `@end<keyword>`.
///
/// Ported from: net/sourceforge/plantuml/core/UmlSource.java
#[derive(Debug, Clone)]
pub struct UmlSource {
    /// The preprocessed source lines (including `@start` / `@end` directives).
    source: Vec<StringLocated>,
    /// The candidate diagram types derived from the first line.
    diagram_types: HashSet<DiagramType>,
}

impl UmlSource {
    /// Creates a `UmlSource` from preprocessed lines.
    ///
    /// Ported from: `UmlSource.createWithRaw(source, checkEndingBackslash, rawSource)`.
    /// The backslash-joining logic is handled by the preprocessor, so here we
    /// take the lines as-is.
    #[must_use]
    pub fn new(source: Vec<StringLocated>) -> Self {
        let diagram_types = if source.is_empty() {
            HashSet::new()
        } else {
            DiagramType::find_start_types(source[0].get_string())
        };
        Self {
            source,
            diagram_types,
        }
    }

    /// Returns the candidate diagram types from the `@start` directive.
    ///
    /// Ported from: `UmlSource.getDiagramTypes()`.
    #[must_use]
    pub fn get_diagram_types(&self) -> &HashSet<DiagramType> {
        &self.diagram_types
    }

    /// Returns the source lines (including `@start` / `@end`).
    #[must_use]
    pub fn get_source(&self) -> &[StringLocated] {
        &self.source
    }

    /// Returns `true` if the source contains only the `@start` and `@end`
    /// directives (i.e. the diagram body is empty).
    ///
    /// Ported from: `UmlSource.getTotalLineCount() == 2` check in
    /// `PSystemCommandFactory.finalizeDiagram`.
    #[must_use]
    pub fn is_empty_body(&self) -> bool {
        self.source.len() <= 2
    }

    /// Returns the total number of source lines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.source.len()
    }

    /// Returns `true` if there are no source lines at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    /// Returns an iterator over the body lines (skipping the `@start` line).
    ///
    /// Ported from: `UmlSource.iterator2()` usage pattern where the first
    /// line (`@start`) is consumed first, then the body is iterated.
    pub fn body_iter(&self) -> impl Iterator<Item = &StringLocated> {
        self.source.iter().skip(1)
    }
}

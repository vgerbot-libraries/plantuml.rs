//! Diagram description returned by `Diagram::export_diagram`.
//!
//! Ported from: net/sourceforge/plantuml/core/DiagramDescription.java

/// A short description of a generated image, optionally carrying image data.
#[derive(Debug, Clone)]
pub struct DiagramDescription {
    description: String,
}

impl DiagramDescription {
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self { description: description.into() }
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Display for DiagramDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}

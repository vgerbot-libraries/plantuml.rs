//! Error types for diagram parsing and rendering.
//!
//! Ported from `net.sourceforge.plantuml.ErrorUml` and `ErrorUmlType`.

use plantuml_core::DiagramType;

/// The type of error that occurred.
///
/// Ported from `net.sourceforge.plantuml.ErrorUmlType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorUmlType {
    /// A syntax error in the diagram source.
    SyntaxError,
    /// An error during diagram execution/rendering.
    ExecutionError,
}

impl std::fmt::Display for ErrorUmlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError => write!(f, "SYNTAX_ERROR"),
            Self::ExecutionError => write!(f, "EXECUTION_ERROR"),
        }
    }
}

/// An error that occurred during diagram parsing or rendering.
///
/// Ported from `net.sourceforge.plantuml.ErrorUml`.
#[derive(Debug, Clone)]
pub struct ErrorUml {
    error: String,
    error_type: ErrorUmlType,
    score: i32,
    position: usize,
    diagram_type: Option<DiagramType>,
}

impl ErrorUml {
    /// Creates a new `ErrorUml` with full details.
    ///
    /// Ported from `ErrorUml(ErrorUmlType, String, int, StringLocated, DiagramType)`.
    #[must_use]
    pub fn new(
        error_type: ErrorUmlType,
        error: String,
        score: i32,
        position: usize,
        diagram_type: Option<DiagramType>,
    ) -> Self {
        Self {
            error,
            error_type,
            score,
            position,
            diagram_type,
        }
    }

    /// Creates a simple `ErrorUml` with no position or diagram type.
    ///
    /// Ported from `ErrorUml(ErrorUmlType, String)`.
    #[must_use]
    pub fn simple(error_type: ErrorUmlType, error: String) -> Self {
        Self {
            error,
            error_type,
            score: 0,
            position: 0,
            diagram_type: None,
        }
    }

    /// Returns the error message, optionally annotated with the assumed diagram type.
    ///
    /// Ported from `ErrorUml.getError`.
    #[must_use]
    pub fn get_error(&self) -> String {
        self.diagram_type.as_ref().map_or_else(|| self.error.clone(), |dt| format!("{} (Assumed diagram type: {})", self.error, dt.human_readable_name()))
    }

    /// Returns the error type.
    #[must_use]
    pub fn get_error_type(&self) -> ErrorUmlType {
        self.error_type
    }

    /// Returns the position (line number) of the error.
    #[must_use]
    pub fn get_position(&self) -> usize {
        self.position
    }

    /// Returns the score (used for error ranking).
    #[must_use]
    pub fn score(&self) -> i32 {
        self.score
    }
}

impl std::fmt::Display for ErrorUml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.error_type, self.position, self.error)
    }
}

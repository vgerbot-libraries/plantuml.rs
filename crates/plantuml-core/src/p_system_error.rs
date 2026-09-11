//! `PSystemError` — returned when a factory fails to create a diagram.
//!
//! Ported from: net/sourceforge/plantuml/error/PSystemError.java
//! and net/sourceforge/plantuml/ErrorUml.java

use crate::diagram_type::DiagramType;

/// The kind of error that occurred during diagram parsing or execution.
///
/// Ported from: net/sourceforge/plantuml/ErrorUmlType.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorUmlType {
    /// The source text could not be parsed by any command.
    SyntaxError,
    /// A command matched but failed during execution.
    ExecutionError,
}

/// An error produced when a `PSystemFactory` cannot create a valid diagram.
///
/// Ported from: net/sourceforge/plantuml/ErrorUml.java
#[derive(Debug, Clone)]
pub struct PSystemError {
    error_type: ErrorUmlType,
    message: String,
    score: i32,
    line_position: usize,
    diagram_type: DiagramType,
}

impl PSystemError {
    /// Creates a new `PSystemError`.
    ///
    /// Ported from: `ErrorUml(ErrorUmlType, String, int, StringLocated, DiagramType)`.
    #[must_use]
    pub fn new(
        error_type: ErrorUmlType,
        message: impl Into<String>,
        score: i32,
        line_position: usize,
        diagram_type: DiagramType,
    ) -> Self {
        Self {
            error_type,
            message: message.into(),
            score,
            line_position,
            diagram_type,
        }
    }

    /// Creates a syntax error with no position info.
    #[must_use]
    pub fn syntax(message: impl Into<String>, diagram_type: DiagramType) -> Self {
        Self::new(ErrorUmlType::SyntaxError, message, 0, 0, diagram_type)
    }

    /// Creates an execution error with no position info.
    #[must_use]
    pub fn execution(message: impl Into<String>, diagram_type: DiagramType) -> Self {
        Self::new(ErrorUmlType::ExecutionError, message, 0, 0, diagram_type)
    }

    /// Returns the error message.
    #[must_use]
    pub fn get_error(&self) -> &str {
        &self.message
    }

    /// Returns the error type.
    #[must_use]
    pub const fn get_error_type(&self) -> ErrorUmlType {
        self.error_type
    }

    /// Returns the score (higher = more severe).
    #[must_use]
    pub const fn get_score(&self) -> i32 {
        self.score
    }

    /// Returns the line position (0-indexed) where the error was detected.
    #[must_use]
    pub const fn get_position(&self) -> usize {
        self.line_position
    }

    /// Returns the diagram type that was being attempted.
    #[must_use]
    pub const fn get_diagram_type(&self) -> DiagramType {
        self.diagram_type
    }
}

impl std::fmt::Display for PSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {}", self.error_type, self.line_position, self.message)
    }
}

impl std::error::Error for PSystemError {}

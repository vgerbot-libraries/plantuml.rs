//! Command execution result — outcome of executing a command.
//!
//! Ported from: `net/sourceforge/plantuml/command/CommandExecutionResult.java`

use crate::stubs::AbstractDiagram;

/// Result of executing a command against a diagram.
///
/// Ported from: `net/sourceforge/plantuml/command/CommandExecutionResult.java`
#[derive(Debug, Clone)]
pub struct CommandExecutionResult {
    error: Option<String>,
    new_diagram: Option<AbstractDiagram>,
    debug_lines: Vec<String>,
    score: i32,
    root_cause: Option<String>,
}

impl CommandExecutionResult {
    /// Creates a successful result.
    ///
    /// Ported from: `CommandExecutionResult.ok()`.
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            error: None,
            new_diagram: None,
            debug_lines: Vec::new(),
            score: 0,
            root_cause: None,
        }
    }

    /// Creates an error result with a message.
    ///
    /// Ported from: `CommandExecutionResult.error(String)`.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            error: Some(message.into()),
            new_diagram: None,
            debug_lines: Vec::new(),
            score: 0,
            root_cause: None,
        }
    }

    /// Creates an error result with a message and score.
    ///
    /// Ported from: `CommandExecutionResult.error(String, int)`.
    #[must_use]
    pub fn error_with_score(message: impl Into<String>, score: i32) -> Self {
        Self {
            error: Some(message.into()),
            new_diagram: None,
            debug_lines: Vec::new(),
            score,
            root_cause: None,
        }
    }

    /// Creates an error result with a message and root cause.
    ///
    /// Ported from: `CommandExecutionResult.error(String, Throwable)`.
    #[must_use]
    pub fn error_with_cause(message: impl Into<String>, cause: impl Into<String>) -> Self {
        Self {
            error: Some(message.into()),
            new_diagram: None,
            debug_lines: Vec::new(),
            score: 0,
            root_cause: Some(cause.into()),
        }
    }

    /// Creates a bad color error.
    ///
    /// Ported from: `CommandExecutionResult.badColor()`.
    #[must_use]
    pub fn bad_color() -> Self {
        Self::error("Bad color")
    }

    /// Creates a result with a new diagram.
    ///
    /// Ported from: `CommandExecutionResult.newDiagram(AbstractDiagram)`.
    #[must_use]
    pub const fn new_diagram(diagram: AbstractDiagram) -> Self {
        Self {
            error: None,
            new_diagram: Some(diagram),
            debug_lines: Vec::new(),
            score: 0,
            root_cause: None,
        }
    }

    /// Returns `true` if the command executed successfully.
    ///
    /// Ported from: `CommandExecutionResult.isOk()`.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// Returns the error message, if any.
    ///
    /// Ported from: `CommandExecutionResult.getError()`.
    #[must_use]
    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns the score.
    ///
    /// Ported from: `CommandExecutionResult.getScore()`.
    #[must_use]
    pub const fn get_score(&self) -> i32 {
        self.score
    }

    /// Returns the new diagram, if any.
    ///
    /// Ported from: `CommandExecutionResult.getNewDiagram()`.
    #[must_use]
    pub const fn get_new_diagram(&self) -> Option<&AbstractDiagram> {
        self.new_diagram.as_ref()
    }

    /// Returns debug lines.
    ///
    /// Ported from: `CommandExecutionResult.getDebugLines()`.
    #[must_use]
    pub fn get_debug_lines(&self) -> &[String] {
        &self.debug_lines
    }

    /// Returns the root cause, if any.
    ///
    /// Ported from: `CommandExecutionResult.getRootCause()`.
    #[must_use]
    pub fn get_root_cause(&self) -> Option<&str> {
        self.root_cause.as_deref()
    }

    /// Sets the diagram and returns self for chaining.
    ///
    /// Ported from: `CommandExecutionResult.withDiagram(AbstractDiagram)`.
    #[must_use]
    pub const fn with_diagram(mut self, diagram: AbstractDiagram) -> Self {
        self.new_diagram = Some(diagram);
        self
    }
}

impl Default for CommandExecutionResult {
    fn default() -> Self {
        Self::ok()
    }
}

//! Command trait — interface for all diagram commands.
//!
//! Ported from: `net/sourceforge/plantuml/command/Command.java`

use crate::command_control::CommandControl;
use crate::command_execution_result::CommandExecutionResult;
use crate::parser_pass::ParserPass;
use crate::stubs::BlocLines;

/// Trait for commands that parse and execute diagram input lines.
///
/// Each diagram type registers a list of `Command` implementations.
/// The `PSystemCommandFactory` tries each command in order against
/// the input lines.
///
/// Ported from: `net/sourceforge/plantuml/command/Command.java`
pub trait Command<D> {
    /// Checks if this command matches the given input lines.
    ///
    /// Returns `Ok` for a full match, `NotOk` if the command doesn't match,
    /// or `OkPartial` for a multi-line command that needs more input.
    ///
    /// Ported from: `Command.isValid(BlocLines)`.
    fn is_valid(&self, lines: &BlocLines) -> CommandControl;

    /// Executes this command against the diagram.
    ///
    /// Returns a `CommandExecutionResult` indicating success or failure.
    ///
    /// Ported from: `Command.execute(D, BlocLines, ParserPass)`.
    fn execute(
        &self,
        diagram: &mut D,
        lines: &BlocLines,
        pass: ParserPass,
    ) -> Result<CommandExecutionResult, crate::stubs::NoSuchColorException>;

    /// Returns an explanation string for debugging.
    ///
    /// Ported from: `Command.explain(BlocLines)`.
    fn explain(&self, _lines: &BlocLines) -> Option<String> {
        None
    }

    /// Returns `true` if this command should run during the given parser pass.
    ///
    /// Most commands are eligible for `ParserPass::One` only.
    ///
    /// Ported from: `Command.isEligibleFor(ParserPass)`.
    fn is_eligible_for(&self, pass: ParserPass) -> bool {
        pass == ParserPass::One
    }

    /// Returns `true` if this command is forbidden for the given lines.
    ///
    /// Default implementation returns `false`.
    ///
    /// Ported from: `Command.isCommandForbidden(BlocLines)`.
    fn is_command_forbidden(&self, _lines: &BlocLines) -> bool {
        false
    }
}

//! Single-line command base — uses legacy regex engine.
//!
//! Ported from: `net/sourceforge/plantuml/command/SingleLineCommand2.java`

use crate::command::Command;
use crate::command_control::CommandControl;
use crate::command_execution_result::CommandExecutionResult;
use crate::parser_pass::ParserPass;
use crate::stubs::{BlocLines, LineLocation, NoSuchColorException};
use plantuml_regex::legacy::iregex::IRegex;
use plantuml_regex::legacy::regex_result::RegexResult;

/// Base class for single-line commands using the legacy regex engine.
///
/// Subclasses implement `execute_arg` to handle the matched regex groups.
///
/// Ported from: `net/sourceforge/plantuml/command/SingleLineCommand2.java`
pub struct SingleLineCommand2<D> {
    pattern: Box<dyn IRegex>,
    do_trim: bool,
    _marker: std::marker::PhantomData<fn() -> D>,
}

impl<D> SingleLineCommand2<D> {
    /// Creates a new `SingleLineCommand2` with the given pattern.
    ///
    /// Ported from: `SingleLineCommand2(IRegex)`.
    #[must_use]
    pub fn new(pattern: Box<dyn IRegex>) -> Self {
        Self {
            pattern,
            do_trim: true,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new `SingleLineCommand2` with trim control.
    ///
    /// Ported from: `SingleLineCommand2(boolean, IRegex)`.
    #[must_use]
    pub fn with_trim(do_trim: bool, pattern: Box<dyn IRegex>) -> Self {
        Self {
            pattern,
            do_trim,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the regex pattern.
    #[must_use]
    pub fn pattern(&self) -> &dyn IRegex {
        &*self.pattern
    }

    /// Returns whether trimming is enabled.
    #[must_use]
    pub const fn do_trim(&self) -> bool {
        self.do_trim
    }

    /// Returns `false` — single-line commands don't use final bracket syntax.
    ///
    /// Ported from: `SingleLineCommand2.syntaxWithFinalBracket()`.
    #[must_use]
    pub const fn syntax_with_final_bracket(&self) -> bool {
        false
    }

    /// Checks if the command is forbidden for the given input.
    ///
    /// Ported from: `SingleLineCommand2.isForbidden(CharSequence)`.
    #[must_use]
    pub const fn is_forbidden(&self, _input: &str) -> bool {
        false
    }

    /// Final verification after matching. Default returns `Ok`.
    ///
    /// Ported from: `SingleLineCommand2.finalVerification()`.
    #[must_use]
    pub const fn final_verification(&self) -> CommandExecutionResult {
        CommandExecutionResult::ok()
    }

    /// Explains a single argument match.
    ///
    /// Ported from: `SingleLineCommand2.explainArg(LineLocation, RegexResult)`.
    #[must_use]
    pub const fn explain_arg(&self, _location: &LineLocation, _result: &RegexResult) -> Option<String> {
        None
    }
}

impl<D: 'static> Command<D> for SingleLineCommand2<D> {
    fn is_valid(&self, lines: &BlocLines) -> CommandControl {
        let Some(first) = lines.first() else {
            return CommandControl::NotOk;
        };
        let text = if self.do_trim { first.trim() } else { first };
        if self.is_forbidden(text) {
            return CommandControl::NotOk;
        }
        match self.pattern.matcher(text) {
            Some(_) => CommandControl::Ok,
            None => CommandControl::NotOk,
        }
    }

    fn execute(
        &self,
        _diagram: &mut D,
        lines: &BlocLines,
        _pass: ParserPass,
    ) -> Result<CommandExecutionResult, NoSuchColorException> {
        let Some(first) = lines.first() else {
            return Ok(CommandExecutionResult::error("Empty input"));
        };
        let text = if self.do_trim { first.trim() } else { first };
        self.pattern.matcher(text).map_or_else(
            || Ok(CommandExecutionResult::error("Pattern does not match")),
            |result| {
                let _ = result;
                Ok(self.final_verification())
            },
        )
    }

    fn explain(&self, lines: &BlocLines) -> Option<String> {
        let first = lines.first()?;
        let text = if self.do_trim { first.trim() } else { first };
        let result = self.pattern.matcher(text)?;
        self.explain_arg(&LineLocation::default(), &result)
    }

    fn is_eligible_for(&self, pass: ParserPass) -> bool {
        pass == ParserPass::One
    }
}

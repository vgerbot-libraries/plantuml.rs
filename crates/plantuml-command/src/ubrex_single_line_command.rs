//! UBrex single-line command base — uses ubrex regex engine.
//!
//! Ported from: `net/sourceforge/plantuml/command/UBrexSingleLineCommand2.java`

use crate::command::Command;
use crate::command_control::CommandControl;
use crate::command_execution_result::CommandExecutionResult;
use crate::parser_pass::ParserPass;
use crate::stubs::{BlocLines, LineLocation, NoSuchColorException};
use plantuml_regex::ubrex::unicode_bracketed_expression::UnicodeBracketedExpression;
use plantuml_regex::legacy::regex_result::RegexResult;

/// Base class for single-line commands using the ubrex regex engine.
///
/// Mirror of `SingleLineCommand2` but uses `UnicodeBracketedExpression`
/// instead of `IRegex` for pattern matching.
///
/// Ported from: `net/sourceforge/plantuml/command/UBrexSingleLineCommand2.java`
pub struct UbrexSingleLineCommand2<D> {
    pattern: UnicodeBracketedExpression,
    do_trim: bool,
    _marker: std::marker::PhantomData<fn() -> D>,
}

impl<D> UbrexSingleLineCommand2<D> {
    /// Creates a new `UbrexSingleLineCommand2` with the given pattern.
    ///
    /// Ported from: `UBrexSingleLineCommand2(UnicodeBracketedExpression)`.
    #[must_use]
    pub fn new(pattern: UnicodeBracketedExpression) -> Self {
        Self {
            pattern,
            do_trim: true,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new `UbrexSingleLineCommand2` with trim control.
    ///
    /// Ported from: `UBrexSingleLineCommand2(boolean, UnicodeBracketedExpression)`.
    #[must_use]
    pub fn with_trim(do_trim: bool, pattern: UnicodeBracketedExpression) -> Self {
        Self {
            pattern,
            do_trim,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the ubrex pattern.
    #[must_use]
    pub fn pattern(&self) -> &UnicodeBracketedExpression {
        &self.pattern
    }

    /// Returns whether trimming is enabled.
    #[must_use]
    pub fn do_trim(&self) -> bool {
        self.do_trim
    }

    /// Returns `false` — single-line commands don't use final bracket syntax.
    #[must_use]
    pub fn syntax_with_final_bracket(&self) -> bool {
        false
    }

    /// Checks if the command is forbidden for the given input.
    ///
    /// Ported from: `UBrexSingleLineCommand2.isForbidden(CharSequence)`.
    #[must_use]
    pub fn is_forbidden(&self, _input: &str) -> bool {
        false
    }

    /// Final verification after matching. Default returns `Ok`.
    ///
    /// Ported from: `UBrexSingleLineCommand2.finalVerification()`.
    #[must_use]
    pub fn final_verification(&self) -> CommandExecutionResult {
        CommandExecutionResult::ok()
    }

    /// Explains a single argument match.
    ///
    /// Ported from: `UBrexSingleLineCommand2.explainArg(LineLocation, RegexResult)`.
    #[must_use]
    pub fn explain_arg(&self, _location: &LineLocation, _result: &RegexResult) -> Option<String> {
        None
    }
}

impl<D: 'static> Command<D> for UbrexSingleLineCommand2<D> {
    fn is_valid(&self, lines: &BlocLines) -> CommandControl {
        let Some(first) = lines.first() else {
            return CommandControl::NotOk;
        };
        let text = if self.do_trim { first.trim() } else { first };
        if self.is_forbidden(text) {
            return CommandControl::NotOk;
        }
        let matcher = self.pattern.match_str(text, 0);
        if matcher.exact_match() {
            CommandControl::Ok
        } else {
            CommandControl::NotOk
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
        let matcher = self.pattern.match_str(text, 0);
        if matcher.exact_match() {
            let result = RegexResult::from_umatcher(matcher);
            let _ = result;
            Ok(self.final_verification())
        } else {
            Ok(CommandExecutionResult::error("Pattern does not match"))
        }
    }

    fn explain(&self, lines: &BlocLines) -> Option<String> {
        let first = lines.first()?;
        let text = if self.do_trim { first.trim() } else { first };
        let matcher = self.pattern.match_str(text, 0);
        if matcher.exact_match() {
            let result = RegexResult::from_umatcher(matcher);
            self.explain_arg(&LineLocation::default(), &result)
        } else {
            None
        }
    }

    fn is_eligible_for(&self, pass: ParserPass) -> bool {
        pass == ParserPass::One
    }
}

//! `UBrex` multi-line command base — uses ubrex regex engine.
//!
//! Ported from: `net/sourceforge/plantuml/command/UBrexCommandMultilines2.java`

use crate::command::Command;
use crate::command_control::CommandControl;
use crate::command_execution_result::CommandExecutionResult;
use crate::multilines_strategy::MultilinesStrategy;
use crate::parser_pass::ParserPass;
use crate::stubs::{BlocLines, NoSuchColorException};
use crate::trim::Trim;
use plantuml_regex::ubrex::unicode_bracketed_expression::{build as ubrex_build, UnicodeBracketedExpression};

/// Base class for multi-line commands using the ubrex regex engine.
///
/// Mirror of `CommandMultilines2` but uses `UnicodeBracketedExpression`
/// for both start and end patterns.
///
/// Ported from: `net/sourceforge/plantuml/command/UBrexCommandMultilines2.java`
pub struct UbrexCommandMultilines2<D> {
    starting: UnicodeBracketedExpression,
    trim_end: Trim,
    strategy: MultilinesStrategy,
    end: std::sync::OnceLock<UnicodeBracketedExpression>,
    end_pattern_string: String,
    _marker: std::marker::PhantomData<fn() -> D>,
}

impl<D> UbrexCommandMultilines2<D> {
    /// Creates a new `UbrexCommandMultilines2`.
    ///
    /// Ported from: `UBrexCommandMultilines2(UnicodeBracketedExpression, MultilinesStrategy, Trim, Lazy<UnicodeBracketedExpression>)`.
    #[must_use]
    pub fn new(
        starting: UnicodeBracketedExpression,
        strategy: MultilinesStrategy,
        trim_end: Trim,
        end_pattern: String,
    ) -> Self {
        Self {
            starting,
            trim_end,
            strategy,
            end: std::sync::OnceLock::new(),
            end_pattern_string: end_pattern,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the starting pattern.
    #[must_use]
    pub const fn starting_pattern(&self) -> &UnicodeBracketedExpression {
        &self.starting
    }

    /// Returns the end pattern, compiling it lazily.
    #[must_use]
    pub fn end_pattern(&self) -> &UnicodeBracketedExpression {
        self.end.get_or_init(|| ubrex_build(&self.end_pattern_string))
    }

    /// Returns the trim mode for the end pattern.
    #[must_use]
    pub const fn trim_end(&self) -> Trim {
        self.trim_end
    }

    /// Returns the multilines strategy.
    #[must_use]
    pub const fn strategy(&self) -> MultilinesStrategy {
        self.strategy
    }

    /// Returns `false` — multi-line commands don't use final bracket syntax.
    #[must_use]
    pub const fn syntax_with_final_bracket(&self) -> bool {
        false
    }

    /// Final verification after collecting all lines. Default returns `Ok`.
    ///
    /// Ported from: `UBrexCommandMultilines2.finalVerification(BlocLines)`.
    #[must_use]
    pub const fn final_verification(&self, _lines: &BlocLines) -> CommandExecutionResult {
        CommandExecutionResult::ok()
    }
}

impl<D: 'static> Command<D> for UbrexCommandMultilines2<D> {
    fn is_valid(&self, lines: &BlocLines) -> CommandControl {
        let Some(first) = lines.first() else {
            return CommandControl::NotOk;
        };
        let text = first.trim();
        let matcher = self.starting.match_str(text, 0);
        if matcher.exact_match() {
            // Check if end pattern matches any subsequent line
            let end = self.end_pattern();
            for line in lines.iter().skip(1) {
                let trimmed = self.trim_end.trim(line);
                let end_matcher = end.match_str(&trimmed, 0);
                if end_matcher.exact_match() {
                    return CommandControl::Ok;
                }
            }
            // Start matched but end not found — partial match
            CommandControl::OkPartial
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
        Ok(self.final_verification(lines))
    }

    fn explain(&self, _lines: &BlocLines) -> Option<String> {
        None
    }

    fn is_eligible_for(&self, pass: ParserPass) -> bool {
        pass == ParserPass::One
    }
}

//! Multi-line command base — uses legacy regex engine.
//!
//! Ported from: `net/sourceforge/plantuml/command/CommandMultilines2.java`

use crate::command::Command;
use crate::command_control::CommandControl;
use crate::command_execution_result::CommandExecutionResult;
use crate::multilines_strategy::MultilinesStrategy;
use crate::parser_pass::ParserPass;
use crate::stubs::{BlocLines, NoSuchColorException};
use crate::trim::Trim;
use plantuml_regex::legacy::iregex::IRegex;
use plantuml_regex::legacy::pattern2::Pattern2;

/// Base class for multi-line commands using the legacy regex engine.
///
/// Multi-line commands have a start pattern and an end pattern.
/// Lines between start and end are collected and passed to `execute_now`.
///
/// Ported from: `net/sourceforge/plantuml/command/CommandMultilines2.java`
pub struct CommandMultilines2<D> {
    starting: Box<dyn IRegex>,
    trim_end: Trim,
    strategy: MultilinesStrategy,
    end: std::sync::OnceLock<Pattern2>,
    end_pattern_string: String,
    _marker: std::marker::PhantomData<fn() -> D>,
}

impl<D> CommandMultilines2<D> {
    /// Creates a new `CommandMultilines2`.
    ///
    /// Ported from: `CommandMultilines2(IRegex, MultilinesStrategy, Trim, Lazy<Pattern2>)`.
    #[must_use]
    pub fn new(
        starting: Box<dyn IRegex>,
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
    ///
    /// Ported from: `CommandMultilines2.getStartingPattern()`.
    #[must_use]
    pub fn starting_pattern(&self) -> &dyn IRegex {
        &*self.starting
    }

    /// Returns the end pattern, compiling it lazily.
    ///
    /// Ported from: `CommandMultilines2.getEndPattern()`.
    #[must_use]
    pub fn end_pattern(&self) -> &Pattern2 {
        self.end.get_or_init(|| Pattern2::cmpile(&self.end_pattern_string))
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
    ///
    /// Ported from: `CommandMultilines2.syntaxWithFinalBracket()`.
    #[must_use]
    pub const fn syntax_with_final_bracket(&self) -> bool {
        false
    }

    /// Final verification after collecting all lines. Default returns `Ok`.
    ///
    /// Ported from: `CommandMultilines2.finalVerification(BlocLines)`.
    #[must_use]
    pub const fn final_verification(&self, _lines: &BlocLines) -> CommandExecutionResult {
        CommandExecutionResult::ok()
    }
}

impl<D: 'static> Command<D> for CommandMultilines2<D> {
    fn is_valid(&self, lines: &BlocLines) -> CommandControl {
        let Some(first) = lines.first() else {
            return CommandControl::NotOk;
        };
        let text = first.get_string().trim();
        match self.starting.matcher(text) {
            Some(_) => {
                // Check if end pattern matches any subsequent line
                let end = self.end_pattern();
                for line in lines.iter().skip(1) {
                    let trimmed = self.trim_end.trim(line.get_string());
                    if end.matcher(&trimmed, 0).matches() {
                        return CommandControl::Ok;
                    }
                }
                // Start matched but end not found — partial match
                CommandControl::OkPartial
            }
            None => CommandControl::NotOk,
        }
    }

    fn execute(
        &self,
        _diagram: &mut D,
        lines: &BlocLines,
        _pass: ParserPass,
    ) -> Result<CommandExecutionResult, NoSuchColorException> {
        // Base implementation: just return Ok.
        // Subclasses implement execute_now.
        Ok(self.final_verification(lines))
    }

    fn explain(&self, _lines: &BlocLines) -> Option<String> {
        None
    }

    fn is_eligible_for(&self, pass: ParserPass) -> bool {
        pass == ParserPass::One
    }
}

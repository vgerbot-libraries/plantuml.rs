//! Eater for function call parsing.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterFunctionCall`.

use std::collections::HashMap;

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::expression::{TokenStack, TValue};
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::StringLocated;

/// Parses a function call (arguments inside parentheses).
///
/// Ported from `net.sourceforge.plantuml.tim.EaterFunctionCall`.
pub struct EaterFunctionCall {
    eater: Eater,
    values: Vec<TValue>,
    named_arguments: HashMap<String, TValue>,
    is_legacy_define: bool,
    unquoted: bool,
}

impl EaterFunctionCall {
    /// Creates a new `EaterFunctionCall`.
    #[must_use]
    pub fn new(s: StringLocated, is_legacy_define: bool, unquoted: bool) -> Self {
        Self {
            eater: Eater::new(s),
            values: Vec::new(),
            named_arguments: HashMap::new(),
            is_legacy_define,
            unquoted,
        }
    }

    /// Analyzes the function call.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_until_char('(');
        self.eater.check_and_eat_char('(')?;
        self.eater.skip_spaces();
        if self.eater.peek_char() == ')' {
            self.eater.check_and_eat_char(')')?;
            return Ok(());
        }
        loop {
            self.eater.skip_spaces();
            if self.is_legacy_define {
                let read = self.eater.eat_and_get_optional_quoted_string()?;
                let value = context.apply_functions_and_variables(
                    memory,
                    &StringLocated::new(read, self.eater.get_line_location()),
                );
                self.values.push(TValue::from_string(value.unwrap_or_default()));
            } else if self.unquoted {
                if self.eater.match_affectation() {
                    let varname = self.eater.eat_and_get_varname()?;
                    self.eater.skip_spaces();
                    self.eater.check_and_eat_char('=')?;
                    self.eater.skip_spaces();
                    let read = self.eater.eat_and_get_optional_quoted_string()?;
                    let value = context.apply_functions_and_variables(
                        memory,
                        &StringLocated::new(read, self.eater.get_line_location()),
                    );
                    self.named_arguments.insert(varname, TValue::from_string(value.unwrap_or_default()));
                } else {
                    let read = self.eater.eat_and_get_optional_quoted_string()?;
                    let value = context.apply_functions_and_variables(
                        memory,
                        &StringLocated::new(read, self.eater.get_line_location()),
                    );
                    self.values.push(TValue::from_string(value.unwrap_or_default()));
                }
            } else if self.eater.match_affectation() {
                let varname = self.eater.eat_and_get_varname()?;
                self.eater.skip_spaces();
                self.eater.check_and_eat_char('=')?;
                self.eater.skip_spaces();
                let mut tokens = TokenStack::eat_until_close_parenthesis_or_comma(&mut self.eater)?;
                tokens = tokens.without_space();
                tokens.guess_functions(self.eater.get_string_located())?;
                let result = tokens.get_result(self.eater.get_string_located(), context, memory)?;
                self.named_arguments.insert(varname, result);
            } else {
                let mut tokens = TokenStack::eat_until_close_parenthesis_or_comma(&mut self.eater)?;
                tokens = tokens.without_space();
                tokens.guess_functions(self.eater.get_string_located())?;
                let result = tokens.get_result(self.eater.get_string_located(), context, memory)?;
                self.values.push(result);
            }
            self.eater.skip_spaces();
            let ch = self.eater.eat_one_char();
            if ch == ',' {
                continue;
            }
            if ch == ')' {
                break;
            }
            if self.unquoted {
                return Err(EaterException::new(
                    "unquoted function/procedure cannot use expression.",
                    self.eater.get_string_located(),
                ));
            }
            return Err(EaterException::new("call001", self.eater.get_string_located()));
        }
        Ok(())
    }

    /// Returns the positional argument values.
    #[must_use]
    pub fn get_values(&self) -> &[TValue] {
        &self.values
    }

    /// Returns the named arguments.
    #[must_use]
    pub fn get_named_arguments(&self) -> &HashMap<String, TValue> {
        &self.named_arguments
    }

    /// Returns the current position in the eater.
    #[must_use]
    pub fn get_current_position(&self) -> usize {
        self.eater.get_current_position()
    }

    /// Eats and returns the end of line.
    pub fn get_end_of_line(&mut self) -> Result<String, EaterException> {
        Ok(self.eater.eat_all_to_end())
    }
}

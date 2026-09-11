//! Abstract base class for parsing preprocessor directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.Eater`.

use std::sync::LazyLock;

use regex::Regex;

use super::eater_exception::EaterException;
use super::expression::{Token, TokenStack, TokenType, TValue};
use super::t_context::TContext;
use super::t_function_impl::TFunctionImpl;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;
use crate::stubs::LineLocation;
use crate::{StringLocated, TLineType};

/// Abstract base for parsing preprocessor directives.
///
/// Ported from `net.sourceforge.plantuml.tim.Eater`.
pub struct Eater {
    i: usize,
    string_located: StringLocated,
}

static AFFECTATION_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$?[_\p{L}][_0-9]*\s*=.*").unwrap_or_else(|_| Regex::new(r"\$?[A-Za-z_][A-Za-z_0-9]*\s*=.*").unwrap()));

impl Eater {
    /// Creates a new `Eater` from a `StringLocated`.
    #[must_use]
    pub fn new(string_located: StringLocated) -> Self {
        Self {
            i: 0,
            string_located,
        }
    }

    /// Returns the line location.
    #[must_use]
    pub fn get_line_location(&self) -> LineLocation {
        self.string_located.get_location().clone()
    }

    /// Returns the `StringLocated`.
    #[must_use]
    pub fn get_string_located(&self) -> &StringLocated {
        &self.string_located
    }

    /// Returns the current position.
    #[must_use]
    pub fn get_current_position(&self) -> usize {
        self.i
    }

    /// Eats all remaining text to the end.
    pub fn eat_all_to_end(&mut self) -> String {
        let result = self.string_located.get_string()[self.i..].to_string();
        self.i = self.string_located.length();
        result
    }

    /// Eats an expression and returns its value.
    ///
    /// Ported from `Eater.eatExpression`.
    pub fn eat_expression(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
    ) -> Result<TValue, EaterException> {
        let ch = self.peek_char();
        if ch == '{' || ch == '[' {
            let data = self.eat_all_to_end();
            let json = serde_json::from_str::<serde_json::Value>(&data)
                .map_err(|e| EaterException::new(e.to_string(), &self.string_located))?;
            return Ok(TValue::from_json(json));
        }
        let token_stack = self.eat_token_stack()?;
        token_stack.get_result(&self.string_located, context, memory)
    }

    /// Eats a token stack.
    ///
    /// Ported from `Eater.eatTokenStack`.
    pub fn eat_token_stack(&mut self) -> Result<TokenStack, EaterException> {
        let mut token_stack = TokenStack::new();
        self.add_into_token_stack(&mut token_stack, false)?;
        if token_stack.size() == 0 {
            return Err(EaterException::new("Missing expression", &self.string_located));
        }
        Ok(token_stack)
    }

    /// Eats an expression, stopping at a colon.
    ///
    /// Ported from `Eater.eatExpressionStopAtColon`.
    pub fn eat_expression_stop_at_colon(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
    ) -> Result<TValue, EaterException> {
        let mut token_stack = TokenStack::new();
        self.add_into_token_stack(&mut token_stack, true)?;
        token_stack.get_result(&self.string_located, context, memory)
    }

    fn add_into_token_stack(
        &mut self,
        token_stack: &mut TokenStack,
        stop_at_colon: bool,
    ) -> Result<(), EaterException> {
        let mut last_token: Option<Token> = None;
        loop {
            let token = TokenType::eat_one_token(last_token.as_ref(), self, stop_at_colon)?;
            let Some(token) = token else { return Ok(()) };
            let is_spaces = token.get_token_type() == TokenType::Spaces;
            token_stack.add(token.clone());
            if !is_spaces {
                last_token = Some(token);
            }
        }
    }

    /// Eats a quoted string and returns it.
    ///
    /// Ported from `Eater.eatAndGetQuotedString`.
    pub fn eat_and_get_quoted_string(&mut self) -> Result<String, EaterException> {
        let separator = self.peek_char();
        if !TLineType::is_quote(separator) {
            return Err(EaterException::new("quote10", &self.string_located));
        }
        self.check_and_eat_char(separator)?;
        let mut value = String::new();
        self.add_up_to(separator, &mut value);
        self.check_and_eat_char(separator)?;
        Ok(value)
    }

    /// Eats an optionally quoted string.
    ///
    /// Ported from `Eater.eatAndGetOptionalQuotedString`.
    pub fn eat_and_get_optional_quoted_string(&mut self) -> Result<String, EaterException> {
        let quote = self.peek_char();
        if TLineType::is_quote(quote) {
            return self.eat_and_get_quoted_string();
        }
        let mut value = String::new();
        let mut level = 0;
        loop {
            let ch = self.peek_char();
            if ch == '\0' {
                return Err(EaterException::new("until001", &self.string_located));
            }
            if level == 0 && (ch == ',' || ch == ')') {
                return Ok(value.trim().to_string());
            }
            let ch = self.eat_one_char();
            if ch == '(' {
                level += 1;
            } else if ch == ')' {
                level -= 1;
            }
            value.push(ch);
        }
    }

    /// Eats a number string.
    ///
    /// Ported from `Eater.eatAndGetNumber`.
    pub fn eat_and_get_number(&mut self) -> String {
        let mut result = String::new();
        loop {
            let ch = self.peek_char();
            if result.is_empty() && ch == '-' {
                result.push(self.eat_one_char());
                continue;
            }
            if ch == '\0' || !TLineType::is_latin_digit(ch) {
                return result;
            }
            result.push(self.eat_one_char());
        }
    }

    /// Eats spaces.
    ///
    /// Ported from `Eater.eatAndGetSpaces`.
    pub fn eat_and_get_spaces(&mut self) -> String {
        let mut result = String::new();
        loop {
            let ch = self.peek_char();
            if ch == '\0' || !TLineType::is_space_char(ch) {
                return result;
            }
            result.push(self.eat_one_char());
        }
    }

    /// Eats a variable name.
    ///
    /// Ported from `Eater.eatAndGetVarname`.
    pub fn eat_and_get_varname(&mut self) -> Result<String, EaterException> {
        let first = self.eat_one_char();
        let mut varname = String::from(first);
        if !TLineType::is_letter_or_emoji_or_underscore_or_dollar(first) {
            return Err(EaterException::new("a002", &self.string_located));
        }
        self.add_up_to_last_letter_or_emoji_or_underscore_or_digit(&mut varname);
        Ok(varname)
    }

    /// Eats a function name.
    ///
    /// Ported from `Eater.eatAndGetFunctionName`.
    pub fn eat_and_get_function_name(&mut self) -> Result<String, EaterException> {
        let first = self.eat_one_char();
        let mut varname = String::from(first);
        if !TLineType::is_letter_or_emoji_or_underscore_or_dollar(first) {
            return Err(EaterException::new("a003", &self.string_located));
        }
        self.add_up_to_last_letter_or_emoji_or_underscore_or_digit(&mut varname);
        Ok(varname)
    }

    /// Skips whitespace.
    pub fn skip_spaces(&mut self) {
        while self.i < self.string_located.length()
            && self.string_located.char_at(self.i).is_whitespace()
        {
            self.i += 1;
        }
    }

    /// Skips until the given character.
    pub fn skip_until_char(&mut self, ch: char) {
        while self.i < self.string_located.length() && self.string_located.char_at(self.i) != ch {
            self.i += 1;
        }
    }

    /// Peeks at the current character. Returns `\0` if at end.
    #[must_use]
    pub fn peek_char(&self) -> char {
        if self.i >= self.string_located.length() {
            return '\0';
        }
        self.string_located.char_at(self.i)
    }

    /// Peeks at the next character (i+1). Returns `\0` if at end.
    #[must_use]
    pub fn peek_char_n2(&self) -> char {
        if self.i + 1 >= self.string_located.length() {
            return '\0';
        }
        self.string_located.char_at(self.i + 1)
    }

    /// Returns `true` if there are more characters.
    #[must_use]
    pub fn has_next_char(&self) -> bool {
        self.i < self.string_located.length()
    }

    /// Eats one character and returns it.
    pub fn eat_one_char(&mut self) -> char {
        let ch = self.string_located.char_at(self.i);
        self.i += 1;
        ch
    }

    /// Checks and eats the given character. Throws if it doesn't match.
    pub fn check_and_eat_char(&mut self, ch: char) -> Result<(), EaterException> {
        if self.i >= self.string_located.length() || self.string_located.char_at(self.i) != ch {
            return Err(EaterException::new("a001", &self.string_located));
        }
        self.i += 1;
        Ok(())
    }

    /// Safely checks and eats the given character. Returns `false` if it doesn't match.
    pub fn safe_check_and_eat_char(&mut self, ch: char) -> Result<bool, EaterException> {
        if self.i >= self.string_located.length() || self.string_located.char_at(self.i) != ch {
            return Ok(false);
        }
        self.i += 1;
        Ok(true)
    }

    /// Optionally eats the given character.
    pub fn optionally_eat_char(&mut self, ch: char) {
        if self.i >= self.string_located.length() || self.string_located.char_at(self.i) != ch {
            return;
        }
        self.i += 1;
    }

    /// Checks and eats a string of characters.
    pub fn check_and_eat_str(&mut self, s: &str) -> Result<(), EaterException> {
        for ch in s.chars() {
            self.check_and_eat_char(ch)?;
        }
        Ok(())
    }

    /// Returns `true` if the remaining text matches an affectation pattern.
    ///
    /// Ported from `Eater.matchAffectation`.
    #[must_use]
    pub fn match_affectation(&self) -> bool {
        let s = self.string_located.get_string();
        let remaining = &s[self.i..];
        AFFECTATION_PATTERN.is_match(remaining)
    }

    fn add_up_to_last_letter_or_emoji_or_underscore_or_digit(&mut self, sb: &mut String) {
        while self.i < self.string_located.length() {
            let ch = self.string_located.char_at(self.i);
            if !TLineType::is_letter_or_emoji_or_underscore_or_digit(ch) {
                return;
            }
            self.i += 1;
            sb.push(ch);
        }
    }

    fn add_up_to(&mut self, separator: char, sb: &mut String) {
        while self.i < self.string_located.length() {
            let ch = self.peek_char();
            if ch == separator {
                return;
            }
            self.i += 1;
            sb.push(ch);
        }
    }

    /// Eats a function declaration.
    ///
    /// Ported from `Eater.eatDeclareFunction`.
    pub fn eat_declare_function(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        unquoted: bool,
        location: &StringLocated,
        allow_no_parenthesis: bool,
        ftype: TFunctionType,
    ) -> Result<TFunctionImpl, EaterException> {
        let args: Vec<super::t_function_argument::TFunctionArgument> = Vec::new();
        let function_name = self.eat_and_get_function_name()?;
        self.skip_spaces();
        if !self.safe_check_and_eat_char('(')? {
            if allow_no_parenthesis {
                return Ok(TFunctionImpl::new(function_name, args, unquoted, ftype));
            }
            return Err(EaterException::new("Missing opening parenthesis", &self.string_located));
        }
        let mut args = args;
        loop {
            self.skip_spaces();
            let ch = self.peek_char();
            if TLineType::is_letter_or_emoji_or_underscore_or_dollar(ch) {
                let varname = self.eat_and_get_varname()?;
                self.skip_spaces();
                let def_value = if self.peek_char() == '=' {
                    self.eat_one_char();
                    let mut def = TokenStack::eat_until_close_parenthesis_or_comma(self)?;
                    def.guess_functions(location)?;
                    Some(def.get_result(&self.string_located, context, memory)?)
                } else {
                    None
                };
                args.push(super::t_function_argument::TFunctionArgument::new(varname, def_value));
            } else if ch == ',' {
                self.check_and_eat_char(',')?;
            } else if ch == ')' {
                self.check_and_eat_char(')')?;
                break;
            } else {
                return Err(EaterException::new("Error in function definition", &self.string_located));
            }
        }
        self.skip_spaces();
        Ok(TFunctionImpl::new(function_name, args, unquoted, ftype))
    }

    /// Eats a return function declaration with optional inline return.
    ///
    /// Ported from `Eater.eatDeclareReturnFunctionWithOptionalReturn`.
    pub fn eat_declare_return_function_with_optional_return(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        unquoted: bool,
        location: &StringLocated,
    ) -> Result<TFunctionImpl, EaterException> {
        let mut result = self.eat_declare_function(
            context,
            memory,
            unquoted,
            location,
            false,
            TFunctionType::ReturnFunction,
        )?;
        if self.peek_char() == 'r' {
            self.check_and_eat_str("return")?;
            self.skip_spaces();
            let line = format!("!return {}", self.eat_all_to_end());
            result.add_body(StringLocated::new(line, location.get_location().clone()))?;
        } else if self.peek_char() == '!' {
            self.check_and_eat_str("!return")?;
            self.skip_spaces();
            let line = format!("!return {}", self.eat_all_to_end());
            result.add_body(StringLocated::new(line, location.get_location().clone()))?;
        }
        Ok(result)
    }

    /// Eats a procedure declaration.
    ///
    /// Ported from `Eater.eatDeclareProcedure`.
    pub fn eat_declare_procedure(
        &mut self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        unquoted: bool,
        location: &StringLocated,
    ) -> Result<TFunctionImpl, EaterException> {
        self.eat_declare_function(context, memory, unquoted, location, false, TFunctionType::Procedure)
    }
}

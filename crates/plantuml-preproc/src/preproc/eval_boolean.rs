//! Boolean expression evaluator for `!ifdef` / `!if` conditions.
//!
//! Ported from `net.sourceforge.plantuml.preproc.EvalBoolean`.

use crate::preproc::truth::Truth;

/// Evaluates boolean expressions like `FOO | !BAR & (BAZ | !QUX)`.
///
/// Ported from `net.sourceforge.plantuml.preproc.EvalBoolean`.
pub struct EvalBoolean<'a> {
    str: &'a str,
    pos: usize,
    ch: char,
    truth: &'a dyn Truth,
}

impl<'a> EvalBoolean<'a> {
    /// Creates a new `EvalBoolean` for the given expression.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.EvalBoolean.EvalBoolean`.
    #[must_use]
    pub fn new(str: &'a str, truth: &'a dyn Truth) -> Self {
        Self {
            str,
            pos: 0,
            ch: '\0',
            truth,
        }
    }

    /// Evaluates the expression and returns the boolean result.
    ///
    /// Returns an error string if the expression is malformed.
    /// Ported from `net.sourceforge.plantuml.preproc.EvalBoolean.eval`.
    pub fn eval(mut self) -> Result<bool, String> {
        self.next_char();
        let x = self.parse_expression()?;
        if self.pos < self.str.len() {
            return Err(format!("Unexpected: {}", self.ch));
        }
        Ok(x)
    }

    fn next_char(&mut self) {
        if self.pos < self.str.len() {
            self.ch = self.str.as_bytes()[self.pos] as char;
            self.pos += 1;
        } else {
            self.ch = '\0';
        }
    }

    fn eat(&mut self, char_to_eat: char) -> bool {
        while self.ch == ' ' {
            self.next_char();
        }
        if self.ch == char_to_eat {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn parse_expression(&mut self) -> Result<bool, String> {
        let mut x = self.parse_term()?;
        loop {
            if self.eat('|') {
                self.eat('|');
                x |= self.parse_term()?;
            } else {
                return Ok(x);
            }
        }
    }

    fn parse_term(&mut self) -> Result<bool, String> {
        let mut x = self.parse_factor()?;
        loop {
            if self.eat('&') {
                self.eat('&');
                x &= self.parse_factor()?;
            } else {
                return Ok(x);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<bool, String> {
        if self.eat('!') {
            return Ok(!self.parse_factor()?);
        }

        let x;
        let start_pos = self.pos - 1; // pos is already advanced by next_char
        if self.eat('(') {
            x = self.parse_expression()?;
            self.eat(')');
        } else if self.is_identifier() {
            while self.is_identifier() {
                self.next_char();
            }
            let func = &self.str[start_pos..self.pos - 1];
            x = self.truth.is_true(func);
        } else {
            return Err(format!("Unexpected: {}", self.ch));
        }

        Ok(x)
    }

    fn is_identifier(&self) -> bool {
        self.ch == '_' || self.ch == '$' || self.ch.is_alphanumeric()
    }
}

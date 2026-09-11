//! Math expression evaluator for `!if` conditions.
//!
//! Ported from `net.sourceforge.plantuml.preproc.EvalMath`.

/// Evaluates math expressions like `1 + 2 * 3`.
///
/// Ported from `net.sourceforge.plantuml.preproc.EvalMath`.
pub struct EvalMath<'a> {
    str: &'a str,
    pos: usize,
    ch: char,
}

impl<'a> EvalMath<'a> {
    /// Creates a new `EvalMath` for the given expression.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.EvalMath.EvalMath`.
    #[must_use]
    pub fn new(str: &'a str) -> Self {
        Self {
            str,
            pos: 0,
            ch: '\0',
        }
    }

    /// Evaluates the expression and returns the numeric result.
    ///
    /// Returns an error string if the expression is malformed.
    /// Ported from `net.sourceforge.plantuml.preproc.EvalMath.eval`.
    pub fn eval(mut self) -> Result<f64, String> {
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

    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut x = self.parse_term()?;
        loop {
            if self.eat('+') {
                x += self.parse_term()?;
            } else if self.eat('-') {
                x -= self.parse_term()?;
            } else {
                return Ok(x);
            }
        }
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut x = self.parse_factor()?;
        loop {
            if self.eat('*') {
                x *= self.parse_factor()?;
            } else if self.eat('/') {
                x /= self.parse_factor()?;
            } else {
                return Ok(x);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        if self.eat('+') {
            return self.parse_factor();
        }
        if self.eat('-') {
            return Ok(-self.parse_factor()?);
        }

        let x;
        let start_pos = self.pos - 1;
        if self.eat('(') {
            x = self.parse_expression()?;
            self.eat(')');
        } else if (self.ch >= '0' && self.ch <= '9') || self.ch == '.' {
            while (self.ch >= '0' && self.ch <= '9') || self.ch == '.' {
                self.next_char();
            }
            let num_str = &self.str[start_pos..self.pos - 1];
            x = num_str.parse::<f64>().map_err(|e| e.to_string())?;
        } else if self.ch >= 'a' && self.ch <= 'z' {
            while self.ch >= 'a' && self.ch <= 'z' {
                self.next_char();
            }
            let func = &self.str[start_pos..self.pos - 1];
            let _ = self.parse_factor()?;
            return Err(format!("Unknown function: {func}"));
        } else {
            return Err(format!("Unexpected: {}", self.ch));
        }

        Ok(x)
    }
}

//! Shunting-yard algorithm — converts infix tokens to RPN.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.ShuntingYard`.
//!
//! See: <https://en.wikipedia.org/wiki/Shunting-yard_algorithm>

use std::collections::VecDeque;

use crate::tim::eater_exception::EaterException;
use crate::StringLocated;

use super::knowledge::Knowledge;
use super::token::Token;
use super::token_iterator::TokenIterator;
use super::token_stack::TokenStack;
use super::token_type::TokenType;

/// Converts an infix token stream to Reverse Polish Notation.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.ShuntingYard`.
pub struct ShuntingYard {
    output_queue: TokenStack,
    operator_stack: VecDeque<Token>,
}

impl ShuntingYard {
    /// Creates a new `ShuntingYard`, processing all tokens from the iterator.
    ///
    /// Ported from `ShuntingYard(TokenIterator, Knowledge, StringLocated)`.
    pub fn new(
        it: &mut dyn TokenIterator,
        knowledge: &dyn Knowledge,
        location: &StringLocated,
    ) -> Result<Self, EaterException> {
        let mut sy = Self {
            output_queue: TokenStack::new(),
            operator_stack: VecDeque::new(),
        };

        while it.has_more_tokens() {
            let token = match it.next_token() {
                Some(t) => t,
                None => break,
            };

            let tt = token.get_token_type();

            if tt == TokenType::Number || tt == TokenType::QuotedString {
                sy.output_queue.add(token);
            } else if tt == TokenType::FunctionName {
                sy.operator_stack.push_front(token);
            } else if tt == TokenType::PlainText {
                let name = token.get_surface();
                match knowledge.get_variable(name)? {
                    Some(variable) => sy.output_queue.add(variable.to_token()),
                    None => {
                        if !Self::is_variable_name(name) {
                            return Err(EaterException::new(
                                format!("Parsing syntax error about {name}"),
                                &location,
                            ));
                        }
                        sy.output_queue
                            .add(Token::new(name.to_string(), TokenType::QuotedString, None));
                    }
                }
            } else if Self::is_operator_or_affectation(&token) {
                while (sy.there_is_a_function_at_the_top()
                    || sy.there_is_an_operator_at_the_top_with_greater_precedence(&token)
                    || sy.the_operator_at_the_top_has_equal_precedence_and_is_left_associative(&token))
                    && sy.the_operator_at_the_top_is_not_a_left_parenthesis()
                {
                    if let Some(op) = sy.operator_stack.pop_front() {
                        sy.output_queue.add(op);
                    }
                }
                sy.operator_stack.push_front(token);
            } else if tt == TokenType::OpenParenFunc {
                sy.operator_stack.push_front(token);
            } else if tt == TokenType::OpenParenMath {
                sy.operator_stack.push_front(token);
            } else if tt == TokenType::CloseParenFunc {
                while let Some(top) = sy.operator_stack.front() {
                    if top.get_token_type() == TokenType::OpenParenFunc {
                        break;
                    }
                    let op = sy.operator_stack.pop_front().unwrap_or_else(|| {
                        Token::new("", TokenType::PlainText, None)
                    });
                    sy.output_queue.add(op);
                }
                if let Some(first) = sy.operator_stack.pop_front() {
                    sy.output_queue.add(first);
                }
            } else if tt == TokenType::CloseParenMath {
                while let Some(top) = sy.operator_stack.front() {
                    if top.get_token_type() == TokenType::OpenParenMath {
                        break;
                    }
                    let op = sy.operator_stack.pop_front().unwrap_or_else(|| {
                        Token::new("", TokenType::PlainText, None)
                    });
                    sy.output_queue.add(op);
                }
                if sy
                    .operator_stack
                    .front()
                    .map_or(false, |t| t.get_token_type() == TokenType::OpenParenMath)
                {
                    sy.operator_stack.pop_front();
                }
            } else if tt == TokenType::Comma {
                while let Some(top) = sy.operator_stack.front() {
                    if top.get_token_type() == TokenType::OpenParenFunc {
                        break;
                    }
                    let op = sy.operator_stack.pop_front().unwrap_or_else(|| {
                        Token::new("", TokenType::PlainText, None)
                    });
                    sy.output_queue.add(op);
                }
            } else {
                return Err(EaterException::new(
                    format!("Unsupported token: {token}"),
                    &location,
                ));
            }
        }

        while let Some(token) = sy.operator_stack.pop_front() {
            sy.output_queue.add(token);
        }

        Ok(sy)
    }

    /// Returns the output queue (RPN token stack).
    #[must_use]
    pub fn get_queue(&self) -> TokenStack {
        self.output_queue.clone()
    }

    fn is_variable_name(name: &str) -> bool {
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '$' || c == '_')
            && !name.is_empty()
    }

    fn is_operator_or_affectation(token: &Token) -> bool {
        token.get_token_type() == TokenType::Operator
            || token.get_token_type() == TokenType::Affectation
    }

    fn there_is_a_function_at_the_top(&self) -> bool {
        self.operator_stack
            .front()
            .map_or(false, |t| t.get_token_type() == TokenType::FunctionName)
    }

    fn there_is_an_operator_at_the_top_with_greater_precedence(&self, token: &Token) -> bool {
        self.operator_stack.front().map_or(false, |top| {
            Self::is_operator_or_affectation(top) && top.get_precedence() > token.get_precedence()
        })
    }

    fn the_operator_at_the_top_has_equal_precedence_and_is_left_associative(
        &self,
        token: &Token,
    ) -> bool {
        self.operator_stack.front().map_or(false, |top| {
            Self::is_operator_or_affectation(top)
                && top.get_left_associativity()
                && top.get_precedence() == token.get_precedence()
        })
    }

    fn the_operator_at_the_top_is_not_a_left_parenthesis(&self) -> bool {
        // Java always returns true here (bug in original code — the check
        // for OPEN_PAREN_MATH returns true instead of false).
        true
    }
}

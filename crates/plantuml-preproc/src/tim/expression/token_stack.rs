//! `TokenStack` — a stack of tokens for expression evaluation.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.TokenStack`.

use std::collections::{VecDeque, HashMap};

use crate::tim::eater::Eater;
use crate::tim::eater_exception::EaterException;
use crate::tim::t_context::TContext;
use crate::tim::t_memory::TMemory;
use crate::StringLocated;

use super::reverse_polish_interpretor::ReversePolishInterpretor;
use super::shunting_yard::ShuntingYard;
use super::token::Token;
use super::token_iterator::{TokenIterator, TokenStackIterator};
use super::token_type::TokenType;

/// A stack of tokens that can be evaluated to a `TValue`.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.TokenStack`.
#[derive(Debug, Clone, Default)]
pub struct TokenStack {
    pub tokens: Vec<Token>,
}

impl TokenStack {
    /// Creates a new empty `TokenStack`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of tokens.
    #[must_use]
    pub fn size(&self) -> usize {
        self.tokens.len()
    }

    /// Returns a sub-token-stack starting at index `i`.
    ///
    /// Ported from `TokenStack.subTokenStack`.
    #[must_use]
    pub fn sub_token_stack(&self, i: usize) -> Self {
        Self {
            tokens: self.tokens[i..].to_vec(),
        }
    }

    /// Adds a token to the stack.
    pub fn add(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Returns a new `TokenStack` with all `SPACES` tokens removed.
    ///
    /// Ported from `TokenStack.withoutSpace`.
    #[must_use]
    pub fn without_space(&self) -> Self {
        let mut result = Self::new();
        for token in &self.tokens {
            if token.get_token_type() != TokenType::Spaces {
                result.add(token.clone());
            }
        }
        result
    }

    /// Eats tokens until a close parenthesis or comma at level 0.
    ///
    /// Ported from `TokenStack.eatUntilCloseParenthesisOrComma(Eater)`.
    pub fn eat_until_close_parenthesis_or_comma(
        eater: &mut Eater,
    ) -> Result<Self, EaterException> {
        let mut result = Self::new();
        let mut level = 0i32;
        let mut last_token: Option<Token> = None;
        loop {
            eater.skip_spaces();
            let ch = eater.peek_char();
            if ch == '\0' {
                return Err(EaterException::new("until001", eater.get_string_located()));
            }
            if level == 0 && (ch == ',' || ch == ')') {
                return Ok(result);
            }
            let Some(token) = TokenType::eat_one_token(last_token.as_ref(), eater, false)? else { continue };
            let tt = token.get_token_type();
            if tt == TokenType::OpenParenMath {
                level += 1;
            } else if tt == TokenType::CloseParenMath {
                level -= 1;
            }
            if token.get_token_type() != TokenType::Spaces {
                last_token = Some(token.clone());
            }
            result.add(token);
        }
    }

    /// Consumes tokens from the iterator until a close parenthesis or comma
    /// at level 0, or a `CLOSE_PAREN_FUNC`.
    ///
    /// Ported from `TokenStack.eatUntilCloseParenthesisOrComma(TokenIterator, StringLocated)`.
    pub fn eat_until_close_parenthesis_or_comma_it(
        it: &mut dyn TokenIterator,
        location: &StringLocated,
    ) -> Result<(), EaterException> {
        let mut level = 0i32;
        loop {
            let Some(ch) = it.peek_token() else { return Err(EaterException::new("until002", location)) };
            let typech = ch.get_token_type();
            if (level == 0
                && (typech == TokenType::Comma || typech == TokenType::CloseParenMath))
                || typech == TokenType::CloseParenFunc
            {
                return Ok(());
            }
            let Some(token) = it.next_token() else { return Err(EaterException::new("until002", location)) };
            let tt = token.get_token_type();
            if tt == TokenType::OpenParenMath || tt == TokenType::OpenParenFunc {
                level += 1;
            } else if tt == TokenType::CloseParenMath || tt == TokenType::CloseParenFunc {
                level -= 1;
            }
        }
    }

    /// Counts the number of function arguments in the token stream.
    ///
    /// Ported from `TokenStack.countFunctionArg`.
    #[allow(clippy::unused_self)]
    fn count_function_arg(
        &self,
        it: &mut dyn TokenIterator,
        location: &StringLocated,
    ) -> Result<i32, EaterException> {
        let type1 = match it.peek_token() {
            Some(t) => t.get_token_type(),
            None => return Err(EaterException::new("count12", location)),
        };
        if type1 == TokenType::CloseParenMath || type1 == TokenType::CloseParenFunc {
            return Ok(0);
        }
        let mut result = 1;
        while it.has_more_tokens() {
            Self::eat_until_close_parenthesis_or_comma_it(it, location)?;
            let Some(token) = it.next_token() else { return Err(EaterException::new("count12", location)) };
            let tt = token.get_token_type();
            if tt == TokenType::CloseParenMath || tt == TokenType::CloseParenFunc {
                return Ok(result);
            }
            if tt == TokenType::Comma {
                result += 1;
            } else {
                return Err(EaterException::new("count13", location));
            }
        }
        Err(EaterException::new("count12", location))
    }

    /// Converts `PLAIN_TEXT` tokens before `(` to `FUNCTION_NAME`,
    /// and reclassifies the parentheses as `OPEN_PAREN_FUNC`/`CLOSE_PAREN_FUNC`.
    ///
    /// Ported from `TokenStack.guessFunctions`.
    pub fn guess_functions(&mut self, location: &StringLocated) -> Result<(), EaterException> {
        let mut open: VecDeque<usize> = VecDeque::new();
        let mut parens: HashMap<usize, usize> = HashMap::new();
        for (i, token) in self.tokens.iter().enumerate() {
            if token.get_token_type() == TokenType::OpenParenMath {
                open.push_front(i);
            } else if token.get_token_type() == TokenType::CloseParenMath {
                if let Some(open_idx) = open.pop_front() {
                    parens.insert(open_idx, i);
                }
            }
        }
        for (&iopen, &iclose) in &parens {
            if iopen > 0 && self.tokens[iopen - 1].get_token_type() == TokenType::PlainText {
                self.tokens[iopen - 1] = self.tokens[iopen - 1].mute_to_function();
                let sub = self.sub_token_stack(iopen + 1);
                let mut it = sub.token_iterator();
                let nb_arg = self.count_function_arg(&mut it, location)?;
                self.tokens[iopen] = Token::new(nb_arg.to_string(), TokenType::OpenParenFunc, None);
                self.tokens[iclose] = Token::new(")", TokenType::CloseParenFunc, None);
            }
        }
        Ok(())
    }

    /// Returns a `TokenIterator` over this stack's tokens.
    pub fn token_iterator(&self) -> TokenStackIterator {
        TokenStackIterator::new(self.tokens.clone())
    }

    /// Evaluates the token stack to a `TValue`.
    ///
    /// Ported from `TokenStack.getResult`.
    pub fn get_result(
        &self,
        location: &StringLocated,
        context: &mut TContext,
        memory: &mut dyn TMemory,
    ) -> Result<super::t_value::TValue, EaterException> {
        let knowledge = context.as_knowledge(memory, location);
        let mut tmp = self.without_space();
        tmp.guess_functions(location)?;
        let mut it = tmp.token_iterator();
        let shunting_yard = ShuntingYard::new(&mut it, &knowledge, location)?;
        let queue = shunting_yard.get_queue();
        let rpn = ReversePolishInterpretor::new(location, &queue, &knowledge, memory, context)?;
        Ok(rpn.get_result())
    }
}

impl std::fmt::Display for TokenStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.tokens)
    }
}

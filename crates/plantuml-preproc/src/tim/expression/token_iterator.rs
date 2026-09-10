//! Token iterator interface.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.TokenIterator`.

use super::token::Token;

/// Iterator over tokens in an expression.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.TokenIterator`.
pub trait TokenIterator {
    /// Returns the next token, or `None` if exhausted.
    fn next_token(&mut self) -> Option<Token>;

    /// Peeks at the current token without consuming it, or `None` if exhausted.
    fn peek_token(&self) -> Option<&Token>;

    /// Returns `true` if there are more tokens to consume.
    fn has_more_tokens(&self) -> bool;
}

/// A simple iterator over a `Vec<Token>`.
///
/// This is the Rust equivalent of Java's `TokenStack.InternalIterator`.
pub struct TokenStackIterator {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStackIterator {
    /// Creates a new iterator over the given token vector.
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
}

impl TokenIterator for TokenStackIterator {
    fn next_token(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn has_more_tokens(&self) -> bool {
        self.pos < self.tokens.len()
    }
}

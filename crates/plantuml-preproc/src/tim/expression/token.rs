//! Token — a single token in the expression tokenizer.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.Token`.

use serde_json::Value;

use super::token_operator::TokenOperator;
use super::token_type::TokenType;

/// A token in the expression tokenizer.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.Token`.
#[derive(Debug, Clone)]
pub struct Token {
    surface: String,
    token_type: TokenType,
    json: Option<Value>,
}

impl Token {
    /// Creates a new token with the given surface text, type, and optional JSON.
    ///
    /// Ported from `Token(String, TokenType, JsonValue)`.
    #[must_use]
    pub fn new(surface: impl Into<String>, token_type: TokenType, json: Option<Value>) -> Self {
        Self {
            surface: surface.into(),
            token_type,
            json,
        }
    }

    /// Creates a new token from a single character.
    ///
    /// Ported from `Token(char, TokenType, JsonValue)`.
    #[must_use]
    pub fn from_char(ch: char, token_type: TokenType, json: Option<Value>) -> Self {
        Self::new(ch.to_string(), token_type, json)
    }

    /// Returns the surface text of this token.
    #[must_use]
    pub fn get_surface(&self) -> &str {
        &self.surface
    }

    /// Returns the token type.
    #[must_use]
    pub const fn get_token_type(&self) -> TokenType {
        self.token_type
    }

    /// Returns the operator for this token, or `None` if not an operator.
    ///
    /// Ported from `Token.getTokenOperator`.
    #[must_use]
    pub fn get_token_operator(&self) -> Option<TokenOperator> {
        if self.token_type != TokenType::Operator {
            return None;
        }
        let ch = self.surface.chars().next().unwrap_or('\0');
        let ch2 = if self.surface.chars().count() > 1 {
            self.surface.chars().nth(1).unwrap_or('\0')
        } else {
            '\0'
        };
        TokenOperator::get_token_operator(ch, ch2)
    }

    /// Mutes this `PLAIN_TEXT` token to a `FUNCTION_NAME` token.
    ///
    /// Ported from `Token.muteToFunction`.
    #[must_use]
    pub fn mute_to_function(&self) -> Token {
        Token::new(self.surface.clone(), TokenType::FunctionName, None)
    }

    /// Returns the JSON value of this `JSON_DATA` token.
    ///
    /// Ported from `Token.getJson`.
    #[must_use]
    pub fn get_json(&self) -> Option<&Value> {
        self.json.as_ref()
    }

    /// Returns the precedence of this token (for operators and affectations).
    ///
    /// Ported from `Token.getPrecedence`.
    #[must_use]
    pub fn get_precedence(&self) -> i32 {
        if self.token_type == TokenType::Affectation {
            return TokenOperator::Equals.get_precedence();
        }
        self.get_token_operator()
            .map_or(0, |op| op.get_precedence())
    }

    /// Returns `true` — all operators are left-associative.
    ///
    /// Ported from `Token.getLeftAssociativity`.
    #[must_use]
    pub const fn get_left_associativity(&self) -> bool {
        true
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{{{}}}", self.token_type, self.surface)
    }
}

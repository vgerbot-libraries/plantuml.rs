//! Token type enum and tokenizer.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.TokenType`.

use crate::tim::eater::Eater;
use crate::tim::eater_exception::EaterException;
use crate::TLineType;

use super::token::Token;
use super::token_operator::{TokenOperator, COMMERCIAL_MINUS_SIGN};

/// The type of an expression token.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.TokenType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// A quoted string, e.g. `"hello"` or `'hello'`.
    QuotedString,
    /// JSON data literal.
    JsonData,
    /// An operator like `+`, `-`, `*`, `/`, `<`, `==`, etc.
    Operator,
    /// Opening parenthesis in a math expression: `(`.
    OpenParenMath,
    /// A comma separator: `,`.
    Comma,
    /// Closing parenthesis in a math expression: `)`.
    CloseParenMath,
    /// A numeric literal, e.g. `42`.
    Number,
    /// Plain text — an identifier or unquoted text.
    PlainText,
    /// Whitespace.
    Spaces,
    /// A function name before `(`.
    FunctionName,
    /// Opening parenthesis for a function call: `(`.
    OpenParenFunc,
    /// Closing parenthesis for a function call: `)`.
    CloseParenFunc,
    /// An affectation operator: `=`.
    Affectation,
}

impl TokenType {
    /// Returns `true` if this token type is a single-char token
    /// (`OPEN_PAREN_MATH`, `COMMA`, `CLOSE_PAREN_MATH`).
    const fn is_single_char1(self) -> bool {
        matches!(
            self,
            Self::OpenParenMath | Self::Comma | Self::CloseParenMath
        )
    }

    /// Returns `true` if the character breaks plain text.
    ///
    /// Ported from `TokenType.isPlainTextBreak`.
    fn is_plain_text_break(ch: char, ch2: char) -> bool {
        let tt = Self::from_char(ch, ch2);
        tt.is_single_char1() || tt == Self::Operator || tt == Self::Spaces || tt == Self::Affectation
    }

    /// Classifies a character into a token type.
    ///
    /// Ported from `TokenType.fromChar`.
    fn from_char(ch: char, ch2: char) -> Self {
        if TLineType::is_quote(ch) {
            return Self::QuotedString;
        }
        if ch == '=' {
            return Self::Affectation;
        }
        if ch == '(' {
            return Self::OpenParenMath;
        }
        if ch == ')' {
            return Self::CloseParenMath;
        }
        if ch == ',' {
            return Self::Comma;
        }
        if TLineType::is_latin_digit(ch) {
            return Self::Number;
        }
        if TLineType::is_space_char(ch) {
            return Self::Spaces;
        }
        if ch == '-' || TokenOperator::get_token_operator(ch, ch2).is_some() {
            return Self::Operator;
        }
        Self::PlainText
    }

    /// Eats one token from the eater.
    ///
    /// Ported from `TokenType.eatOneToken`.
    ///
    /// Returns `Ok(None)` at end of input, or when a colon is encountered
    /// (if `manage_colon` is `true`).
    pub fn eat_one_token(
        last_token: Option<&Token>,
        eater: &mut Eater,
        manage_colon: bool,
    ) -> Result<Option<Token>, EaterException> {
        let mut ch = eater.peek_char();
        if ch == '\0' {
            return Ok(None);
        }

        if manage_colon && ch == ':' {
            return Ok(None);
        }

        if ch == '-' && Self::is_subtraction_operator(last_token) {
            ch = COMMERCIAL_MINUS_SIGN;
        }

        let token_operator = TokenOperator::get_token_operator(ch, eater.peek_char_n2());

        if TLineType::is_quote(ch) {
            let s = eater.eat_and_get_quoted_string()?;
            return Ok(Some(Token::new(s, Self::QuotedString, None)));
        }

        if let Some(_op) = token_operator {
            let display = TokenOperator::get_token_operator(ch, eater.peek_char_n2())
                .map_or(1, |op| op.get_display().chars().count());
            if display == 1 {
                eater.eat_one_char();
                return Ok(Some(Token::new(ch.to_string(), Self::Operator, None)));
            }
            let c1 = eater.eat_one_char();
            let c2 = eater.eat_one_char();
            return Ok(Some(Token::new(
                format!("{c1}{c2}"),
                Self::Operator,
                None,
            )));
        }

        if ch == '=' {
            eater.eat_one_char();
            return Ok(Some(Token::new(ch.to_string(), Self::Affectation, None)));
        }

        if ch == '(' {
            eater.eat_one_char();
            return Ok(Some(Token::new(ch.to_string(), Self::OpenParenMath, None)));
        }

        if ch == ')' {
            eater.eat_one_char();
            return Ok(Some(Token::new(ch.to_string(), Self::CloseParenMath, None)));
        }

        if ch == ',' {
            eater.eat_one_char();
            return Ok(Some(Token::new(ch.to_string(), Self::Comma, None)));
        }

        if TLineType::is_latin_digit(ch) || ch == '-' {
            let s = eater.eat_and_get_number();
            return Ok(Some(Token::new(s, Self::Number, None)));
        }

        if TLineType::is_space_char(ch) {
            let s = eater.eat_and_get_spaces();
            return Ok(Some(Token::new(s, Self::Spaces, None)));
        }

        let s = Self::eat_and_get_token_plain_text(eater)?;
        Ok(Some(Token::new(s, Self::PlainText, None)))
    }

    /// Returns `true` if the `-` after `last_token` should be treated as
    /// a subtraction operator (converted to `COMMERCIAL_MINUS_SIGN`).
    ///
    /// Ported from `TokenType.isSubtractionOperator`.
    fn is_subtraction_operator(last_token: Option<&Token>) -> bool {
        let Some(last_token) = last_token else { return false };
        let tt = last_token.get_token_type();
        !matches!(
            tt,
            Self::Operator | Self::OpenParenMath | Self::Comma | Self::Affectation
        )
    }

    /// Eats plain text from the eater until a break character.
    ///
    /// Ported from `TokenType.eatAndGetTokenPlainText`.
    fn eat_and_get_token_plain_text(eater: &mut Eater) -> Result<String, EaterException> {
        let mut result = String::new();
        loop {
            let ch = eater.peek_char();
            if ch == '\0' || Self::is_plain_text_break(ch, eater.peek_char_n2()) {
                return Ok(result);
            }
            result.push(eater.eat_one_char());
        }
    }
}

//! Token operators with precedence.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.TokenOperator`.
//!
//! See: <https://en.cppreference.com/w/c/language/operator_precedence>

use super::t_value::TValue;

/// The commercial minus sign `\u{2052}` used internally to distinguish
/// the subtraction operator from the unary minus sign.
///
/// Ported from `TokenType.COMMERCIAL_MINUS_SIGN`.
pub const COMMERCIAL_MINUS_SIGN: char = '\u{2052}';

/// An operator token with precedence and display string.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.TokenOperator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenOperator {
    /// `*` — precedence 97.
    Multiplication,
    /// `/` — precedence 97.
    Division,
    /// `+` — precedence 96.
    Addition,
    /// `\u{2052}` — precedence 96.
    Substraction,
    /// `<` — precedence 94.
    LessThan,
    /// `>` — precedence 94.
    GreaterThan,
    /// `<=` — precedence 94.
    LessThanOrEquals,
    /// `>=` — precedence 94.
    GreaterThanOrEquals,
    /// `==` — precedence 93.
    Equals,
    /// `!=` — precedence 93.
    NotEquals,
    /// `&&` — precedence 89.
    LogicalAnd,
    /// `||` — precedence 88.
    LogicalOr,
}

impl TokenOperator {
    /// Returns the operator matching the given characters, or `None`.
    ///
    /// Ported from `TokenOperator.getTokenOperator`.
    #[must_use]
    pub fn get_token_operator(ch: char, ch2: char) -> Option<Self> {
        match ch {
            '*' => Some(Self::Multiplication),
            '/' => Some(Self::Division),
            '+' => Some(Self::Addition),
            COMMERCIAL_MINUS_SIGN => Some(Self::Substraction),
            '<' => {
                if ch2 == '=' {
                    Some(Self::LessThanOrEquals)
                } else {
                    Some(Self::LessThan)
                }
            }
            '>' => {
                if ch2 == '=' {
                    Some(Self::GreaterThanOrEquals)
                } else {
                    Some(Self::GreaterThan)
                }
            }
            '=' => {
                if ch2 == '=' {
                    Some(Self::Equals)
                } else {
                    None
                }
            }
            '!' => {
                if ch2 == '=' {
                    Some(Self::NotEquals)
                } else {
                    None
                }
            }
            '&' => {
                if ch2 == '&' {
                    Some(Self::LogicalAnd)
                } else {
                    None
                }
            }
            '|' => {
                if ch2 == '|' {
                    Some(Self::LogicalOr)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns the precedence of this operator.
    #[must_use]
    pub const fn get_precedence(&self) -> i32 {
        match self {
            Self::Multiplication | Self::Division => 100 - 3,
            Self::Addition | Self::Substraction => 100 - 4,
            Self::LessThan
            | Self::GreaterThan
            | Self::LessThanOrEquals
            | Self::GreaterThanOrEquals => 100 - 6,
            Self::Equals | Self::NotEquals => 100 - 7,
            Self::LogicalAnd => 100 - 11,
            Self::LogicalOr => 100 - 12,
        }
    }

    /// Returns the display string for this operator.
    #[must_use]
    pub const fn get_display(&self) -> &'static str {
        match self {
            Self::Multiplication => "*",
            Self::Division => "/",
            Self::Addition => "+",
            Self::Substraction => "\u{2052}",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::LessThanOrEquals => "<=",
            Self::GreaterThanOrEquals => ">=",
            Self::Equals => "==",
            Self::NotEquals => "!=",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
        }
    }

    /// Applies this operator to two values.
    ///
    /// Ported from `TokenOperator.operate`.
    #[must_use]
    pub fn operate(&self, v1: &TValue, v2: &TValue) -> TValue {
        match self {
            Self::Multiplication => v1.multiply(v2),
            Self::Division => v1.divided_by(v2),
            Self::Addition => v1.add(v2),
            Self::Substraction => v1.minus(v2),
            Self::LessThan => v1.less_than(v2),
            Self::GreaterThan => v1.greater_than(v2),
            Self::LessThanOrEquals => v1.less_than_or_equals(v2),
            Self::GreaterThanOrEquals => v1.greater_than_or_equals(v2),
            Self::Equals => v1.equals_operation(v2),
            Self::NotEquals => v1.not_equals(v2),
            Self::LogicalAnd => v1.logical_and(v2),
            Self::LogicalOr => v1.logical_or(v2),
        }
    }
}

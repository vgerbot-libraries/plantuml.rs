//! TValue — the main value type for expression evaluation.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.TValue`.

use serde_json::Value;

use super::token::Token;
use super::token_type::TokenType;

/// A value that can be an integer, a string, or JSON.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.TValue`.
///
/// In Java, `TValue` has three nullable fields (`intValue`, `stringValue`,
/// `jsonValue`). In Rust, we use an enum for a cleaner representation.
/// The semantics match the Java original:
/// - `Int` → `intValue` set, others null
/// - `Str` → `stringValue` set, others null
/// - `Json` → `jsonValue` set, others null
#[derive(Debug, Clone, PartialEq)]
pub enum TValue {
    /// An integer value.
    Int(i32),
    /// A string value.
    Str(String),
    /// A JSON value.
    Json(Value),
}

impl TValue {
    /// Creates a `TValue` from an integer.
    ///
    /// Ported from `TValue.fromInt`.
    #[must_use]
    pub fn from_int(v: i32) -> Self {
        Self::Int(v)
    }

    /// Creates a `TValue` from a boolean (`true` → 1, `false` → 0).
    ///
    /// Ported from `TValue.fromBoolean`.
    #[must_use]
    pub fn from_boolean(b: bool) -> Self {
        Self::Int(if b { 1 } else { 0 })
    }

    /// Creates a `TValue` from a JSON value.
    ///
    /// Ported from `TValue.fromJson`.
    #[must_use]
    pub fn from_json(json: Value) -> Self {
        Self::Json(json)
    }

    /// Creates a `TValue` from a string.
    ///
    /// Ported from `TValue.fromString(String)`.
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::Str(s.into())
    }

    /// Creates a `TValue` from a `QUOTED_STRING` token.
    ///
    /// Ported from `TValue.fromString(Token)`.
    #[must_use]
    pub fn from_string_token(token: &Token) -> Self {
        Self::Str(token.get_surface().to_string())
    }

    /// Creates a `TValue` from a `NUMBER` token, parsing the surface as `i32`.
    ///
    /// Ported from `TValue.fromNumber`.
    ///
    /// # Panics
    /// Panics if the token surface is not a valid `i32`. This matches the
    /// Java behavior which throws `NumberFormatException`.
    pub fn from_number(token: &Token) -> Self {
        Self::Int(token.get_surface().parse::<i32>().unwrap_or(0))
    }

    /// Returns `true` if this is a number (not a string or JSON).
    ///
    /// Ported from `TValue.isNumber`.
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    /// Returns `true` if this is a string.
    ///
    /// Ported from `TValue.isString`.
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::Str(_))
    }

    /// Returns `true` if this is JSON.
    ///
    /// Ported from `TValue.isJson`.
    #[must_use]
    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json(_))
    }

    /// Returns the integer value.
    ///
    /// Ported from `TValue.toInt`.
    #[must_use]
    pub fn to_int(&self) -> i32 {
        match self {
            Self::Int(i) => *i,
            Self::Str(s) => s.parse().unwrap_or(0),
            Self::Json(Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Self::Json(Value::String(s)) => s.parse().unwrap_or(0),
            Self::Json(Value::Bool(b)) => i32::from(*b),
            _ => 0,
        }
    }

    /// Returns the boolean value: numbers are `true` if non-zero,
    /// strings are `true` if non-empty, JSON follows the same rules.
    ///
    /// Ported from `TValue.toBoolean`.
    #[must_use]
    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Int(i) => *i != 0,
            Self::Str(s) => !s.is_empty(),
            Self::Json(Value::Bool(b)) => *b,
            Self::Json(Value::Null) => false,
            Self::Json(j) => !j.to_string().is_empty(),
        }
    }

    /// Returns the string representation.
    ///
    /// Ported from `TValue.toString`.
    #[must_use]
    pub fn to_string_value(&self) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Str(s) => s.clone(),
            Self::Json(Value::String(s)) => s.clone(),
            Self::Json(j) => j.to_string(),
        }
    }

    /// Converts this value to a token.
    ///
    /// Ported from `TValue.toToken`.
    #[must_use]
    pub fn to_token(&self) -> Token {
        if self.is_number() {
            return Token::new(self.to_string_value(), TokenType::Number, None);
        }
        if self.is_json() {
            if let Self::Json(j) = self {
                return Token::new(self.to_string_value(), TokenType::JsonData, Some(j.clone()));
            }
        }
        Token::new(self.to_string_value(), TokenType::QuotedString, None)
    }

    /// Returns the JSON value, or `Null` if this is not JSON.
    ///
    /// Ported from `TValue.toJson`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Json(j) => j.clone(),
            _ => Value::Null,
        }
    }

    /// Returns the JSON representation of this value.
    ///
    /// Ported from `TValue.toJsonValue`.
    /// Numbers → `Json.value(int)`, strings → `Json.value(string)`, else `jsonValue`.
    #[must_use]
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::Int(i) => Value::from(*i),
            Self::Str(s) => Value::String(s.clone()),
            Self::Json(j) => j.clone(),
        }
    }

    // ---- Arithmetic operations ----

    /// Adds two values. Numbers add numerically; otherwise string concatenation.
    ///
    /// Ported from `TValue.add`.
    #[must_use]
    pub fn add(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_int(self.to_int() + v2.to_int());
        }
        TValue::from_string(format!("{}{}", self.to_string_value(), v2.to_string_value()))
    }

    /// Subtracts two values. Numbers subtract numerically; otherwise string concatenation.
    ///
    /// Ported from `TValue.minus`.
    #[must_use]
    pub fn minus(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_int(self.to_int() - v2.to_int());
        }
        TValue::from_string(format!("{}{}", self.to_string_value(), v2.to_string_value()))
    }

    /// Multiplies two values. Numbers multiply numerically; otherwise string concat with `*`.
    ///
    /// Ported from `TValue.multiply`.
    #[must_use]
    pub fn multiply(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_int(self.to_int() * v2.to_int());
        }
        TValue::from_string(format!("{}*{}", self.to_string_value(), v2.to_string_value()))
    }

    /// Divides two values. Numbers divide numerically; otherwise string concat with `/`.
    ///
    /// Ported from `TValue.dividedBy`.
    #[must_use]
    pub fn divided_by(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_int(self.to_int() / v2.to_int());
        }
        TValue::from_string(format!("{}/{}", self.to_string_value(), v2.to_string_value()))
    }

    // ---- Comparison operations ----

    /// Greater-than comparison. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.greaterThan`.
    #[must_use]
    pub fn greater_than(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_boolean(self.to_int() > v2.to_int());
        }
        TValue::from_boolean(self.to_string_value() > v2.to_string_value())
    }

    /// Greater-than-or-equals comparison. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.greaterThanOrEquals`.
    #[must_use]
    pub fn greater_than_or_equals(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_boolean(self.to_int() >= v2.to_int());
        }
        TValue::from_boolean(self.to_string_value() >= v2.to_string_value())
    }

    /// Less-than comparison. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.lessThan`.
    #[must_use]
    pub fn less_than(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_boolean(self.to_int() < v2.to_int());
        }
        TValue::from_boolean(self.to_string_value() < v2.to_string_value())
    }

    /// Less-than-or-equals comparison. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.lessThanOrEquals`.
    #[must_use]
    pub fn less_than_or_equals(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_boolean(self.to_int() <= v2.to_int());
        }
        TValue::from_boolean(self.to_string_value() <= v2.to_string_value())
    }

    /// Equality comparison. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.equalsOperation`.
    #[must_use]
    pub fn equals_operation(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_boolean(self.to_int() == v2.to_int());
        }
        TValue::from_boolean(self.to_string_value() == v2.to_string_value())
    }

    /// Not-equals comparison. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.notEquals`.
    #[must_use]
    pub fn not_equals(&self, v2: &TValue) -> TValue {
        if self.is_number() && v2.is_number() {
            return TValue::from_boolean(self.to_int() != v2.to_int());
        }
        TValue::from_boolean(self.to_string_value() != v2.to_string_value())
    }

    // ---- Logical operations ----

    /// Logical AND. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.logicalAnd`.
    #[must_use]
    pub fn logical_and(&self, v2: &TValue) -> TValue {
        TValue::from_boolean(self.to_boolean() && v2.to_boolean())
    }

    /// Logical OR. Returns a boolean `TValue`.
    ///
    /// Ported from `TValue.logicalOr`.
    #[must_use]
    pub fn logical_or(&self, v2: &TValue) -> TValue {
        TValue::from_boolean(self.to_boolean() || v2.to_boolean())
    }
}

impl std::fmt::Display for TValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{i}"),
            Self::Str(s) => write!(f, "{s}"),
            Self::Json(Value::String(s)) => write!(f, "{s}"),
            Self::Json(j) => write!(f, "{j}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_int() {
        assert_eq!(TValue::from_int(42), TValue::Int(42));
        assert!(TValue::from_int(42).is_number());
    }

    #[test]
    fn test_from_boolean() {
        assert_eq!(TValue::from_boolean(true), TValue::Int(1));
        assert_eq!(TValue::from_boolean(false), TValue::Int(0));
    }

    #[test]
    fn test_from_string() {
        let v = TValue::from_string("hello");
        assert!(v.is_string());
        assert_eq!(v.to_string_value(), "hello");
    }

    #[test]
    fn test_add_numbers() {
        assert_eq!(TValue::from_int(3).add(&TValue::from_int(4)), TValue::from_int(7));
    }

    #[test]
    fn test_add_strings() {
        assert_eq!(
            TValue::from_string("foo").add(&TValue::from_string("bar")),
            TValue::from_string("foobar")
        );
    }

    #[test]
    fn test_minus_numbers() {
        assert_eq!(TValue::from_int(10).minus(&TValue::from_int(3)), TValue::from_int(7));
    }

    #[test]
    fn test_multiply_numbers() {
        assert_eq!(TValue::from_int(6).multiply(&TValue::from_int(7)), TValue::from_int(42));
    }

    #[test]
    fn test_divided_by_numbers() {
        assert_eq!(TValue::from_int(20).divided_by(&TValue::from_int(4)), TValue::from_int(5));
    }

    #[test]
    fn test_to_boolean() {
        assert!(TValue::from_int(1).to_boolean());
        assert!(!TValue::from_int(0).to_boolean());
        assert!(TValue::from_string("x").to_boolean());
        assert!(!TValue::from_string("").to_boolean());
    }

    #[test]
    fn test_comparisons() {
        assert_eq!(TValue::from_int(5).greater_than(&TValue::from_int(3)), TValue::from_boolean(true));
        assert_eq!(TValue::from_int(3).greater_than(&TValue::from_int(5)), TValue::from_boolean(false));
        assert_eq!(TValue::from_int(3).less_than(&TValue::from_int(5)), TValue::from_boolean(true));
        assert_eq!(TValue::from_int(5).equals_operation(&TValue::from_int(5)), TValue::from_boolean(true));
        assert_eq!(TValue::from_int(5).not_equals(&TValue::from_int(3)), TValue::from_boolean(true));
    }

    #[test]
    fn test_logical() {
        assert_eq!(TValue::from_int(1).logical_and(&TValue::from_int(1)), TValue::from_boolean(true));
        assert_eq!(TValue::from_int(1).logical_and(&TValue::from_int(0)), TValue::from_boolean(false));
        assert_eq!(TValue::from_int(0).logical_or(&TValue::from_int(1)), TValue::from_boolean(true));
    }

    #[test]
    fn test_to_json_value() {
        assert_eq!(TValue::from_int(42).to_json_value(), Value::from(42));
        assert_eq!(TValue::from_string("hi").to_json_value(), Value::String("hi".into()));
    }
}

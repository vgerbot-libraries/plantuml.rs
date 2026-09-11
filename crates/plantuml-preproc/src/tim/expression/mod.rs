//! Expression evaluation module.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression` package.
//!
//! This module implements the expression tokenizer, shunting-yard algorithm,
//! and reverse Polish notation interpreter used by the `PlantUML` preprocessor.

#[allow(clippy::module_inception)]
pub mod expression;
pub mod knowledge;
pub mod reverse_polish_interpretor;
pub mod shunting_yard;
pub mod t_value;
pub mod token;
pub mod token_iterator;
pub mod token_operator;
pub mod token_stack;
pub mod token_type;

// Re-export key types for convenience.
pub use expression::Expression;
pub use knowledge::Knowledge;
pub use reverse_polish_interpretor::ReversePolishInterpretor;
pub use shunting_yard::ShuntingYard;
pub use t_value::TValue;
pub use token::Token;
pub use token_iterator::{TokenIterator, TokenStackIterator};
pub use token_operator::TokenOperator;
pub use token_stack::TokenStack;
pub use token_type::TokenType;

/// Type alias for JSON values, using `serde_json::Value`.
///
/// Ported from `net.sourceforge.plantuml.json.JsonValue`.
pub type JsonValue = serde_json::Value;

//! Reverse Polish Notation interpreter.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.ReversePolishInterpretor`.

use std::collections::{VecDeque, HashMap};

use crate::tim::eater_exception::EaterException;
use crate::tim::t_context::TContext;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::tim::t_memory::TMemory;
use crate::StringLocated;

use super::knowledge::Knowledge;
use super::t_value::TValue;
use super::token::Token;
use super::token_iterator::TokenIterator;
use super::token_stack::TokenStack;
use super::token_type::TokenType;

/// Evaluates a Reverse Polish Notation token queue to a `TValue`.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.ReversePolishInterpretor`.
pub struct ReversePolishInterpretor {
    result: TValue,
}

impl ReversePolishInterpretor {
    /// Creates a new `ReversePolishInterpretor`, evaluating the queue.
    ///
    /// Ported from `ReversePolishInterpretor(StringLocated, TokenStack, Knowledge, TMemory, TContext)`.
    pub fn new(
        location: &StringLocated,
        queue: &TokenStack,
        knowledge: &dyn Knowledge,
        memory: &mut dyn TMemory,
        context: &mut TContext,
    ) -> Result<Self, EaterException> {
        let mut named: HashMap<String, TValue> = HashMap::new();
        let mut stack: VecDeque<TValue> = VecDeque::new();
        let mut it = queue.token_iterator();

        while it.has_more_tokens() {
            let token = match it.next_token() {
                Some(t) => t,
                None => break,
            };
            let tt = token.get_token_type();

            if tt == TokenType::Number {
                stack.push_front(TValue::from_number(&token));
            } else if tt == TokenType::QuotedString {
                stack.push_front(TValue::from_string_token(&token));
            } else if tt == TokenType::JsonData {
                if let Some(json) = token.get_json() {
                    stack.push_front(TValue::from_json(json.clone()));
                }
            } else if tt == TokenType::Affectation {
                let v2 = stack.pop_front();
                let v1 = stack.pop_front();
                match (v1, v2) {
                    (Some(v1), Some(v2)) => {
                        named.insert(v1.to_string_value(), v2);
                    }
                    _ => return Err(EaterException::new("rpn42", &location)),
                }
            } else if tt == TokenType::Operator {
                let v2 = stack.pop_front();
                let v1 = stack.pop_front();
                let op = token.get_token_operator();
                let op = match op {
                    Some(o) => o,
                    None => return Err(EaterException::new("bad op", &location)),
                };
                match (v1, v2) {
                    (Some(v1), Some(v2)) => {
                        let tmp = op.operate(&v1, &v2);
                        stack.push_front(tmp);
                    }
                    _ => return Err(EaterException::new("rpn42", &location)),
                }
            } else if tt == TokenType::OpenParenFunc {
                let nb = token.get_surface().parse::<i32>().unwrap_or(0) - named.len() as i32;
                let token2 = match it.next_token() {
                    Some(t) => t,
                    None => return Err(EaterException::new("rpn43", &location)),
                };
                if token2.get_token_type() != TokenType::FunctionName {
                    return Err(EaterException::new("rpn43", &location));
                }
                let signature = TFunctionSignature::new(token2.get_surface(), nb);
                let function = match knowledge.get_function(&signature) {
                    Some(f) => f,
                    None => {
                        return Err(EaterException::new(
                            format!("Unknown built-in function {}", token2.get_surface()),
                            &location,
                        ));
                    }
                };
                let empty_set = std::collections::HashSet::new();
                if !function.can_cover(nb, &empty_set) {
                    return Err(EaterException::new(
                        format!(
                            "Bad number of arguments for {}",
                            function.get_signature().get_function_name()
                        ),
                        &location,
                    ));
                }
                let mut args: Vec<TValue> = Vec::new();
                for _ in 0..nb {
                    if let Some(v) = stack.pop_front() {
                        args.insert(0, v);
                    }
                }
                let r = function.execute_return_function(context, memory, location, &args, &named)?;
                named.clear();
                stack.push_front(r);
            } else {
                return Err(EaterException::new("rpn41", &location));
            }
        }

        let result = stack
            .pop_front()
            .unwrap_or_else(|| TValue::from_string(""));
        Ok(Self { result })
    }

    /// Returns the evaluation result.
    #[must_use]
    pub fn get_result(&self) -> TValue {
        self.result.clone()
    }
}

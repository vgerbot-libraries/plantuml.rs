//! Knowledge trait — interface for variable and function lookup.
//!
//! Ported from `net.sourceforge.plantuml.tim.expression.Knowledge`.

use crate::tim::eater_exception::EaterException;
use crate::tim::t_context::TContext;
use crate::tim::t_function::TFunction;
use crate::tim::t_function_signature::TFunctionSignature;
use crate::tim::t_memory::TMemory;
use crate::StringLocated;

use super::t_value::TValue;

/// Trait for looking up variables and functions during expression evaluation.
///
/// Ported from `net.sourceforge.plantuml.tim.expression.Knowledge`.
///
/// In Java, `getVariable` returns `null` for missing variables and
/// `getFunction` returns `null` for missing functions. In Rust, both
/// use `Option`.
pub trait Knowledge {
    /// Looks up a variable by name.
    ///
    /// Returns `Ok(None)` if the variable does not exist (Java returns `null`).
    fn get_variable(&self, name: &str) -> Result<Option<TValue>, EaterException>;

    /// Looks up a function by signature.
    ///
    /// Returns `None` if no matching function exists.
    fn get_function(&self, signature: &TFunctionSignature) -> Option<&dyn TFunction>;
}

/// A `Knowledge` implementation backed by a `TContext` and `TMemory`.
///
/// Ported from the anonymous `Knowledge` class created in `TContext.asKnowledge`.
///
/// This struct holds raw pointers to `TContext` and `TMemory` because the
/// `Knowledge` trait is used immutably (`&dyn Knowledge`) but `get_variable`
/// may need to call `TContext::from_json` which requires `&mut TContext`.
/// This mirrors the Java pattern where the anonymous class captures mutable
/// `this`. The pointers are valid for the duration of expression evaluation
/// (single-threaded, stack-owned).
///
/// The lifetime of the borrowed references is erased via `transmute` on the
/// raw pointers. This is safe because the caller (`TContext::as_knowledge`)
/// guarantees that both `context` and `memory` outlive the
/// `ContextKnowledge`, and no other mutable references exist during
/// expression evaluation (single-threaded).
pub struct ContextKnowledge {
    context: *mut TContext,
    memory: *mut dyn TMemory,
    location: StringLocated,
}

impl ContextKnowledge {
    /// Creates a new `ContextKnowledge` from raw pointers.
    ///
    /// # Safety
    /// The caller must ensure `context` and `memory` outlive this `ContextKnowledge`
    /// and that no other mutable references to them exist during its use.
    #[must_use]
    pub fn new(context: *mut TContext, memory: *mut dyn TMemory, location: StringLocated) -> Self {
        Self {
            context,
            memory,
            location,
        }
    }
}

impl Knowledge for ContextKnowledge {
    fn get_variable(&self, name: &str) -> Result<Option<TValue>, EaterException> {
        if name.contains('.') || name.contains('[') {
            // SAFETY: context and memory are valid for the duration of expression
            // evaluation. This mirrors the Java anonymous class capturing mutable `this`.
            let context = unsafe { &mut *self.context };
            let memory = unsafe { &mut *self.memory };
            let result = context.from_json(memory, name, &self.location);
            return Ok(Some(result));
        }
        // SAFETY: same as above — memory is valid for the duration.
        let memory = unsafe { &*self.memory };
        Ok(memory.get_variable(name))
    }

    fn get_function(&self, signature: &TFunctionSignature) -> Option<&dyn TFunction> {
        // SAFETY: context is valid for the duration of expression evaluation.
        let context = unsafe { &*self.context };
        context.get_function_smart(signature)
    }
}

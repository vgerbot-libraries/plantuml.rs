//! Execution context stacks for if/while/foreach.
//!
//! Ported from `net.sourceforge.plantuml.tim.ExecutionContexts`.

use std::collections::VecDeque;

use super::execution_context_foreach::ExecutionContextForeach;
use super::execution_context_if::ExecutionContextIf;
use super::execution_context_while::ExecutionContextWhile;

/// Base class providing stacks for if/while/foreach execution contexts.
///
/// Ported from `net.sourceforge.plantuml.tim.ExecutionContexts`.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContexts {
    all_ifs: VecDeque<ExecutionContextIf>,
    all_whiles: VecDeque<ExecutionContextWhile>,
    all_foreachs: VecDeque<ExecutionContextForeach>,
}

impl ExecutionContexts {
    /// Adds an if context.
    pub fn add_if(&mut self, value: ExecutionContextIf) {
        self.all_ifs.push_back(value);
    }

    /// Adds a while context.
    pub fn add_while(&mut self, value: ExecutionContextWhile) {
        self.all_whiles.push_back(value);
    }

    /// Adds a foreach context.
    pub fn add_foreach(&mut self, value: ExecutionContextForeach) {
        self.all_foreachs.push_back(value);
    }

    /// Peeks at the top if context.
    #[must_use]
    pub fn peek_if(&self) -> Option<&ExecutionContextIf> {
        self.all_ifs.back()
    }

    /// Peeks at the top while context.
    #[must_use]
    pub fn peek_while(&self) -> Option<&ExecutionContextWhile> {
        self.all_whiles.back()
    }

    /// Peeks at the top foreach context.
    #[must_use]
    pub fn peek_foreach(&self) -> Option<&ExecutionContextForeach> {
        self.all_foreachs.back()
    }

    /// Pops the top if context.
    pub fn poll_if(&mut self) -> Option<ExecutionContextIf> {
        self.all_ifs.pop_back()
    }

    /// Pops the top while context.
    pub fn poll_while(&mut self) -> Option<ExecutionContextWhile> {
        self.all_whiles.pop_back()
    }

    /// Pops the top foreach context.
    pub fn poll_foreach(&mut self) -> Option<ExecutionContextForeach> {
        self.all_foreachs.pop_back()
    }

    /// Returns `true` if all if conditions are satisfied.
    ///
    /// Ported from `ExecutionContexts.areAllIfOk`.
    #[must_use]
    pub fn are_all_if_ok(&self) -> bool {
        self.all_ifs.iter().all(|ctx| ctx.condition_is_ok_here())
    }
}

#![allow(clippy::missing_const_for_fn)]
//! `PlantUML` preprocessor crate.
//!
//! Ported from `net.sourceforge.plantuml.tim`, `net.sourceforge.plantuml.preproc`,
//! and `net.sourceforge.plantuml.preproc2` packages.

pub mod preproc;
pub mod preproc2;
pub mod stubs;
pub mod string_located;
pub mod t_line_type;
pub mod tim;
pub mod wasm_time;

// Re-export key types
pub use string_located::StringLocated;
pub use t_line_type::TLineType;
pub use tim::{
    Eater, EaterException, ExecutionContextForeach, ExecutionContextIf, ExecutionContextWhile,
    ExecutionContexts, FunctionsSet, JsonValue, Knowledge, TContext, TFunction, TFunctionArgument,
    TFunctionImpl, TFunctionSignature, TFunctionType, TMemory, TMemoryGlobal, TMemoryLocal,
    TimLoader, Token, TokenStack, TokenType, TValue, TVariableScope, Trie, TrieImpl,
};

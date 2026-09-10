//! Iterator module for preprocessor code iteration.
//!
//! Ported from `net.sourceforge.plantuml.tim.iterator` package.

pub mod abstract_code_iterator;
pub mod code_iterator;
pub mod code_iterator_affectation;
pub mod code_iterator_foreach;
pub mod code_iterator_if;
pub mod code_iterator_impl;
pub mod code_iterator_inner_comment;
pub mod code_iterator_legacy_define;
pub mod code_iterator_long_comment;
pub mod code_iterator_procedure;
pub mod code_iterator_return_function;
pub mod code_iterator_short_comment;
pub mod code_iterator_sub;
pub mod code_iterator_while;
pub mod code_position;

pub use code_iterator::CodeIterator;
pub use code_iterator_impl::{CodeIteratorImpl, Position};
pub use code_position::CodePosition;

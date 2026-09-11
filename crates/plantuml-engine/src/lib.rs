#![allow(
    clippy::suboptimal_flops,
    clippy::imprecise_flops,
    clippy::missing_const_for_fn,
    clippy::manual_midpoint,
    clippy::items_after_statements,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::match_same_arms,
    clippy::implicit_hasher,
    clippy::branches_sharing_code,
    clippy::if_same_then_else,
    clippy::manual_strip,
    clippy::manual_memcpy,
    clippy::explicit_iter_loop,
    clippy::too_many_arguments,
    clippy::collection_is_never_read,
)]
//! plantuml-engine — the main `PlantUML` engine.
//!
//! Ported from `net.sourceforge.plantuml` package.
//!
//! This crate provides the entry point for parsing and rendering `PlantUML`
//! diagrams. For Phase 4, only PREPROC format is supported.

pub mod block_uml;
pub mod block_uml_builder;
pub mod definitions_container;
pub mod error_uml;
pub mod source_string_reader;
pub mod start_utils;
pub mod sequence_renderer;
pub mod render;

pub use block_uml::BlockUml;
pub use block_uml_builder::BlockUmlBuilder;
pub use definitions_container::DefinitionsContainer;
pub use error_uml::{ErrorUml, ErrorUmlType};
pub use source_string_reader::SourceStringReader;
pub use start_utils::StartUtils;
pub use render::{render, render_svg, render_preproc, RenderError};

//! Preprocessor2 module — read filters and preprocessor pipeline.
//!
//! Ported from `net.sourceforge.plantuml.preproc2`.

pub mod preprocessor;
pub mod preprocessor_include_strategy;
pub mod preprocessor_utils;
pub mod read_filter;
pub mod read_filter_add_config;
pub mod read_filter_and;
pub mod read_filter_merge_lines;
pub mod read_filter_quote_comment;

// Re-export key types
pub use preprocessor::Preprocessor;
pub use preprocessor_include_strategy::PreprocessorIncludeStrategy;
pub use preprocessor_utils::PreprocessorUtils;
pub use read_filter::ReadFilter;
pub use read_filter_add_config::ReadFilterAddConfig;
pub use read_filter_and::ReadFilterAnd;
pub use read_filter_merge_lines::ReadFilterMergeLines;
pub use read_filter_quote_comment::ReadFilterQuoteComment;

//! `PlantUML` command framework — Command pattern with regex-based parsing.
//!
//! Ported from: `net/sourceforge/plantuml/command/` package.

pub mod stubs;

pub mod command;
pub mod command_control;
pub mod command_execution_result;
pub mod command_multilines;
pub mod multilines_strategy;
pub mod parser_pass;
pub mod single_line_command;
pub mod trim;
pub mod ubrex_command_multilines;
pub mod ubrex_single_line_command;

pub use command::Command;
pub use command_control::CommandControl;
pub use command_execution_result::CommandExecutionResult;
pub use command_multilines::CommandMultilines2;
pub use multilines_strategy::MultilinesStrategy;
pub use parser_pass::ParserPass;
pub use single_line_command::SingleLineCommand2;
pub use trim::Trim;
pub use ubrex_command_multilines::UbrexCommandMultilines2;
pub use stubs::{BlocLines, NoSuchColorException, StringLocated};

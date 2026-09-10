//! Parser pass enum — controls multi-pass parsing.
//!
//! Ported from: `net/sourceforge/plantuml/command/ParserPass.java`

/// Controls which pass a command is eligible for.
///
/// Most commands are eligible for `One` only. Some commands need to run
/// in a second or third pass after other commands have been processed.
///
/// Ported from: `net/sourceforge/plantuml/command/ParserPass.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParserPass {
    /// First pass — most commands run here.
    One,
    /// Second pass — runs after all `One` commands.
    Two,
    /// Third pass — runs after all `Two` commands.
    Three,
}

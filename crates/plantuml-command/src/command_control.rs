//! Command control enum — tri-state result of command validation.
//!
//! Ported from: `net/sourceforge/plantuml/command/CommandControl.java`

/// Result of checking whether a command matches input lines.
///
/// Ported from: `net/sourceforge/plantuml/command/CommandControl.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandControl {
    /// Command fully matches the input.
    Ok,
    /// Command does not match the input.
    NotOk,
    /// Multi-line command partially matched — more lines needed.
    OkPartial,
}

//! MergeStrategy — how to merge two style declarations.
//!
//! Ported from: `net/sourceforge/plantuml/style/MergeStrategy.java`

/// Strategy for merging style values.
///
/// Ported from: `net/sourceforge/plantuml/style/MergeStrategy.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    KeepExistingValueOfStereotype,
    OverwriteExistingValue,
}

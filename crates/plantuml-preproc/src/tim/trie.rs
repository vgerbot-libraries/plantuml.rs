//! Trie trait for prefix matching.
//!
//! Ported from `net.sourceforge.plantuml.tim.Trie`.

/// A trie for efficient prefix matching of strings.
///
/// Ported from `net.sourceforge.plantuml.tim.Trie`.
pub trait Trie {
    /// Adds a string to the trie.
    fn add(&mut self, s: &str);

    /// Returns the longest match starting at the given position in `s`.
    fn get_longuest_match_starting_in(&self, s: &str, pos: usize) -> String;
}

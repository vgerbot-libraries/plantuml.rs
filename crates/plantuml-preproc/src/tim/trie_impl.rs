//! Trie implementation for prefix matching.
//!
//! Ported from `net.sourceforge.plantuml.tim.TrieImpl`.

use std::collections::HashMap;

use super::trie::Trie;

/// A trie implementation using a HashMap of character → child nodes.
///
/// Ported from `net.sourceforge.plantuml.tim.TrieImpl`.
#[derive(Debug, Clone, Default)]
pub struct TrieImpl {
    brothers: HashMap<char, TrieImpl>,
}

impl TrieImpl {
    /// Creates a new empty `TrieImpl`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes a string from the trie. Returns `true` if the string was found.
    ///
    /// Ported from `TrieImpl.remove`.
    pub fn remove(&mut self, s: &str) -> bool {
        self.remove_internal(&format!("{s}\0"))
    }

    fn remove_internal(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() <= 1 {
            return false;
        }
        let mut current = self;
        for i in 0..chars.len() {
            let first = chars[i];
            let child = match current.brothers.get_mut(&first) {
                Some(c) => c,
                None => return false,
            };
            if i == chars.len() - 2 {
                return child.brothers.remove(&'\0').is_some();
            }
            current = child;
        }
        false
    }

    fn get_or_create(&mut self, added: char) -> &mut TrieImpl {
        self.brothers.entry(added).or_default()
    }
}

impl Trie for TrieImpl {
    fn add(&mut self, s: &str) {
        if s.contains('\0') {
            return;
        }
        let full = format!("{s}\0");
        let mut current = self;
        for ch in full.chars() {
            current = current.get_or_create(ch);
        }
    }

    fn get_longuest_match_starting_in(&self, s: &str, pos: usize) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        let mut current = self;
        let mut idx = pos;

        loop {
            if idx >= chars.len() {
                if current.brothers.contains_key(&'\0') {
                    return result;
                }
                return String::new();
            }
            let ch = chars[idx];
            let child = match current.brothers.get(&ch) {
                Some(c) => c,
                None => {
                    if current.brothers.contains_key(&'\0') {
                        return result;
                    }
                    return String::new();
                }
            };
            if child.brothers.is_empty() {
                if current.brothers.contains_key(&'\0') {
                    return result;
                }
                return String::new();
            }
            result.push(ch);
            current = child;
            idx += 1;
        }
    }
}

//! Removes lines starting with emoji characters.
//!
//! Ported from `net.sourceforge.plantuml.preproc.CommentEmoji`.

use crate::StringLocated;

/// Filters out lines that start with emoji or symbol characters.
///
/// Ported from `net.sourceforge.plantuml.preproc.CommentEmoji`.
pub struct CommentEmoji;

impl CommentEmoji {
    /// Removes lines whose first code point falls in known emoji/symbol Unicode blocks.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.CommentEmoji.remove`.
    #[must_use]
    pub fn remove(input: &[StringLocated]) -> Vec<StringLocated> {
        let mut result = Vec::new();
        for sl in input {
            let line = sl.get_string();
            if !line.is_empty() {
                if let Some(first_cp) = line.chars().next() {
                    let num_block = (first_cp as u32) / 256;
                    if num_block == 0x26
                        || num_block == 0x27
                        || num_block == 0x1f3
                        || num_block == 0x1f4
                        || num_block == 0x1f5
                        || num_block == 0x1f6
                        || num_block == 0x1f9
                    {
                        continue;
                    }
                }
            }
            result.push(sl.clone());
        }
        result
    }
}

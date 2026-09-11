//! Bitmask-based character set for ranges 32–128 (case-insensitive).
//!
//! Uses two `u64` bitmasks to cover the 97 possible character offsets
//! (char code 32 through 128, offset 0 through 96).
//!
//! Ported from: `com/plantuml/ubrex/CharSet.java`
use super::case_mode::CaseMode;

pub struct CharSet {
    mask1: u64,
    mask2: u64,
}

impl Default for CharSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CharSet {
    pub const fn new() -> Self {
        Self {
            mask1: 0,
            mask2: 0,
        }
    }

    /// Adds a single character to the set (case-insensitive).
    pub fn add_char(&mut self, ch: char) {
        let ch = CaseMode::ensure_lowercase(ch);
        assert!((' '..='\u{80}').contains(&ch), "Bad char: {ch}");
        let offset = ch as u32 - 32;
        if offset < 64 {
            self.mask1 |= 1u64 << offset;
        } else {
            self.mask2 |= 1u64 << (offset - 64);
        }
    }

    /// Adds a range of characters to the set (case-insensitive).
    pub fn add_range(&mut self, start: char, end: char) {
        assert!(start <= end, "Invalid range: '{start}' is greater than '{end}'.");
        assert!(!(start < ' ' || end > '\u{80}'), "Characters must be in the range 32 to 128.");
        let start = CaseMode::ensure_lowercase(start);
        let end = CaseMode::ensure_lowercase(end);

        let start_offset = (start as u32 - 32) as i32;
        let end_offset = (end as u32 - 32) as i32;

        if end_offset < 64 {
            // Entire range in mask1
            let length = end_offset - start_offset + 1;
            let range_mask = if length == 64 {
                u64::MAX
            } else {
                ((1u64 << length) - 1) << start_offset
            };
            self.mask1 |= range_mask;
        } else if start_offset >= 64 {
            // Entire range in mask2
            let adjusted_start = start_offset - 64;
            let adjusted_end = end_offset - 64;
            let length = adjusted_end - adjusted_start + 1;
            let range_mask = ((1u64 << length) - 1) << adjusted_start;
            self.mask2 |= range_mask;
        } else {
            // Range spans both masks
            let length1 = 64 - start_offset;
            let mask_part1 = if length1 == 64 {
                u64::MAX
            } else {
                ((1u64 << length1) - 1) << start_offset
            };
            self.mask1 |= mask_part1;

            let adjusted_end = end_offset - 64;
            let length2 = adjusted_end + 1;
            let mask_part2 = (1u64 << length2) - 1;
            self.mask2 |= mask_part2;
        }
    }

    /// Returns `true` if `ch` is in the set (case-insensitive).
    pub fn contains(&self, ch: char) -> bool {
        let ch = CaseMode::ensure_lowercase(ch);
        let offset = (ch as u32 - 32) as i32;
        if offset > 96 {
            return false;
        }
        if offset < 0 {
            return false;
        }
        if offset < 64 {
            (self.mask1 & (1u64 << offset)) != 0
        } else {
            (self.mask2 & (1u64 << (offset - 64))) != 0
        }
    }
}

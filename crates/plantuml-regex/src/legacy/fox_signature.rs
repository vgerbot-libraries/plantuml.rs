//! Fox signature — fast-reject bitmask for regex pattern matching.
//!
//! Ported from `net.sourceforge.plantuml.text.FoxSignature` (Java).
//!
//! The fox signature is a 64-bit bitmask where each bit represents a character
//! class. By computing the signature of both a regex pattern and an input
//! string, we can quickly reject inputs that cannot possibly match: if
//! `sig(regex) & sig(input) != sig(regex)`, the input lacks a required
//! character and cannot match.

use std::sync::LazyLock;

/// Internal data holding the precomputed mask table.
struct FoxSignatureData {
    masks: [u64; 127],
    mask_spaces: u64,
    mask_special1: u64,
}

static FOX_DATA: LazyLock<FoxSignatureData> = LazyLock::new(|| {
    let mut masks = [0u64; 127];

    // The full string of characters that get individual bit positions.
    // Ported from Java: "ABCDEFGHIJKLMNOPQRSTUVWXYZ0!\"#$%&'()*+,-./:;<=>?@[\\]^_{|}~"
    let full = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0!\"#$%&'()*+,-./:;<=>?@[\\]^_{|}~";

    let mut m: u64 = 1;
    let mask_spaces = m;
    m <<= 1;
    let mask_special1 = m;
    m <<= 1;

    for ch in full.chars() {
        let idx = ch as usize;
        if idx < 127 {
            masks[idx] = m;
        }
        // Uppercase letters also set the same bit for their lowercase equivalent.
        if ch.is_ascii_uppercase() {
            let lower = (ch as u8 + (b'a' - b'A')) as usize;
            if lower < 127 {
                masks[lower] = m;
            }
        } else if ch == '.' || ch == '=' || ch == '-' || ch == '~' {
            masks[idx] |= mask_special1;
        }
        m <<= 1;
    }

    // Whitespace characters all map to MASK_SPACES.
    masks[b' ' as usize] = mask_spaces;
    masks[b'\t' as usize] = mask_spaces;
    masks[b'\r' as usize] = mask_spaces;
    masks[b'\n' as usize] = mask_spaces;
    masks[b'\x0C' as usize] = mask_spaces; // form feed

    FoxSignatureData {
        masks,
        mask_spaces,
        mask_special1,
    }
});

/// Returns the mask for a single character.
///
/// Ported from `FoxSignature.getMask(char)`.
fn get_mask(ch: char) -> u64 {
    let data = &*FOX_DATA;
    let idx = ch as u32;
    if (idx as usize) < data.masks.len() {
        data.masks[idx as usize]
    } else if idx == 0x00A0 {
        // Non-breaking space maps to MASK_SPACES.
        data.mask_spaces
    } else {
        0
    }
}

/// Returns the special spaces mask (bit 0).
///
/// Ported from `FoxSignature.getSpecialSpaces()`.
pub fn get_special_spaces() -> u64 {
    FOX_DATA.mask_spaces
}

/// Returns the special1 mask (bit 1), used for `.=-~`.
///
/// Ported from `FoxSignature.getSpecial1()`.
pub fn get_special1() -> u64 {
    FOX_DATA.mask_special1
}

/// Computes the fox signature from a real (non-regex) string.
///
/// Ported from `FoxSignature.getFoxSignatureFromRealString(String)`.
pub fn get_fox_signature_from_real_string(s: &str) -> u64 {
    let mut result = 0u64;
    for ch in s.chars() {
        result |= get_mask(ch);
    }
    result
}

/// Computes the fox signature from a regex pattern string.
///
/// Ported from `FoxSignature.getFoxSignatureFromRegex(String)`.
///
/// The algorithm skips:
/// - `.` followed by `+` or `*` (matches anything, no required char)
/// - `\b` (word boundary, no char)
/// - Characters followed by `?` or `*` (optional, might not appear)
///
/// Characters followed by `+` (one-or-more) are included since they must
/// appear at least once.
pub fn get_fox_signature_from_regex(s: &str) -> u64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = 0u64;
    let mut i = 0;
    while i < len {
        match chars[i] {
            '.' => {
                // Dot matches anything; skip quantifier if present.
                if i + 1 < len && (chars[i + 1] == '+' || chars[i + 1] == '*') {
                    i += 1;
                }
                // Java throws if '.' is not followed by + or *, but we skip silently.
            }
            '\\' => {
                // Escape sequence.
                if i + 1 < len && chars[i + 1] == 'b' {
                    // \b word boundary — skip both chars.
                    i += 1;
                }
                // Java throws for \ followed by letter/digit, but we skip silently.
                // For \ followed by other chars (e.g. \. \[ \]), the next char
                // is processed normally in the next iteration.
            }
            _ => {
                if i + 1 < len && (chars[i + 1] == '?' || chars[i + 1] == '*') {
                    // Optional or zero-or-more: the char might not appear, skip both.
                    i += 1;
                } else {
                    result |= get_mask(chars[i]);
                    if i + 1 < len && chars[i + 1] == '+' {
                        // One-or-more quantifier: skip it.
                        i += 1;
                    }
                }
            }
        }
        i += 1;
    }
    result
}

/// Converts a fox signature bitmask back to a string of the characters it
/// represents (for debugging).
///
/// Ported from `FoxSignature.backToString(long)`.
pub fn back_to_string(check: u64) -> String {
    let data = &*FOX_DATA;
    let mut sb = String::new();
    for i in 0..data.masks.len() {
        if data.masks[i] != 0 && (check & data.masks[i]) != 0 {
            sb.push(char::from_u32(i as u32).unwrap_or('?'));
        }
    }
    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_masks() {
        assert_eq!(get_special_spaces(), 1);
        assert_eq!(get_special1(), 2);
    }

    #[test]
    fn test_real_string_signature() {
        // 'A' should have a non-zero mask.
        let sig = get_fox_signature_from_real_string("ABC");
        assert_ne!(sig, 0);
        // 'a' should have the same mask as 'A'.
        let sig_upper = get_fox_signature_from_real_string("A");
        let sig_lower = get_fox_signature_from_real_string("a");
        assert_eq!(sig_upper, sig_lower);
    }

    #[test]
    fn test_spaces_signature() {
        let sig = get_fox_signature_from_real_string(" \t\n");
        assert_eq!(sig, get_special_spaces());
    }

    #[test]
    fn test_nbsp_maps_to_spaces() {
        let sig = get_fox_signature_from_real_string("\u{00A0}");
        assert_eq!(sig, get_special_spaces());
    }

    #[test]
    fn test_regex_signature_skips_optional() {
        // 'a?' should not contribute to the signature (optional).
        let sig = get_fox_signature_from_regex("a?");
        assert_eq!(sig, 0);
        // 'a*' should not contribute either.
        let sig = get_fox_signature_from_regex("a*");
        assert_eq!(sig, 0);
    }

    #[test]
    fn test_regex_signature_includes_required() {
        // 'a+' should contribute.
        let sig = get_fox_signature_from_regex("a+");
        let expected = get_fox_signature_from_real_string("a");
        assert_eq!(sig, expected);
        // Plain 'a' should contribute.
        let sig = get_fox_signature_from_regex("a");
        assert_eq!(sig, expected);
    }

    #[test]
    fn test_regex_signature_dot() {
        // '.+' and '.*' should not contribute.
        assert_eq!(get_fox_signature_from_regex(".+"), 0);
        assert_eq!(get_fox_signature_from_regex(".*"), 0);
    }

    #[test]
    fn test_back_to_string() {
        let sig = get_fox_signature_from_real_string("A");
        let s = back_to_string(sig);
        assert!(s.contains('A') || s.contains('a'));
    }
}

/// Navigator over a character sequence with reverse/search/jump support for
/// lookbehind and parsing.
///
/// The Java original wraps a `CharSequence` and shares it across derived
/// navigators (sub-sequence, reverse, copy).  In Rust we store the characters
/// in an `Rc<Vec<char>>` so that derived navigators are cheap to create.
///
/// `p1` and `p2` are `isize` to match Java's signed `int` semantics: `jump`
/// can make `p2 < p1` in reversed mode, producing a negative `length()`.
///
/// Ported from: `com/plantuml/ubrex/TextNavigator.java`

use std::any::Any;
use std::fmt;
use std::rc::Rc;

use super::challenge::Challenge;

pub struct TextNavigator {
    chars: Rc<Vec<char>>,
    p1: isize,
    p2: isize,
    reversed: bool,
}

impl TextNavigator {
    /// Creates a navigator over the full `content`.
    pub fn build(content: &str) -> TextNavigator {
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len() as isize;
        TextNavigator {
            chars: Rc::new(chars),
            p1: 0,
            p2: len,
            reversed: false,
        }
    }

    /// Creates a reversed navigator covering characters `[0, p1 + pos)`.
    pub fn reverse(&self, pos: usize) -> TextNavigator {
        if self.reversed {
            panic!("Cannot reverse an already reversed navigator");
        }
        TextNavigator {
            chars: Rc::clone(&self.chars),
            p1: 0,
            p2: self.p1 + pos as isize,
            reversed: true,
        }
    }

    /// Returns a deep copy of this navigator.
    pub fn copy(&self) -> TextNavigator {
        TextNavigator {
            chars: Rc::clone(&self.chars),
            p1: self.p1,
            p2: self.p2,
            reversed: self.reversed,
        }
    }

    /// Returns the index of the first occurrence of `ch`, or `None`.
    pub fn index_of(&self, ch: char) -> Option<usize> {
        for i in 0..self.length() {
            if self.char_at(i) == ch {
                return Some(i);
            }
        }
        None
    }

    /// Returns a sub-sequence navigator `[begin, end)`.
    pub fn sub_sequence(&self, begin: usize, end: usize) -> TextNavigator {
        if begin > end || end > self.length() {
            panic!(
                "subSequence({}, {}) out of bounds for length {}",
                begin,
                end,
                self.length()
            );
        }
        if self.reversed {
            TextNavigator {
                chars: Rc::clone(&self.chars),
                p1: self.p2 - end as isize,
                p2: self.p2 - begin as isize,
                reversed: self.reversed,
            }
        } else {
            TextNavigator {
                chars: Rc::clone(&self.chars),
                p1: self.p1 + begin as isize,
                p2: self.p1 + end as isize,
                reversed: self.reversed,
            }
        }
    }

    /// Returns the length of the visible window.
    /// May be negative when `jump` has shrunk the window past `p1` in reversed
    /// mode, matching Java's signed `int` arithmetic.
    pub fn length(&self) -> usize {
        let len = self.p2 - self.p1;
        if len < 0 {
            0
        } else {
            len as usize
        }
    }

    /// Returns the raw (possibly negative) length, matching Java's `int` return.
    pub fn raw_length(&self) -> isize {
        self.p2 - self.p1
    }

    /// Returns the character at `index` within the visible window.
    pub fn char_at(&self, index: usize) -> char {
        if index >= self.length() {
            panic!("Index {} out of bounds for length {}", index, self.length());
        }
        if self.reversed {
            self.chars[(self.p2 - index as isize - 1) as usize]
        } else {
            self.chars[(self.p1 + index as isize) as usize]
        }
    }

    /// Advances the start (or end if reversed) of the visible window by `step`.
    pub fn jump(&mut self, step: usize) {
        if self.reversed {
            self.p2 -= step as isize;
        } else {
            self.p1 += step as isize;
        }
    }

    /// Returns `true` if `searched` occurs starting at `ahead` within the window.
    pub fn starts_with(&self, searched: &str, ahead: usize) -> bool {
        let searched_chars: Vec<char> = searched.chars().collect();
        for (i, &sc) in searched_chars.iter().enumerate() {
            if ahead + i >= self.length() {
                return false;
            }
            if self.char_at(ahead + i) != sc {
                return false;
            }
        }
        true
    }

    /// Searches for `searched` starting at `ahead`; returns the index or `None`.
    pub fn search(&self, searched: &str, ahead: usize) -> Option<usize> {
        let searched_len = searched.chars().count();
        if searched_len == 0 {
            return Some(ahead);
        }
        let max = self.length().saturating_sub(searched_len);
        for i in ahead..=max {
            if self.starts_with(searched, i) {
                return Some(i);
            }
        }
        None
    }

    /// Searches for a position where `pattern` matches, starting at `ahead`.
    /// Returns the position, or `Challenge::NO_MATCH` if not found.
    pub fn search_pattern(&self, pattern: &dyn Challenge, ahead: usize) -> i32 {
        for i in ahead..self.length() {
            let result = pattern.run_challenge(self, i);
            if result.full_capture_length >= 0 {
                return i as i32;
            }
        }
        super::challenge::NO_MATCH
    }
}

impl fmt::Display for TextNavigator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let length = self.length();
        let mut s = String::with_capacity(length);
        for i in 0..length {
            s.push(self.char_at(i));
        }
        f.write_str(&s)
    }
}

impl AsAny for TextNavigator {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Helper trait for downcasting.
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_length() {
        let tn = TextNavigator::build("hello");
        assert_eq!(tn.length(), 5);
    }

    #[test]
    fn char_at_forward() {
        let tn = TextNavigator::build("abc");
        assert_eq!(tn.char_at(0), 'a');
        assert_eq!(tn.char_at(1), 'b');
        assert_eq!(tn.char_at(2), 'c');
    }

    #[test]
    fn sub_sequence_forward() {
        let tn = TextNavigator::build("hello");
        let sub = tn.sub_sequence(1, 4);
        assert_eq!(sub.length(), 3);
        assert_eq!(sub.char_at(0), 'e');
        assert_eq!(sub.char_at(1), 'l');
        assert_eq!(sub.char_at(2), 'l');
    }

    #[test]
    fn reverse_basic() {
        let tn = TextNavigator::build("abcdef");
        let rev = tn.reverse(4);
        assert_eq!(rev.length(), 4);
        assert_eq!(rev.char_at(0), 'd');
        assert_eq!(rev.char_at(1), 'c');
        assert_eq!(rev.char_at(2), 'b');
        assert_eq!(rev.char_at(3), 'a');
    }

    #[test]
    fn jump_forward() {
        let mut tn = TextNavigator::build("hello");
        tn.jump(2);
        assert_eq!(tn.length(), 3);
        assert_eq!(tn.char_at(0), 'l');
    }

    #[test]
    fn jump_reversed() {
        let tn = TextNavigator::build("abcdef");
        let mut rev = tn.reverse(4);
        rev.jump(2);
        assert_eq!(rev.length(), 2);
        assert_eq!(rev.char_at(0), 'b');
        assert_eq!(rev.char_at(1), 'a');
    }

    #[test]
    fn jump_reversed_past_zero() {
        // In Java, jump can make p2 < p1, producing negative length.
        // length() should return 0, not panic.
        let tn = TextNavigator::build("abc");
        let mut rev = tn.reverse(2);
        rev.jump(3);
        assert_eq!(rev.length(), 0);
    }

    #[test]
    fn starts_with_basic() {
        let tn = TextNavigator::build("hello world");
        assert!(tn.starts_with("hello", 0));
        assert!(!tn.starts_with("world", 0));
        assert!(tn.starts_with("world", 6));
    }

    #[test]
    fn search_basic() {
        let tn = TextNavigator::build("hello world");
        assert_eq!(tn.search("world", 0), Some(6));
        assert_eq!(tn.search("xyz", 0), None);
    }

    #[test]
    fn to_string_forward() {
        let tn = TextNavigator::build("hello");
        assert_eq!(tn.to_string(), "hello");
    }

    #[test]
    fn to_string_reversed() {
        let tn = TextNavigator::build("abcdef");
        let rev = tn.reverse(4);
        assert_eq!(rev.to_string(), "dcba");
    }
}

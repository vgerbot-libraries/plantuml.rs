//! Result of a `Challenge::run_challenge` call.
//!
//! `full_capture_length` is the number of characters consumed (≥ 0 on success,
//! negative on no match).  `capture` holds named-group captures.
//!
//! Ported from: `com/plantuml/ubrex/ChallengeResult.java`
use super::capture::Capture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeResult {
    pub full_capture_length: i32,
    pub capture: Capture,
}

impl ChallengeResult {
    /// Creates a no-match result.
    pub const fn no_match() -> Self {
        Self {
            full_capture_length: super::challenge::NO_MATCH,
            capture: Capture::empty(),
        }
    }

    /// Creates a zero-length match (e.g. for lookarounds).
    pub const fn zero() -> Self {
        Self {
            full_capture_length: 0,
            capture: Capture::empty(),
        }
    }

    /// Creates a one-character match.
    pub const fn one() -> Self {
        Self {
            full_capture_length: 1,
            capture: Capture::empty(),
        }
    }

    /// Creates a result with the given length and empty capture.
    pub const fn new(length: i32) -> Self {
        Self {
            full_capture_length: length,
            capture: Capture::empty(),
        }
    }

    /// Creates a result with the given length and capture.
    pub const fn with_capture(length: i32, capture: Capture) -> Self {
        Self {
            full_capture_length: length,
            capture,
        }
    }

    /// Returns all values for `key` from the capture.
    pub fn find_values_by_key(&self, key: &str) -> Vec<String> {
        self.capture.find_values_by_key(key)
    }

    /// Returns the first set of values whose key starts with `key_prefix`.
    pub fn find_first_values_by_key_prefix(&self, key_prefix: &str) -> Option<Vec<String>> {
        self.capture.find_first_values_by_key_prefix(key_prefix)
    }

    /// Returns a capture containing only entries with the given prefix.
    pub fn extract_by_prefix(&self, key: &str) -> Capture {
        self.capture.extract_by_prefix(key)
    }
}

impl std::fmt::Display for ChallengeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fullCaptureLength={} {}",
            self.full_capture_length, self.capture
        )
    }
}

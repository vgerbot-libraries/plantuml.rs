/// Core matching trait for the ubrex regex engine.
///
/// Each `Challenge` tests whether a pattern matches at a given position in a
/// `TextNavigator`, returning a `ChallengeResult`.
///
/// Ported from: `com/plantuml/ubrex/Challenge.java`

use std::any::Any;

use super::challenge_result::ChallengeResult;
use super::text_navigator::TextNavigator;

/// Sentinel value indicating no match (Java: `Integer.MIN_VALUE`).
pub const NO_MATCH: i32 = i32::MIN;

pub trait Challenge: std::fmt::Display {
    /// Tests whether this pattern matches at `position` in `string`.
    fn run_challenge(&self, string: &TextNavigator, position: usize) -> ChallengeResult;

    /// Convenience: match against a plain `&str` starting at position 0.
    fn run_challenge_str(&self, string: &str) -> ChallengeResult {
        let nav = TextNavigator::build(string);
        self.run_challenge(&nav, 0)
    }

    /// Downcast support for `UBrexNamed` type-checking.
    fn as_any(&self) -> &dyn Any;
}

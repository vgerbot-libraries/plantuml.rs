/// Base trait for ubrex builder parts.
///
/// Each builder part wraps a `Challenge` and provides `match` via
/// `UnicodeBracketedExpression`.
///
/// Ported from: `com/plantuml/ubrex/builder/UBrexPart.java`

use std::rc::Rc;

use crate::ubrex::challenge::Challenge;
use crate::ubrex::text_navigator::TextNavigator;
use crate::ubrex::u_matcher::UMatcher;

pub struct UBrexPart {
    challenge: Rc<dyn Challenge>,
}

impl UBrexPart {
    pub fn new(challenge: Rc<dyn Challenge>) -> Self {
        UBrexPart { challenge }
    }

    pub fn get_challenge(&self) -> &Rc<dyn Challenge> {
        &self.challenge
    }

    /// Matches against `string` starting at `position`.
    pub fn match_text(&self, string: &TextNavigator, position: usize) -> Box<dyn UMatcher> {
        let expr = crate::ubrex::unicode_bracketed_expression::from(Rc::clone(&self.challenge));
        expr.match_text(string, position)
    }

    /// Matches against a plain `&str` starting at `position`.
    pub fn match_str(&self, string: &str, position: usize) -> Box<dyn UMatcher> {
        let expr = crate::ubrex::unicode_bracketed_expression::from(Rc::clone(&self.challenge));
        expr.match_str(string, position)
    }
}

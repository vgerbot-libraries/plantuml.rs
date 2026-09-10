/// Repetition specification parsed from `{...}` quantifier syntax.
///
/// Supports exact counts (`{3}`), ranges (`{2-5}`), minimum-or-more (`{3+}`),
/// and semicolon-separated combinations (`{1;3;5-8;10+}`).
///
/// Ported from: `com/plantuml/ubrex/Repetition.java`

use std::collections::BTreeSet;

use super::text_navigator::TextNavigator;

pub struct Repetition {
    values: BTreeSet<i32>,
    min_inclusive: i32,
}

impl Repetition {
    pub fn new() -> Self {
        Repetition {
            values: BTreeSet::new(),
            min_inclusive: i32::MAX,
        }
    }

    /// Parses a repetition spec from `input`, consuming characters until `}`.
    /// The `}` is also consumed.
    pub fn parse(input: &mut TextNavigator) -> Repetition {
        let mut result = Repetition::new();
        let mut token = String::new();
        loop {
            let ch = input.char_at(0);
            if ch == '}' {
                result.add_token(&token);
                input.jump(1);
                return result;
            } else if ch == ';' {
                result.add_token(&token);
                token.clear();
            } else {
                token.push(ch);
            }
            input.jump(1);
        }
    }

    fn add_token(&mut self, token: &str) {
        if token.is_empty() {
            panic!("empty repetition token");
        }
        if token.ends_with('+') {
            let min: i32 = token[..token.len() - 1]
                .parse()
                .unwrap_or_else(|_| panic!("invalid repetition token: {token}"));
            self.min_inclusive = min;
        } else if let Some(dash) = token.find('-') {
            let min: i32 = token[..dash]
                .parse()
                .unwrap_or_else(|_| panic!("invalid repetition token: {token}"));
            let max: i32 = token[dash + 1..]
                .parse()
                .unwrap_or_else(|_| panic!("invalid repetition token: {token}"));
            for i in min..=max {
                self.add(i);
            }
        } else {
            let exact: i32 = token
                .parse()
                .unwrap_or_else(|_| panic!("invalid repetition token: {token}"));
            self.add(exact);
        }
    }

    fn add(&mut self, nb: i32) {
        self.values.insert(nb);
    }

    /// Returns `true` if `nb` repetitions satisfies this specification.
    pub fn matches(&self, nb: i32) -> bool {
        if nb >= self.min_inclusive {
            return true;
        }
        self.values.contains(&nb)
    }
}

impl std::fmt::Display for Repetition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.values)
    }
}

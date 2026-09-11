//! Recursive-descent parser that dispatches by Unicode bracket character.
//!
//! Dispatch table:
//! - `〒` → lookaround
//! - `【】` → alternation
//! - `〇` → quantifier
//! - `〄` → up-to
//! - `〘〙` → group
//! - `「」` → character set
//! - `〴` → character class
//! - `〶` → named group
//! - `〃` → double quote
//! - default → regular character
//!
//! Ported from: `com/plantuml/ubrex/AtomicParser.java`
use std::rc::Rc;

use super::challenge::Challenge;
use super::challenge_alternative::ChallengeAlternative;
use super::challenge_char_class::ChallengeCharClass;
use super::challenge_char_set::ChallengeCharSet;
use super::challenge_end_of_text::ChallengeEndOfText;
use super::challenge_lazzy_one_or_more::ChallengeLazzyOneOrMore;
use super::challenge_look_ahead::ChallengeLookAhead;
use super::challenge_look_behind::ChallengeLookBehind;
use super::challenge_one_or_more::ChallengeOneOrMore;
use super::challenge_one_or_more_up_to_old_version::ChallengeOneOrMoreUpToOldVersion;
use super::challenge_optional::ChallengeOptional;
use super::challenge_repetition::ChallengeRepetition;
use super::challenge_single_char::ChallengeSingleChar;
use super::challenge_up_to::ChallengeUpTo;
use super::challenge_zero_or_more::ChallengeZeroOrMore;
use super::char_class::CharClass;
use super::composite_list::CompositeList;
use super::composite_named::CompositeNamed;
use super::look_around::LookAround;
use super::repetition::Repetition;
use super::text_navigator::TextNavigator;

pub struct AtomicParser;

impl Default for AtomicParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicParser {
    pub const fn new() -> Self {
        Self
    }

    /// Finds the matching closing bracket, accounting for nesting.
    fn get_closing_bracket(input: &TextNavigator, open: char, close: char) -> Option<usize> {
        let mut level = 0i32;
        for i in 1..input.length() {
            let ch = input.char_at(i);
            if ch == open {
                level += 1;
            } else if ch == close {
                if level == 0 {
                    return Some(i);
                }
                level -= 1;
                assert!(level >= 0, "Unbalanced brackets");
            }
        }
        None
    }

    /// Parses a single challenge (expects exactly one result).
    fn parse_single(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        let result = self.parse(input);
        assert!(result.len() == 1, "Expected single challenge, got {}", result.len());
        result.into_iter().next().unwrap()
    }

    /// Parses one or more challenges from the start of `input`.
    pub fn parse(&self, input: &mut TextNavigator) -> Vec<Rc<dyn Challenge>> {
        let ch = input.char_at(0);
        match ch {
            '┇' => panic!("Unexpected ┇"),
            '〒' => vec![self.manage_look_around(input)],
            '【' => vec![self.manage_alternative(input)],
            '〇' => self.manage_quantifier(input),
            '〄' => self.manage_up_to(input),
            '〘' => vec![self.manage_group(input)],
            '「' => vec![self.manage_character_set(input)],
            '〴' => vec![self.manage_class(input)],
            '〶' => self.manage_named(input),
            '〃' => vec![self.manage_double_quote(input)],
            _ => vec![self.manage_regular_character(input)],
        }
    }

    fn manage_look_around(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        input.jump(1);
        let look = LookAround::from(input).expect("Syntax error in lookaround");
        input.jump(look.definition_size());

        if look == LookAround::EndOfText {
            return Rc::new(ChallengeEndOfText);
        }
        let p1 = self.parse_single(input);
        if look.is_look_behind() {
            Rc::new(ChallengeLookBehind::new(p1, look))
        } else if look.is_look_ahead() {
            Rc::new(ChallengeLookAhead::new(p1, look))
        } else {
            panic!("Unknown lookaround type");
        }
    }

    #[allow(clippy::unused_self)]
    fn manage_class(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        input.jump(1);
        let result = CharClass::from_definition(input);
        input.jump(result.definition_length());
        Rc::new(ChallengeCharClass::new(result))
    }

    fn manage_quantifier(&self, input: &mut TextNavigator) -> Vec<Rc<dyn Challenge>> {
        let operator = input.char_at(1);
        input.jump(2);
        if operator == '{' {
            vec![self.manage_quantifier_bracket(input)]
        } else if operator == 'l' {
            self.manage_quantifier_lazzy(input)
        } else {
            let origin = self.parse_single(input);
            match operator {
                '+' => vec![Rc::new(ChallengeOneOrMore::new(origin))],
                '*' => vec![Rc::new(ChallengeZeroOrMore::new(origin))],
                '?' => vec![Rc::new(ChallengeOptional::new(origin))],
                _ => panic!("wip01: unknown quantifier {operator}"),
            }
        }
    }

    fn manage_quantifier_lazzy(&self, input: &mut TextNavigator) -> Vec<Rc<dyn Challenge>> {
        input.jump(1);
        let origin = self.parse_single(input);
        let remaining = CompositeList::parse_and_build_from_text_navigator(input);

        // The lazzy quantifier only peeks at remaining as a stop condition.
        // The remaining challenges are returned as siblings, so they get
        // matched normally by the parent CompositeList/CompositeNamed.
        let mut result: Vec<Rc<dyn Challenge>> = Vec::new();
        let remaining_rc: Rc<dyn Challenge> = Rc::new(remaining.clone());
        result.push(Rc::new(ChallengeLazzyOneOrMore::new(origin, remaining_rc)));
        for c in remaining.get_internal_challenges_list() {
            result.push(Rc::clone(c));
        }
        result
    }

    fn manage_quantifier_bracket(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        let repetition = Repetition::parse(input);
        let origin = self.parse_single(input);
        Rc::new(ChallengeRepetition::new(repetition, origin))
    }

    fn manage_up_to(&self, input: &mut TextNavigator) -> Vec<Rc<dyn Challenge>> {
        let operator = input.char_at(1);
        if operator == '>' {
            input.jump(2);
            let p2 = self.parse_single(input);
            return vec![
                Rc::new(ChallengeUpTo::new(Rc::clone(&p2))),
                p2,
            ];
        }
        assert!(operator == '+', "manageQuantifierUpTo1");

        input.jump(2);
        self.skip_spaces(input);

        let p1 = self.parse_single(input);

        self.skip_spaces(input);

        assert!(input.char_at(0) == '-', "manageQuantifierUpTo2");
        assert!(input.char_at(1) == '>', "manageQuantifierUpTo2");

        input.jump(2);
        self.skip_spaces(input);

        let p2 = self.parse_single(input);
        vec![
            Rc::new(ChallengeOneOrMoreUpToOldVersion::new(Rc::clone(&p1), Rc::clone(&p2))),
            p2,
        ]
    }

    #[allow(clippy::unused_self)]
    fn skip_spaces(&self, input: &mut TextNavigator) {
        while input.length() > 0 && input.char_at(0) == ' ' {
            input.jump(1);
        }
    }

    #[allow(clippy::unused_self)]
    fn manage_group(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        let end = Self::get_closing_bracket(input, '〘', '〙').expect("wip99: unclosed group");
        let sub = input.sub_sequence(1, end);
        let mut sub_nav = sub;
        let result = CompositeList::parse_and_build_from_text_navigator(&mut sub_nav);
        input.jump(end + 1);
        Rc::new(result)
    }

    #[allow(clippy::unused_self)]
    fn manage_alternative(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        let mut result = ChallengeAlternative::new();

        let mut start = 1usize;
        let mut level = 0i32;
        for i in 1..input.length() {
            let ch = input.char_at(i);
            if ch == '【' {
                level += 1;
            } else if level == 0 && ch == '┇' {
                let sub = input.sub_sequence(start, i);
                let mut sub_nav = sub;
                let part = CompositeList::parse_and_build_from_text_navigator(&mut sub_nav);
                result.add_alternative(Rc::new(part));
                start = i + 1;
            } else if ch == '】' {
                if level == 0 {
                    let sub = input.sub_sequence(start, i);
                    let mut sub_nav = sub;
                    let part = CompositeList::parse_and_build_from_text_navigator(&mut sub_nav);
                    result.add_alternative(Rc::new(part));
                    input.jump(i + 1);
                    return Rc::new(result);
                }
                level -= 1;
                assert!(level >= 0, "Unbalanced alternative brackets");
            }
        }
        panic!("wip32: unclosed alternative");
    }

    fn manage_named(&self, input: &mut TextNavigator) -> Vec<Rc<dyn Challenge>> {
        let mut name = String::new();
        assert!(input.char_at(1) == '$', "varname must have a $");

        input.jump(2);
        loop {
            let ch = input.char_at(0);
            input.jump(1);
            if ch == '=' {
                assert!(!name.is_empty(), "no name!");
                let parsed = self.parse(input);
                let named = CompositeNamed::new(name.clone(), Rc::clone(&parsed[0]));
                if parsed.len() == 1 {
                    return vec![Rc::new(named)];
                }
                let mut result: Vec<Rc<dyn Challenge>> = Vec::new();
                result.push(Rc::new(named));
                for c in parsed.into_iter().skip(1) {
                    result.push(c);
                }
                return result;
            }
            if is_identifier_part(ch) {
                name.push(ch);
            } else {
                panic!("Unsupported name character: {ch}");
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn manage_character_set(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        let end = input.index_of('」').expect("wip80: unclosed charset");
        let sub = input.sub_sequence(1, end);
        let result = ChallengeCharSet::build(&sub.to_string());
        input.jump(end + 1);
        Rc::new(result)
    }

    #[allow(clippy::unused_self)]
    fn manage_regular_character(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        let ch = input.char_at(0);
        assert!(ch != ' ', "no space allowed");
        let ch = if ch == '∙' { ' ' } else { ch };
        input.jump(1);
        Rc::new(ChallengeSingleChar::new(ch))
    }

    #[allow(clippy::unused_self)]
    fn manage_double_quote(&self, input: &mut TextNavigator) -> Rc<dyn Challenge> {
        input.jump(1);
        Rc::new(ChallengeSingleChar::new('"'))
    }
}

/// Checks if a character is a valid identifier part.
/// Approximates `Character.isJavaIdentifierPart` for the characters
/// used in ubrex patterns.
fn is_identifier_part(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

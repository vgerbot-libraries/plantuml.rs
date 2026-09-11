//! Entry point for the ubrex regex engine.
//!
//! Build a pattern with `UnicodeBracketedExpression::build(pattern)` then call
//! `match(text)` to get a `UMatcher`.
//!
//! Ported from: `com/plantuml/ubrex/UnicodeBracketedExpression.java`
use std::rc::Rc;

use super::capture::Capture;
use super::capture_lookup::CaptureLookup;
use super::challenge::Challenge;
use super::challenge_result::ChallengeResult;
use super::composite_list::CompositeList;
use super::text_navigator::TextNavigator;
use super::u_matcher::UMatcher;

/// Builds a `UnicodeBracketedExpression` from a ubrex pattern string.
pub fn build(ubrex: &str) -> UnicodeBracketedExpression {
    let challenge = CompositeList::parse_and_build(ubrex);
    from(Rc::new(challenge))
}

/// Creates a `UnicodeBracketedExpression` from a `Challenge`.
pub fn from(challenge: Rc<dyn Challenge>) -> UnicodeBracketedExpression {
    UnicodeBracketedExpression { challenge }
}

/// A compiled ubrex pattern that can match against text.
pub struct UnicodeBracketedExpression {
    challenge: Rc<dyn Challenge>,
}

impl UnicodeBracketedExpression {
    /// Matches against `string` starting at `position`.
    pub fn match_text(&self, string: &TextNavigator, position: usize) -> Box<dyn UMatcher> {
        let shall_we_pass = self.challenge.run_challenge(string, position);
        let accepted_match = if shall_we_pass.full_capture_length < 0 {
            String::new()
        } else {
            string
                .sub_sequence(position, position + shall_we_pass.full_capture_length as usize)
                .to_string()
        };
        Box::new(MatcherImpl {
            shall_we_pass,
            accepted_match,
            text_length: string.length(),
            position,
        })
    }

    /// Matches against `string` starting at position 0.
    pub fn match_full(&self, string: &TextNavigator) -> Box<dyn UMatcher> {
        self.match_text(string, 0)
    }

    /// Matches against a plain `&str` starting at `position`.
    pub fn match_str(&self, string: &str, position: usize) -> Box<dyn UMatcher> {
        let nav = TextNavigator::build(string);
        self.match_text(&nav, position)
    }
}

struct MatcherImpl {
    shall_we_pass: ChallengeResult,
    accepted_match: String,
    text_length: usize,
    position: usize,
}

impl UMatcher for MatcherImpl {
    fn start_match(&self) -> bool {
        self.shall_we_pass.full_capture_length >= 0
    }

    fn exact_match(&self) -> bool {
        if !self.start_match() {
            return false;
        }
        self.position + self.shall_we_pass.full_capture_length as usize == self.text_length
    }

    fn get_accepted_match(&self) -> String {
        self.accepted_match.clone()
    }

    fn find_first_values_by_key_prefix(&self, key_prefix: &str) -> Option<Vec<String>> {
        self.shall_we_pass.find_first_values_by_key_prefix(key_prefix)
    }

    fn extract_by_prefix(&self, key: &str) -> Capture {
        self.shall_we_pass.extract_by_prefix(key)
    }
}

impl CaptureLookup for MatcherImpl {
    fn find_first_value_by_key(&self, key: &str) -> Option<String> {
        self.shall_we_pass.find_values_by_key(key).into_iter().next()
    }

    fn find_values_by_key(&self, key: &str) -> Vec<String> {
        self.shall_we_pass.find_values_by_key(key)
    }
}

impl std::fmt::Display for MatcherImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.accepted_match, self.shall_we_pass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::text_navigator::TextNavigator;

    fn match_str(pattern: &str, text: &str) -> String {
        let cut = build(pattern);
        cut.match_str(text, 0).get_accepted_match()
    }

    fn match_nav(pattern: &str, text: &str) -> Box<dyn UMatcher> {
        let cut = build(pattern);
        cut.match_text(&TextNavigator::build(text), 0)
    }

    fn match_nav_pos(pattern: &str, text: &str, pos: usize) -> Box<dyn UMatcher> {
        let cut = build(pattern);
        cut.match_text(&TextNavigator::build(text), pos)
    }

    fn values_to_string(values: &[String]) -> String {
        format!("{:?}", values)
    }

    // ===== SimpleTest =====
    /// Ported from: `SimpleTest.java`
    #[test]
    fn test_add_and_get_longest_match() {
        assert_eq!("hello", match_str("hello", "hello"));
        assert_eq!("hello", match_str("hello", "hello world"));
        assert_eq!("hello", match_nav_pos("hello", "say hello", 4).get_accepted_match());
        assert_eq!("", match_str("hello", "hi there"));
    }

    #[test]
    fn test_longest_match_at_end() {
        assert_eq!("end", match_nav_pos("end", "the end", 4).get_accepted_match());
        let text = "the end";
        assert_eq!("", match_nav_pos("end", text, text.len()).get_accepted_match());
    }

    // ===== Test1 =====
    /// Ported from: `Test1.java`
    #[test]
    fn test1() {
        let cut = build("if 〇*〴s 〴g  〶$IF1=〇*〴G 〴g 〇*〴s 〇?〘as 〇+〴s 〶$ASIF1=〇+「〴an_.」 〇+〴s 〙  〇?〘then〙");
        let matcher = cut.match_text(&TextNavigator::build("if \"\" then"), 0);
        assert_eq!("if \"\" then", matcher.get_accepted_match());
    }

    // ===== AlternativeTest =====
    /// Ported from: `AlternativeTest.java`
    #[test]
    fn alt_test1() {
        assert_eq!("a", match_str("【a┇b┇c】", "a"));
        assert_eq!("b", match_str("【a┇b┇c】", "b"));
        assert_eq!("c", match_str("【a┇b┇c】", "c"));
        assert_eq!("", match_str("【a┇b┇c】", "h"));
    }

    #[test]
    fn alt_test2() {
        assert_eq!("ab", match_str("【ab┇cd】", "ab"));
        assert_eq!("cd", match_str("【ab┇cd】", "cd"));
        assert_eq!("", match_str("【ab┇cd】", "c"));
        assert_eq!("", match_str("【ab┇cd】", "h"));
    }

    #[test]
    fn alt_test3() {
        let cut = build("【〇+a┇〇+b┇〇?c】");
        assert_eq!("", cut.match_text(&TextNavigator::build(""), 0).get_accepted_match());
        assert_eq!("aaaa", cut.match_text(&TextNavigator::build("aaaa"), 0).get_accepted_match());
        assert_eq!("bb", cut.match_text(&TextNavigator::build("bb"), 0).get_accepted_match());
        assert_eq!("c", cut.match_text(&TextNavigator::build("c"), 0).get_accepted_match());
    }

    #[test]
    fn alt_test4() {
        let cut = build("partition∙〇+〴S∙【 # 〇{6}「0〜9a〜fA〜F」┇ 〇?# 〇+〴w 】");
        assert_eq!("partition to #012345", cut.match_str("partition to #012345", 0).get_accepted_match());
        assert_eq!("partition to gray", cut.match_str("partition to gray", 0).get_accepted_match());
    }

    #[test]
    fn alt_test5() {
        let cut = build("partition∙〇+〴S∙# 〇{6}「0〜9a〜fA〜F」");
        assert_eq!("partition to #012345", cut.match_str("partition to #012345", 0).get_accepted_match());
    }

    #[test]
    fn alt_test50() {
        let cut = build("partition∙〇+〴S∙# 〇+「0〜9」");
        assert_eq!("partition to #012345", cut.match_str("partition to #012345", 0).get_accepted_match());
    }

    // ===== NamedGroupTest =====
    /// Ported from: `NamedGroupTest.java`
    #[test]
    fn named_group_no_match() {
        let cut = build("a  〶$group1=〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("heo"), 0);
        assert_eq!("", matcher.get_accepted_match());
        assert_eq!("[]", values_to_string(&matcher.find_values_by_key("group1")));
    }

    #[test]
    fn named_group_single_char() {
        let cut = build("a  〶$group1=〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("abc"), 0);
        assert_eq!("abc", matcher.get_accepted_match());
        assert_eq!("[\"b\"]", values_to_string(&matcher.find_values_by_key("group1")));
    }

    #[test]
    fn named_group_double_chars() {
        let cut = build("a  〶$group1=〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("abbc"), 0);
        assert_eq!("abbc", matcher.get_accepted_match());
        assert_eq!("[\"bb\"]", values_to_string(&matcher.find_values_by_key("group1")));
    }

    #[test]
    fn named_group_triple_chars() {
        let cut = build("a  〶$group1=〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("abbbc"), 0);
        assert_eq!("abbbc", matcher.get_accepted_match());
        assert_eq!("[\"bbb\"]", values_to_string(&matcher.find_values_by_key("group1")));
    }

    // ===== NamedGroup2Test =====
    /// Ported from: `NamedGroup2Test.java`
    #[test]
    fn named_group2_10() {
        let cut = build("〶$gr=〘 〇*〴s  〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("000"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"000\"]", values_to_string(&matcher.find_values_by_key("gr")));
    }

    #[test]
    fn named_group2_20() {
        let cut = build("〇+〘 〇*〴s  〶$num=〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("123 456"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123\", \"456\"]", values_to_string(&matcher.find_values_by_key("num")));
    }

    #[test]
    fn named_group2_21() {
        let cut = build("〇+〘 〇*〴s  〶$num=〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("123 456 789"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123\", \"456\", \"789\"]", values_to_string(&matcher.find_values_by_key("num")));
    }

    #[test]
    fn named_group2_30() {
        let cut = build("〶$gr=〇+〘 〇*〴s  〶$num=〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("123 456"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123 456\"]", values_to_string(&matcher.find_values_by_key("gr")));
        assert_eq!("[\"123\", \"456\"]", values_to_string(&matcher.find_values_by_key("gr/num")));
    }

    #[test]
    fn named_group2_00() {
        let cut = build("〘 〇*〴s  〶$num=〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("123 456"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123\"]", values_to_string(&matcher.find_values_by_key("num")));
    }

    #[test]
    fn named_group2_01() {
        let cut = build("〇+〘 〇*〴s  〶$num=〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("123 456"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123\", \"456\"]", values_to_string(&matcher.find_values_by_key("num")));
    }

    #[test]
    fn named_group2_02() {
        let cut = build(" 〶$gr=〘 〇*〴s  〶$num=〇+〴d 〙");
        let matcher = cut.match_text(&TextNavigator::build("123 456"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123\"]", values_to_string(&matcher.find_values_by_key("gr")));
        assert_eq!("[\"123\"]", values_to_string(&matcher.find_values_by_key("gr/num")));
    }

    #[test]
    fn named_group2_31() {
        let cut = build("〶$gr=〇+〘 〇*〴s  〶$num=〇+〴d  〶$letter=〇*「a〜z」 〙");
        let matcher = cut.match_text(&TextNavigator::build("123qsd 456 0ccc 0"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123qsd 456 0ccc 0\"]", values_to_string(&matcher.find_values_by_key("gr")));
        assert_eq!("[\"123\", \"456\", \"0\", \"0\"]", values_to_string(&matcher.find_values_by_key("gr/num")));
        assert_eq!("[\"qsd\", \"\", \"ccc\", \"\"]", values_to_string(&matcher.find_values_by_key("gr/letter")));
    }

    #[test]
    fn named_group2_32() {
        let cut = build("〶$gr=〇+〘 〇*「〴s_」  〶$num=〇+〴d  〶$letter=〇*「a〜z」 〙");
        let matcher = cut.match_text(&TextNavigator::build("123qsd 456 0ccc 0"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123qsd 456 0ccc 0\"]", values_to_string(&matcher.find_values_by_key("gr")));
        assert_eq!("[\"123\", \"456\", \"0\", \"0\"]", values_to_string(&matcher.find_values_by_key("gr/num")));
        assert_eq!("[\"qsd\", \"\", \"ccc\", \"\"]", values_to_string(&matcher.find_values_by_key("gr/letter")));
    }

    #[test]
    fn named_group2_33() {
        let cut = build("〇+〘 *  〶$gr=〇+〘 〇*「〴s_」  〶$num=〇+〴d  〶$letter=〇*「a〜z」 〙   〙");
        let matcher = cut.match_text(&TextNavigator::build("*123qsd 456 1ccc 2*3aa"), 0);
        assert!(matcher.start_match());
        assert_eq!("[\"123qsd 456 1ccc 2\", \"3aa\"]", values_to_string(&matcher.find_values_by_key("gr")));
        assert_eq!("[\"123\", \"456\", \"1\", \"2\", \"3\"]", values_to_string(&matcher.find_values_by_key("gr/num")));
        assert_eq!("[\"qsd\", \"\", \"ccc\", \"\", \"aa\"]", values_to_string(&matcher.find_values_by_key("gr/letter")));
    }

    // ===== TwoGroupsSimpleTest =====
    /// Ported from: `TwoGroupsSimpleTest.java`
    #[test]
    fn two_groups_simple_no_match() {
        let cut = build("0〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("09"), 0);
        assert_eq!("", matcher.get_accepted_match());
    }

    #[test]
    fn two_groups_simple_match() {
        let cut = build("0〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("0abc9"), 0);
        assert_eq!("0abc9", matcher.get_accepted_match());
    }

    #[test]
    fn two_groups_simple_match2() {
        let cut = build("0〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("0abbbc9"), 0);
        assert_eq!("0abbbc9", matcher.get_accepted_match());
    }

    // ===== TwoGroupsTest =====
    /// Ported from: `TwoGroupsTest.java`
    #[test]
    fn two_groups_no_group_match() {
        let cut = build("0〇*〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("09"), 0);
        assert_eq!("09", matcher.get_accepted_match());
    }

    #[test]
    fn two_groups_minimal_match() {
        let cut = build("0〇*〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("0abbc9"), 0);
        assert_eq!("0abbc9", matcher.get_accepted_match());
    }

    #[test]
    fn two_groups_with_prefix_wildcards() {
        let cut = build("0〇*〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("0XYZaabc9"), 0);
        assert_eq!("", matcher.get_accepted_match());
    }

    #[test]
    fn two_groups_several() {
        let cut = build("0〇*〘a〘〇+b〙c〙9");
        let matcher = cut.match_text(&TextNavigator::build("0abbcabbbbc9"), 0);
        assert_eq!("0abbcabbbbc9", matcher.get_accepted_match());
    }

    // ===== UnnamedGroupTest =====
    /// Ported from: `UnnamedGroupTest.java`
    #[test]
    fn unnamed_group_no_match() {
        let cut = build("a〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("heo"), 0);
        assert_eq!("", matcher.get_accepted_match());
    }

    #[test]
    fn unnamed_group_single_char() {
        let cut = build("a〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("abc"), 0);
        assert_eq!("abc", matcher.get_accepted_match());
    }

    #[test]
    fn unnamed_group_double_chars() {
        let cut = build("a〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("abbc"), 0);
        assert_eq!("abbc", matcher.get_accepted_match());
    }

    #[test]
    fn unnamed_group_triple_chars() {
        let cut = build("a〘〇+b〙c");
        let matcher = cut.match_text(&TextNavigator::build("abbbc"), 0);
        assert_eq!("abbbc", matcher.get_accepted_match());
    }

    // ===== OneOrMoreTest =====
    /// Ported from: `OneOrMoreTest.java`
    #[test]
    fn one_or_more_invalid() {
        assert_eq!("", match_str("he〇+lo", "heo"));
    }

    #[test]
    fn one_or_more_helo() {
        assert_eq!("helo", match_str("he〇+lo", "helo"));
    }

    #[test]
    fn one_or_more_hello() {
        assert_eq!("hello", match_str("he〇+lo", "hello"));
    }

    #[test]
    fn one_or_more_helllo() {
        assert_eq!("helllo", match_str("he〇+lo", "helllo"));
    }

    // ===== OneOrMoreLazzyTest =====
    /// Ported from: `OneOrMoreLazzyTest.java`
    #[test]
    fn lazzy_test10() {
        assert_eq!("abbbbend", match_str("a 〇l+〴w end  〒$", "abbbbend"));
    }

    #[test]
    fn lazzy_test20() {
        assert_eq!("aabbbbend", match_str("aa 〇l+〴w end  〒$", "aabbbbend"));
    }

    #[test]
    fn lazzy_test30() {
        let cut = build("aa 〇l+〶$gr1=〴w 〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("aabbbbend"), 0);
        assert_eq!("aabbbbend", matcher.get_accepted_match());
        assert_eq!("[\"b\", \"b\", \"b\", \"b\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    #[test]
    fn lazzy_test40() {
        let cut = build("aa 〶$gr1=〇l+〴w   〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("aabbbbend"), 0);
        assert_eq!("aabbbbend", matcher.get_accepted_match());
        assert_eq!("[\"bbbb\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    #[test]
    fn lazzy_test50() {
        let cut = build("aaa 〶$gr1=〇l+〴w   〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("aaabbbbend"), 0);
        assert_eq!("aaabbbbend", matcher.get_accepted_match());
        assert_eq!("[\"bbbb\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    #[test]
    fn lazzy_test60() {
        let cut = build("aaa 〶$gr1=〇l+〘〴w〴w〙   〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("aaabbbbbbend"), 0);
        assert_eq!("aaabbbbbbend", matcher.get_accepted_match());
        assert_eq!("[\"bbbbbb\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    #[test]
    fn lazzy_test61() {
        let cut = build(" 〶$gr1=〇l+〘〴w〴w〙   〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("bbbbbbend"), 0);
        assert_eq!("bbbbbbend", matcher.get_accepted_match());
        assert_eq!("[\"bbbbbb\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    #[test]
    fn lazzy_test70() {
        let cut = build(" 〶$gr1=〇l+〘〴w〴w〙  〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("bbend"), 0);
        assert_eq!("bbend", matcher.get_accepted_match());
        assert_eq!("[\"bb\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    #[test]
    fn lazzy_test71() {
        let cut = build("aaaa 〶$gr1=〇l+〘〴w〴w〙   〶$gr2=end  〒$");
        let matcher = cut.match_text(&TextNavigator::build("aaaabbend"), 0);
        assert_eq!("aaaabbend", matcher.get_accepted_match());
        assert_eq!("[\"bb\"]", values_to_string(&matcher.find_values_by_key("gr1")));
        assert_eq!("[\"e\"]", values_to_string(&matcher.find_values_by_key("gr2")));
    }

    // ===== ZeroOrMoreTest =====
    /// Ported from: `ZeroOrMoreTest.java`
    #[test]
    fn zero_or_more_heo() {
        assert_eq!("heo", match_str("he〇*lo", "heo"));
    }

    #[test]
    fn zero_or_more_helo() {
        assert_eq!("helo", match_str("he〇*lo", "helo"));
    }

    #[test]
    fn zero_or_more_hello() {
        assert_eq!("hello", match_str("he〇*lo", "hello"));
    }

    // ===== OptionalTest =====
    /// Ported from: `OptionalTest.java`
    #[test]
    fn optional_heo() {
        assert_eq!("heo", match_str("he〇?lo", "heo"));
    }

    #[test]
    fn optional_helo() {
        assert_eq!("helo", match_str("he〇?lo", "helo"));
    }

    #[test]
    fn optional_hello() {
        assert_eq!("", match_str("he〇?lo", "hello"));
    }

    // ===== RepetitionTest =====
    /// Ported from: `RepetitionTest.java`
    #[test]
    fn rep_exact6_match() {
        assert_eq!("hellllllo", match_str("he〇{6}lo", "hellllllo"));
    }

    #[test]
    fn rep_exact6_no_match() {
        assert_eq!("", match_str("he〇{6}lo", "helllllo"));
    }

    #[test]
    fn rep_exact2() {
        let cut = build("〇{2}a");
        assert!(cut.match_str("aa", 0).exact_match());
        assert!(!cut.match_str("a", 0).exact_match());
        assert!(!cut.match_str("aaa", 0).exact_match());
    }

    #[test]
    fn rep_exact_multi_digit() {
        let cut = build("〇{12}x");
        let twelve = "xxxxxxxxxxxx";
        assert!(cut.match_str(twelve, 0).exact_match());
        assert!(!cut.match_str(&format!("{twelve}x"), 0).exact_match());
        assert!(!cut.match_str(&twelve[1..], 0).exact_match());
    }

    #[test]
    fn rep_range1to3() {
        let cut = build("〇{1-3}aZ");
        assert!(cut.match_str("aZ", 0).exact_match());
        assert!(cut.match_str("aaZ", 0).exact_match());
        assert!(cut.match_str("aaaZ", 0).exact_match());
        assert!(!cut.match_str("Z", 0).exact_match());
        assert!(!cut.match_str("aaaaZ", 0).exact_match());
    }

    #[test]
    fn rep_range2to2() {
        let cut = build("〇{2-2}a");
        assert!(cut.match_str("aa", 0).exact_match());
        assert!(!cut.match_str("a", 0).exact_match());
        assert!(!cut.match_str("aaa", 0).exact_match());
    }

    #[test]
    fn rep_min_or_more2() {
        let cut = build("〇{2+}aZ");
        assert!(!cut.match_str("aZ", 0).exact_match());
        assert!(cut.match_str("aaZ", 0).exact_match());
        assert!(cut.match_str("aaaZ", 0).exact_match());
        assert!(cut.match_str("aaaaaZ", 0).exact_match());
    }

    #[test]
    fn rep_min_or_more1() {
        let cut = build("〇{1+}b");
        assert!(cut.match_str("b", 0).exact_match());
        assert!(cut.match_str("bb", 0).exact_match());
        assert!(cut.match_str("bbbbb", 0).exact_match());
        assert!(!cut.match_str("", 0).exact_match());
    }

    #[test]
    fn rep_combined_exact() {
        let cut = build("〇{1;3;5}xZ");
        assert!(cut.match_str("xZ", 0).exact_match());
        assert!(!cut.match_str("xxZ", 0).exact_match());
        assert!(cut.match_str("xxxZ", 0).exact_match());
        assert!(!cut.match_str("xxxxZ", 0).exact_match());
        assert!(cut.match_str("xxxxxZ", 0).exact_match());
    }

    #[test]
    fn rep_combined_range_and_more() {
        let cut = build("〇{2;4-6;8+}aZ");
        assert!(!cut.match_str("aZ", 0).exact_match());
        assert!(cut.match_str("aaZ", 0).exact_match());
        assert!(!cut.match_str("aaaZ", 0).exact_match());
        assert!(cut.match_str("aaaaZ", 0).exact_match());
        assert!(cut.match_str("aaaaaZ", 0).exact_match());
        assert!(cut.match_str("aaaaaaZ", 0).exact_match());
        assert!(!cut.match_str("aaaaaaaZ", 0).exact_match());
        assert!(cut.match_str("aaaaaaaaZ", 0).exact_match());
        assert!(cut.match_str("aaaaaaaaaaZ", 0).exact_match());
    }

    #[test]
    fn rep_range_with_charset() {
        let cut = build("〇{1-2}「ab」Z");
        assert!(cut.match_str("aZ", 0).exact_match());
        assert!(cut.match_str("bZ", 0).exact_match());
        assert!(cut.match_str("abZ", 0).exact_match());
        assert!(cut.match_str("baZ", 0).exact_match());
        assert!(!cut.match_str("aabZ", 0).exact_match());
        assert!(!cut.match_str("cZ", 0).exact_match());
    }

    #[test]
    fn rep_range_with_digit_class() {
        let cut = build("〇{1-4}〴d");
        assert!(cut.match_str("1", 0).exact_match());
        assert!(cut.match_str("12", 0).exact_match());
        assert!(cut.match_str("123", 0).exact_match());
        assert!(cut.match_str("1234", 0).exact_match());
        assert!(!cut.match_str("12345", 0).exact_match());
        assert!(!cut.match_str("", 0).exact_match());
    }

    // ===== RangeTest =====
    /// Ported from: `RangeTest.java`
    #[test]
    fn range_test1() {
        assert_eq!("h", match_str("「〴w」", "h"));
    }

    #[test]
    fn range_test2() {
        assert_eq!("h", match_str("「〤〴d」", "h"));
    }

    #[test]
    fn range_test3() {
        assert_eq!("", match_str("「〤h」", "h"));
    }

    #[test]
    fn range_test4() {
        assert_eq!("h", match_str("「〤p」", "h"));
    }

    // ===== EndTest =====
    /// Ported from: `EndTest.java`
    #[test]
    fn end_test1() {
        let cut = build("ab");
        assert!(cut.match_text(&TextNavigator::build("ab"), 0).start_match());
    }

    #[test]
    fn end_test2() {
        let cut = build("ab");
        assert!(cut.match_text(&TextNavigator::build("abc"), 0).start_match());
    }

    #[test]
    fn end_test3() {
        let cut = build("ab 〒$");
        assert!(cut.match_text(&TextNavigator::build("ab"), 0).start_match());
    }

    #[test]
    fn end_test4() {
        let cut = build("ab 〒$");
        assert!(!cut.match_text(&TextNavigator::build("abc"), 0).start_match());
    }

    // ===== CharClassTest =====
    /// Ported from: `CharClassTest.java`
    #[test]
    fn char_class_test1() {
        assert_eq!("h", match_str("〴w", "h"));
    }

    #[test]
    fn char_class_test2() {
        let cut = build("〇+〴w");
        assert_eq!("aa", cut.match_text(&TextNavigator::build("aaé"), 0).get_accepted_match());
        assert_eq!("h", cut.match_text(&TextNavigator::build("h"), 0).get_accepted_match());
        assert_eq!("ab", cut.match_text(&TextNavigator::build("ab"), 0).get_accepted_match());
        assert_eq!("abc", cut.match_text(&TextNavigator::build("abc"), 0).get_accepted_match());
    }

    // ===== LookAheadTest =====
    /// Ported from: `LookAheadTest.java`
    #[test]
    fn look_ahead_test1() {
        assert_eq!("ab", match_str("ab", "ab"));
    }

    #[test]
    fn look_ahead_test2() {
        assert_eq!("ab", match_str("ab", "abc"));
    }

    #[test]
    fn look_ahead_test3() {
        let cut = build("a 〒=b");
        assert_eq!("a", cut.match_str("abc", 0).get_accepted_match());
        assert_eq!("a", cut.match_str("ab", 0).get_accepted_match());
        assert_eq!("", cut.match_str("a", 0).get_accepted_match());
        assert_eq!("", cut.match_str("aa", 0).get_accepted_match());
    }

    #[test]
    fn look_ahead_test4() {
        let cut = build("a 〒!b");
        assert_eq!("", cut.match_str("abc", 0).get_accepted_match());
        assert_eq!("", cut.match_str("ab", 0).get_accepted_match());
        assert_eq!("a", cut.match_str("a", 0).get_accepted_match());
        assert_eq!("a", cut.match_str("aa", 0).get_accepted_match());
    }

    // ===== LookAheadTest2 =====
    /// Ported from: `LookAheadTest2.java`
    #[test]
    fn look_ahead2_test1a() {
        let cut = build("〇+〘「〤|」  〒=Z 〙");
        assert_eq!("b", cut.match_text(&TextNavigator::build("bZ"), 0).get_accepted_match());
    }

    #[test]
    fn look_ahead2_test4a() {
        let cut = build("〇+〘「〤|」  〒=Z 〙");
        assert_eq!("bZ", cut.match_text(&TextNavigator::build("bZZ"), 0).get_accepted_match());
    }

    #[test]
    fn look_ahead2_test4() {
        let cut = build("〇+〘「〤|」  〒!Z 〙");
        assert_eq!("bb", cut.match_text(&TextNavigator::build("bbbZ"), 0).get_accepted_match());
    }

    #[test]
    fn look_ahead2_test5() {
        let cut = build("〇+〘「〤|」  〒!〘a | 〙 〙");
        assert_eq!("bbbZ", cut.match_text(&TextNavigator::build("bbbZ"), 0).get_accepted_match());
    }

    #[test]
    fn look_ahead2_test6() {
        let cut = build("〇+〘「〤|」  〒!〘a | 〙 〙");
        assert_eq!("bbbZ", cut.match_text(&TextNavigator::build("bbbZ|"), 0).get_accepted_match());
    }

    #[test]
    fn look_ahead2_test7() {
        let cut = build("〇+〘「〤|」  〒!〘a | 〙 〙");
        assert_eq!("babbZ", cut.match_text(&TextNavigator::build("babbZ|"), 0).get_accepted_match());
    }

    #[test]
    fn look_ahead2_test8() {
        let cut = build("〇+〘「〤|」  〒!〘〇+a | 〙 〙");
        assert_eq!("12", cut.match_text(&TextNavigator::build("123a|"), 0).get_accepted_match());
    }

    // ===== LookAheadTest3 =====
    /// Ported from: `LookAheadTest3.java`
    #[test]
    fn look_ahead3_test1() {
        let cut = build("a【 ; ┇ 〒!「〴w;:.」 】");
        assert_eq!("a;", cut.match_str("a;", 0).get_accepted_match());
    }

    #[test]
    fn look_ahead3_test2() {
        let cut = build("a【 ; ┇ 〒!「〴w;:.」 】");
        assert_eq!("a", cut.match_str("a,", 0).get_accepted_match());
    }

    #[test]
    fn look_ahead3_test3() {
        let cut = build("a【 ; ┇ 〒!「〴w;:.」 】");
        assert_eq!("", cut.match_str("a.", 0).get_accepted_match());
    }

    // ===== LookBehindTest =====
    /// Ported from: `LookBehindTest.java`
    #[test]
    fn look_behind_test1() {
        let cut = build("ab");
        assert!(cut.match_text(&TextNavigator::build("ab"), 0).start_match());
    }

    #[test]
    fn look_behind_test2() {
        let cut = build("abc");
        assert!(cut.match_text(&TextNavigator::build("abc"), 0).start_match());
    }

    #[test]
    fn look_behind_test31() {
        let cut = build("〇*「〤_」  _");
        assert!(cut.match_text(&TextNavigator::build("abcd_"), 0).start_match());
    }

    #[test]
    fn look_behind_test32() {
        let cut = build("〇*「〤_」  〒<!〴d  _");
        assert!(cut.match_text(&TextNavigator::build("abcd_"), 0).start_match());
        assert!(cut.match_text(&TextNavigator::build("_"), 0).start_match());
        assert!(!cut.match_text(&TextNavigator::build("0_"), 0).start_match());
    }

    #[test]
    fn look_behind_test4() {
        let cut = build("〇*「〤_」  〒<=〴d  _");
        assert!(!cut.match_text(&TextNavigator::build("abcd_"), 0).start_match());
        assert!(!cut.match_text(&TextNavigator::build("_"), 0).start_match());
        assert!(cut.match_text(&TextNavigator::build("0_"), 0).start_match());
    }

    #[test]
    fn look_behind_test5() {
        let cut = build("〒<!「〴w」 _");
        assert!(cut.match_text(&TextNavigator::build("abc._"), 4).start_match());
        assert!(!cut.match_text(&TextNavigator::build("abcd_"), 4).start_match());
        assert!(!cut.match_text(&TextNavigator::build("d_d"), 1).start_match());
        assert!(cut.match_text(&TextNavigator::build("_"), 0).start_match());
        assert!(!cut.match_text(&TextNavigator::build("._"), 0).start_match());
        assert!(cut.match_text(&TextNavigator::build("._"), 1).start_match());
        assert!(!cut.match_text(&TextNavigator::build("0_"), 1).start_match());
    }

    // ===== TableTest =====
    /// Ported from: `TableTest.java`
    #[test]
    fn table_test1() {
        let cut = build("| 〇+〘 〇*〴s  〶$cell=〇*「〤|」  〇*〴s  | 〙");
        assert_eq!("|a|", cut.match_text(&TextNavigator::build("|a|"), 0).get_accepted_match());
        assert_eq!("[\"a\"]", values_to_string(&cut.match_text(&TextNavigator::build("|a|"), 0).find_values_by_key("cell")));
    }

    // ===== UpToTest =====
    /// Ported from: `UpToTest.java`
    #[test]
    fn up_to_test10() {
        assert_eq!("a000000001", match_str("a 〄>1", "a000000001"));
    }

    #[test]
    fn up_to_test11() {
        assert_eq!("a000000001", match_str("a 〄>1", "a000000001"));
    }

    #[test]
    fn up_to_test20() {
        let cut = build("a 〶$cell=〄>1");
        assert_eq!("a000000001", cut.match_text(&TextNavigator::build("a000000001"), 0).get_accepted_match());
        assert_eq!("[\"00000000\"]", values_to_string(&cut.match_text(&TextNavigator::build("a000000001"), 0).find_values_by_key("cell")));
    }

    // ===== UpToTest2 =====
    /// Ported from: `UpToTest2.java`
    #[test]
    fn up_to2_test1() {
        assert_eq!("abcdz", match_str("〄+〴w->z", "abcdzedf"));
    }

    #[test]
    fn up_to2_test2() {
        assert_eq!("abcdz", match_str("〄+〴w -> z", "abcdzedf"));
    }

    #[test]
    fn up_to2_test3() {
        let cut = build("〄+〶$txt=〴w -> z");
        assert_eq!("abcdz", cut.match_text(&TextNavigator::build("abcdzedf"), 0).get_accepted_match());
        assert_eq!("[\"a\", \"b\", \"c\", \"d\"]", values_to_string(&cut.match_text(&TextNavigator::build("abcdzedf"), 0).find_values_by_key("txt")));
    }

    #[test]
    fn up_to2_test4() {
        let cut = build("〶$txt=〄+〴w -> z");
        assert_eq!("[\"abcd\"]", values_to_string(&cut.match_text(&TextNavigator::build("abcdzedf"), 0).find_values_by_key("txt")));
    }

    #[test]
    fn up_to2_test5() {
        let cut = build("Z〶$txt=〄+〴w -> Z");
        assert_eq!("ZabcdZ", cut.match_text(&TextNavigator::build("ZabcdZedf"), 0).get_accepted_match());
        assert_eq!("[\"abcd\"]", values_to_string(&cut.match_text(&TextNavigator::build("ZabcdZedf"), 0).find_values_by_key("txt")));
    }

    #[test]
    fn up_to2_test6() {
        let cut = build("〇+〘 Z〶$txt=〄+〴w -> Z 〇*〴s 〙");
        assert_eq!("ZabcdZ", cut.match_text(&TextNavigator::build("ZabcdZedf"), 0).get_accepted_match());
        assert_eq!("[\"abcd\"]", values_to_string(&cut.match_text(&TextNavigator::build("ZabcdZedf"), 0).find_values_by_key("txt")));
    }

    #[test]
    fn up_to2_test7() {
        let cut = build("〇+〘 Z〶$txt=〄+〴w -> Z 〇*〴s 〙");
        assert_eq!("ZabcdZ ZedfZ", cut.match_text(&TextNavigator::build("ZabcdZ ZedfZ"), 0).get_accepted_match());
        assert_eq!("[\"abcd\", \"edf\"]", values_to_string(&cut.match_text(&TextNavigator::build("ZabcdZ ZedfZ"), 0).find_values_by_key("txt")));
    }

    #[test]
    fn up_to2_test8() {
        let cut = build("〇+〘 Z〄+〶$txt=〴w -> Z 〇*〴s 〙");
        assert_eq!("ZabcdZ ZedfZ", cut.match_text(&TextNavigator::build("ZabcdZ ZedfZ"), 0).get_accepted_match());
        assert_eq!("[\"a\", \"b\", \"c\", \"d\", \"e\", \"d\", \"f\"]", values_to_string(&cut.match_text(&TextNavigator::build("ZabcdZ ZedfZ"), 0).find_values_by_key("txt")));
    }

    // ===== UrlUbrexTest =====
    /// Ported from: `UrlUbrexTest.java`
    fn build_part(parts: &[&str]) -> UnicodeBracketedExpression {
        let combined: String = parts.iter().copied().collect();
        build(&combined)
    }

    #[test]
    fn url_test_tooltip() {
        let cut = build_part(&["〇?〘 〇*〴s { 〇*「〤{}」 } 〙"]);
        assert_eq!("{tooltip}", cut.match_text(&TextNavigator::build("{tooltip}"), 0).get_accepted_match());
    }

    #[test]
    fn url_test_s_quoted() {
        let cut = build_part(&[
            "[[ 〇*〴s",
            "〃  〇+「〤〃」 〃",
            "〇?〘 〇*〴s { 〇*「〤{}」 } 〙",
            "〇?〘 〴s 「〤〴s{}[]」  〇*「〤[]」 〙",
            "〇*〴s  ]]",
        ]);
        assert_eq!("[[\"hello1\"]]", cut.match_text(&TextNavigator::build("[[\"hello1\"]]"), 0).get_accepted_match());
        assert_eq!("[[\"he llo2\"]]", cut.match_text(&TextNavigator::build("[[\"he llo2\"]]"), 0).get_accepted_match());
        assert_eq!("[[\"he llo3\"]]", cut.match_text(&TextNavigator::build("[[\"he llo3\"]]xx"), 0).get_accepted_match());
        assert_eq!("", cut.match_text(&TextNavigator::build("x[[\"he llo4\"]]xxx"), 0).get_accepted_match());
        assert_eq!("", cut.match_text(&TextNavigator::build("[x[\"he llo5\"]]xxx"), 0).get_accepted_match());
        assert_eq!("[[\"he llo6\"{tooltip}]]", cut.match_text(&TextNavigator::build("[[\"he llo6\"{tooltip}]]"), 0).get_accepted_match());
    }

    #[test]
    fn url_test_s_only_tooltip() {
        let cut = build_part(&[
            "[[ 〇*〴s",
            "{  〇+「〤{}」 }",
            "〇*〴s  ]]",
        ]);
        assert_eq!("[[ { hello1  }  ]]", cut.match_text(&TextNavigator::build("[[ { hello1  }  ]]"), 0).get_accepted_match());
        assert_eq!("[[{hello1}]]", cut.match_text(&TextNavigator::build("[[{hello1}]]xx"), 0).get_accepted_match());
    }

    #[test]
    fn url_test_s_only_tooltip_and_label() {
        let cut = build_part(&[
            "[[ 〇*〴s",
            "{  〇+「〤{}」 }",
            "〇*〴s",
            "「〤〴s[]{}」 〇*「〤[]」 ",
            "〇*〴s  ]]",
        ]);
        assert_eq!("[[{hello1}toto]]", cut.match_text(&TextNavigator::build("[[{hello1}toto]]"), 0).get_accepted_match());
        assert_eq!("[[{hello2}   toto{}  ]]", cut.match_text(&TextNavigator::build("[[{hello2}   toto{}  ]]"), 0).get_accepted_match());
    }

    #[test]
    fn url_test_s_link_tooltip_nolabel() {
        let cut = build_part(&[
            "[[ 〇*〴s",
            "〇+「〤〴s〴g{}[]」",
            "〇*〴s",
            "{  〇*「〤{}」  } ",
            "〇*〴s  ]]",
        ]);
        assert_eq!("[[http://toto1 {foo}]]", cut.match_text(&TextNavigator::build("[[http://toto1 {foo}]]"), 0).get_accepted_match());
    }

    #[test]
    fn url_test_s_link_with_optional_tooltip_with_optional_label() {
        let cut = build_part(&[
            "[[ 〇*〴s",
            "〇+「〤〴s〴g[]」",
            "〇?〘  〇*〴s  {  〇*「〤{}」  } 〙",
            "〇?〘  〴s   「〤〴s{}[]」  〇*「〤[]」   〙",
            "〇*〴s  ]]",
        ]);
        assert_eq!("[[http://toto1]]", cut.match_text(&TextNavigator::build("[[http://toto1]]"), 0).get_accepted_match());
        assert_eq!("[[http://toto2 {foo}]]", cut.match_text(&TextNavigator::build("[[http://toto2 {foo}]]"), 0).get_accepted_match());
        assert_eq!("[[http://toto2 label]]", cut.match_text(&TextNavigator::build("[[http://toto2 label]]"), 0).get_accepted_match());
        assert_eq!("[[http://toto2 my label]]", cut.match_text(&TextNavigator::build("[[http://toto2 my label]]"), 0).get_accepted_match());
        assert_eq!("", cut.match_text(&TextNavigator::build("[[http://toto2  my label]]"), 0).get_accepted_match());
        assert_eq!("[[http://toto2{tooltip} my label]]", cut.match_text(&TextNavigator::build("[[http://toto2{tooltip} my label]]"), 0).get_accepted_match());
    }
}

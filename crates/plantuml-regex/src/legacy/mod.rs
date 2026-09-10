//! Legacy regex wrapper around the standard `regex` crate.
//!
//! Ported from `net.sourceforge.plantuml.regex` (Java).
//!
//! This module provides the `IRegex` trait and implementations (`RegexLeaf`,
//! `RegexConcat`, `RegexOr`, `RegexOptional`, `RegexRepeated*`) that wrap
//! standard regex patterns with PlantUML-specific macro expansion (`%s`,
//! `%q`, `%g`, `%pLN`) and fox-signature fast rejection.

pub mod fox_signature;
pub mod iregex;
pub mod matcher2;
pub mod matcher_iterator;
pub mod pattern2;
pub mod regex_composed;
pub mod regex_concat;
pub mod regex_leaf;
pub mod regex_optional;
pub mod regex_or;
pub mod regex_partial_match;
pub mod regex_repeated_one_or_more;
pub mod regex_repeated_zero_or_more;
pub mod regex_result;

pub use iregex::IRegex;
pub use matcher2::Matcher2;
pub use matcher_iterator::MatcherIterator;
pub use pattern2::Pattern2;
pub use regex_composed::RegexComposed;
pub use regex_concat::RegexConcat;
pub use regex_leaf::RegexLeaf;
pub use regex_optional::RegexOptional;
pub use regex_or::RegexOr;
pub use regex_partial_match::RegexPartialMatch;
pub use regex_repeated_one_or_more::RegexRepeatedOneOrMore;
pub use regex_repeated_zero_or_more::RegexRepeatedZeroOrMore;
pub use regex_result::RegexResult;

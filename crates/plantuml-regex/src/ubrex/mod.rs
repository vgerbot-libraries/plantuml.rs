//! ubrex regex engine using Unicode bracket notation.
//!
//! Ported from: `com/plantuml/ubrex/`
pub mod atomic_parser;
pub mod capture;
pub mod capture_entry;
pub mod capture_lookup;
pub mod case_mode;
pub mod challenge;
pub mod challenge_alternative;
pub mod challenge_char_class;
pub mod challenge_char_set;
pub mod challenge_end_of_text;
pub mod challenge_lazzy_one_or_more;
pub mod challenge_look_ahead;
pub mod challenge_look_behind;
pub mod challenge_one_or_more;
pub mod challenge_one_or_more_up_to_old_version;
pub mod challenge_optional;
pub mod challenge_repetition;
pub mod challenge_result;
pub mod challenge_single_char;
pub mod challenge_up_to;
pub mod challenge_zero_or_more;
pub mod char_class;
pub mod char_class_raw;
pub mod char_class_type;
pub mod char_set;
pub mod composite_list;
pub mod composite_named;
pub mod look_around;
pub mod repetition;
pub mod safe_list;
pub mod text_navigator;
pub mod u_matcher;
pub mod unicode_bracketed_expression;

pub mod builder;

// Re-exports
pub use atomic_parser::AtomicParser;
pub use capture::Capture;
pub use capture_entry::CaptureEntry;
pub use capture_lookup::CaptureLookup;
pub use case_mode::CaseMode;
pub use challenge::Challenge;
pub use challenge_alternative::ChallengeAlternative;
pub use challenge_char_class::ChallengeCharClass;
pub use challenge_char_set::ChallengeCharSet;
pub use challenge_end_of_text::ChallengeEndOfText;
pub use challenge_lazzy_one_or_more::ChallengeLazzyOneOrMore;
pub use challenge_look_ahead::ChallengeLookAhead;
pub use challenge_look_behind::ChallengeLookBehind;
pub use challenge_one_or_more::ChallengeOneOrMore;
pub use challenge_one_or_more_up_to_old_version::ChallengeOneOrMoreUpToOldVersion;
pub use challenge_optional::ChallengeOptional;
pub use challenge_repetition::ChallengeRepetition;
pub use challenge_result::ChallengeResult;
pub use challenge_single_char::ChallengeSingleChar;
pub use challenge_up_to::ChallengeUpTo;
pub use challenge_zero_or_more::ChallengeZeroOrMore;
pub use char_class::CharClass;
pub use char_class_raw::CharClassRaw;
pub use char_class_type::CharClassType;
pub use char_set::CharSet;
pub use composite_list::CompositeList;
pub use composite_named::CompositeNamed;
pub use look_around::LookAround;
pub use repetition::Repetition;
pub use safe_list::SafeList;
pub use text_navigator::TextNavigator;
pub use u_matcher::UMatcher;
pub use unicode_bracketed_expression::{build, from, UnicodeBracketedExpression};

pub use builder::{
    UBrexConcat, UBrexLeaf, UBrexNamed, UBrexOneOrMore, UBrexOptional, UBrexOr, UBrexPart,
    UBrexUpto, UBrexZeroOrMore,
};

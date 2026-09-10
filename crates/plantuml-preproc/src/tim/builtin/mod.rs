//! Builtin preprocessor functions ported from
//! `net.sourceforge.plantuml.tim.builtin`.
//!
//! Each builtin is a struct implementing [`crate::tim::t_function::TFunction`].
//! Most extend [`SimpleReturnFunction`] via the [`impl_simple_return_function!`]
//! macro. The function [`register_all`] adds every builtin to a
//! [`crate::tim::functions_set::FunctionsSet`], mirroring Java's
//! `TContext.addStandardFunctions`.

pub mod always_false;
pub mod always_true;
pub mod backslash;
pub mod bool_val;
pub mod breakline;
pub mod call_user_function;
pub mod chr;
pub mod darken;
pub mod date_function;
pub mod dec2hex;
pub mod dirpath;
pub mod dollar;
pub mod eval;
pub mod feature;
pub mod file_exists;
pub mod filedate;
pub mod filename;
pub mod filename_no_extension;
pub mod function_exists;
pub mod get_all_stdlib;
pub mod get_all_theme;
pub mod get_current_theme;
pub mod get_json_key;
pub mod get_json_type;
pub mod get_stdlib;
pub mod get_variable_value;
pub mod get_version;
pub mod getenv;
pub mod hex2dec;
pub mod hsl_color;
pub mod int_val;
pub mod invoke_procedure;
pub mod is_dark;
pub mod is_light;
pub mod json_add;
pub mod json_key_exists;
pub mod json_merge;
pub mod json_remove;
pub mod json_set;
pub mod left_align;
pub mod lighten;
pub mod load_json;
pub mod logical_and;
pub mod logical_nand;
pub mod logical_nor;
pub mod logical_not;
pub mod logical_nxor;
pub mod logical_or;
pub mod logical_xor;
pub mod lower;
pub mod modulo;
pub mod newline;
pub mod newline_short;
pub mod now;
pub mod ord;
pub mod percent;
pub mod random_function;
pub mod retrieve_procedure;
pub mod reverse_color;
pub mod reverse_hsluv_color;
pub mod right_align;
pub mod set_variable_value;
pub mod simple_return_function;
pub mod size;
pub mod split_str;
pub mod split_str_regex;
pub mod str2json;
pub mod string_function;
pub mod strlen;
pub mod strpos;
pub mod substr;
pub mod tabulation;
pub mod upper;
pub mod variable_exists;
pub mod xargs;

pub use simple_return_function::SimpleReturnFunction;

// Re-export the macro so submodules can use it without the `crate::` prefix.
pub use crate::impl_simple_return_function;

/// Jaws character constants ported from `net.sourceforge.plantuml.jaws.Jaws`.
///
/// These are private-use-area Unicode code points used as sentinel characters
/// in the preprocessor output.
pub mod jaws {
    /// `\u{E100}` — newline sentinel.
    pub const BLOCK_E1_NEWLINE: char = '\u{E100}';
    /// `\u{E101}` — left-align newline sentinel.
    pub const BLOCK_E1_NEWLINE_LEFT_ALIGN: char = '\u{E101}';
    /// `\u{E102}` — right-align newline sentinel.
    pub const BLOCK_E1_NEWLINE_RIGHT_ALIGN: char = '\u{E102}';
    /// `\u{E103}` — breakline sentinel.
    pub const BLOCK_E1_BREAKLINE: char = '\u{E103}';
    /// `\u{E110}` — real backslash sentinel.
    pub const BLOCK_E1_REAL_BACKSLASH: char = '\u{E110}';
    /// `\u{E111}` — real tabulation sentinel.
    pub const BLOCK_E1_REAL_TABULATION: char = '\u{E111}';
}

/// Jaws flags ported from `net.sourceforge.plantuml.jaws.JawsFlags`.
pub mod jaws_flags {
    /// Whether to use BLOCK_E1 sentinels in newline functions.
    pub const USE_BLOCK_E1_IN_NEWLINE_FUNCTION: bool = true;
}

/// Registers all builtin functions into the given [`FunctionsSet`].
///
/// Ported from `TContext.addStandardFunctions(Defines)`.
///
/// `defines` provides environment values for `Dirpath`, `Filedate`,
/// `Filename`, and `FilenameNoExtension`.
pub fn register_all(
    functions_set: &mut crate::tim::functions_set::FunctionsSet,
    defines: &crate::preproc::defines::Defines,
) {
    use crate::tim::t_function::TFunction;

    let builtins: Vec<Box<dyn TFunction>> = vec![
        Box::new(always_false::AlwaysFalse),
        Box::new(always_true::AlwaysTrue),
        Box::new(backslash::Backslash),
        Box::new(bool_val::BoolVal),
        Box::new(breakline::Breakline),
        Box::new(call_user_function::CallUserFunction),
        Box::new(chr::Chr),
        Box::new(darken::Darken),
        Box::new(date_function::DateFunction),
        Box::new(dec2hex::Dec2hex),
        Box::new(dirpath::Dirpath::new(defines)),
        Box::new(dollar::Dollar),
        Box::new(eval::Eval),
        Box::new(feature::Feature),
        Box::new(filedate::Filedate::new(defines)),
        Box::new(file_exists::FileExists),
        Box::new(filename::Filename::new(defines)),
        Box::new(filename_no_extension::FilenameNoExtension::new(defines)),
        Box::new(function_exists::FunctionExists),
        Box::new(get_all_stdlib::GetAllStdlib),
        Box::new(get_all_theme::GetAllTheme),
        Box::new(get_current_theme::GetCurrentTheme),
        Box::new(get_json_key::GetJsonKey),
        Box::new(get_json_type::GetJsonType),
        Box::new(get_stdlib::GetStdlib),
        Box::new(get_variable_value::GetVariableValue),
        Box::new(get_version::GetVersion),
        Box::new(getenv::Getenv),
        Box::new(hex2dec::Hex2dec),
        Box::new(hsl_color::HslColor),
        Box::new(int_val::IntVal),
        Box::new(invoke_procedure::InvokeProcedure),
        Box::new(is_dark::IsDark),
        Box::new(is_light::IsLight),
        Box::new(json_add::JsonAdd),
        Box::new(json_key_exists::JsonKeyExists),
        Box::new(json_merge::JsonMerge),
        Box::new(json_remove::JsonRemove),
        Box::new(json_set::JsonSet),
        Box::new(left_align::LeftAlign),
        Box::new(lighten::Lighten),
        Box::new(load_json::LoadJson),
        Box::new(logical_and::LogicalAnd),
        Box::new(logical_nand::LogicalNand),
        Box::new(logical_nor::LogicalNor),
        Box::new(logical_not::LogicalNot),
        Box::new(logical_nxor::LogicalNxor),
        Box::new(logical_or::LogicalOr),
        Box::new(logical_xor::LogicalXor),
        Box::new(lower::Lower),
        Box::new(modulo::Modulo),
        Box::new(newline::Newline),
        Box::new(newline_short::NewlineShort),
        Box::new(now::Now),
        Box::new(ord::Ord),
        Box::new(percent::Percent),
        Box::new(random_function::RandomFunction::new()),
        Box::new(retrieve_procedure::RetrieveProcedure),
        Box::new(reverse_color::ReverseColor),
        Box::new(reverse_hsluv_color::ReverseHsluvColor),
        Box::new(right_align::RightAlign),
        Box::new(set_variable_value::SetVariableValue),
        Box::new(size::Size),
        Box::new(split_str::SplitStr),
        Box::new(split_str_regex::SplitStrRegex),
        Box::new(str2json::Str2Json),
        Box::new(string_function::StringFunction),
        Box::new(strlen::Strlen),
        Box::new(strpos::Strpos),
        Box::new(substr::Substr),
        Box::new(tabulation::Tabulation),
        Box::new(upper::Upper),
        Box::new(variable_exists::VariableExists),
        Box::new(xargs::Xargs),
    ];

    for func in builtins {
        functions_set.add_function(func);
    }
}

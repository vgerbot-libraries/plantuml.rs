//! Helper functions for SkinParam.
//!
//! Split from `skin_param.rs` to keep the main file under 800 lines.

/// Returns `true` if the string contains only ASCII digits.
pub(crate) fn is_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Returns `true` if the string contains only ASCII digits and dots.
pub(crate) fn is_digits_or_dot(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Returns `true` if the string is a valid integer or decimal number.
pub(crate) fn is_int_or_decimal(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Removes surrounding double quotes from a string, if present.
pub(crate) fn remove_quotes(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

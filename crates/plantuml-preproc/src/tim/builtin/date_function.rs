//! Ported from `net.sourceforge.plantuml.tim.builtin.DateFunction`.

use std::sync::LazyLock;

use crate::tim::eater_exception::EaterException;
use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%date", 3));

/// `%date([format[, timestamp[, timezone]]])` — formats a date.
///
/// TODO: implement full date formatting. Currently returns a basic timestamp.
pub struct DateFunction;

impl SimpleReturnFunction for DateFunction {}

crate::impl_simple_return_function!(
    DateFunction,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 0 || nb_arg == 1 || nb_arg == 2 || nb_arg == 3,
    execute = |_self, _context, _memory, location, values, _named| {
        if values.is_empty() {
            // Return current time as a string
            let now = crate::wasm_time::now_secs();
            return Ok(TValue::from_string(now.to_string()));
        }
        let format = values[0].to_string();
        // TODO: implement SimpleDateFormat equivalent.
        // For now, return the format string as-is.
        let _ = format;
        Err(EaterException::new(
            "Bad date pattern".to_string(),
            location,
        ))
    },
);

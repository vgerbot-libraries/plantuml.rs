//! Ported from `net.sourceforge.plantuml.tim.builtin.Xargs`.

use std::sync::LazyLock;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%xargs", 0));

/// `%xargs()` — returns the extra arguments passed to the preprocessor.
pub struct Xargs;

impl SimpleReturnFunction for Xargs {}

crate::impl_simple_return_function!(
    Xargs,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, _values, _named| {
        // TODO: implement with context.get_xargs() once available on TContext.
        Ok(TValue::from_string(""))
    },
);

//! Ported from `net.sourceforge.plantuml.tim.builtin.FileExists`.

use std::sync::LazyLock;
use std::path::Path;

use crate::tim::expression::TValue;
use crate::tim::t_function_signature::TFunctionSignature;

use super::SimpleReturnFunction;

static SIGNATURE: LazyLock<TFunctionSignature> =
    LazyLock::new(|| TFunctionSignature::new("%file_exists", 1));

/// `%file_exists(path)` — returns true if the file exists.
///
/// TODO: implement with `SFile` security checks once the security module is ported.
pub struct FileExists;

impl SimpleReturnFunction for FileExists {}

crate::impl_simple_return_function!(
    FileExists,
    &*SIGNATURE,
    can_cover = |nb_arg, _named| nb_arg == 1,
    execute = |_self, _context, _memory, _location, values, _named| {
        let path = values[0].to_string();
        // TODO: implement with SFile security checks.
        Ok(TValue::from_boolean(Path::new(&path).exists()))
    },
);

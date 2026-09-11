//! Implementation of user-defined preprocessor functions.
//!
//! Ported from `net.sourceforge.plantuml.tim.TFunctionImpl`.

use std::collections::{HashMap, HashSet};

use super::eater_exception::EaterException;
use super::expression::TValue;
use super::t_context::TContext;
use super::t_function::TFunction;
use super::t_function_argument::TFunctionArgument;
use super::t_function_signature::TFunctionSignature;
use super::t_function_type::TFunctionType;
use super::t_memory::TMemory;
use crate::stubs::LineLocation;
use crate::{StringLocated, TLineType};

/// A user-defined preprocessor function (procedure, return function, or legacy define).
///
/// Ported from `net.sourceforge.plantuml.tim.TFunctionImpl`.
pub struct TFunctionImpl {
    signature: TFunctionSignature,
    args: Vec<TFunctionArgument>,
    body: Vec<StringLocated>,
    unquoted: bool,
    function_type: TFunctionType,
    legacy_definition: Option<String>,
    contains_return: bool,
}

impl TFunctionImpl {
    /// Creates a new `TFunctionImpl`.
    #[must_use]
    pub fn new(
        function_name: impl Into<String>,
        args: Vec<TFunctionArgument>,
        unquoted: bool,
        function_type: TFunctionType,
    ) -> Self {
        let names: HashSet<String> = args.iter().map(|a| a.get_name().to_string()).collect();
        Self {
            signature: TFunctionSignature::with_named_arguments(function_name, args.len() as i32, names),
            args,
            body: Vec::new(),
            unquoted,
            function_type,
            legacy_definition: None,
            contains_return: false,
        }
    }

    /// Adds a body line.
    pub fn add_body(&mut self, s: StringLocated) -> Result<(), EaterException> {

        let t = s.get_type();
        if t == TLineType::Return {
            self.contains_return = true;
            if self.function_type == TFunctionType::Procedure {
                return Err(EaterException::new(
                    "A procedure cannot have !return directive. Declare it as a function instead ?",
                    &s,
                ));
            }
        }
        self.body.push(s);
        Ok(())
    }

    /// Sets the legacy definition string.
    pub fn set_legacy_definition(&mut self, legacy_definition: impl Into<String>) {
        self.legacy_definition = Some(legacy_definition.into());
    }

    /// Returns `true` if the function has a body.
    #[must_use]
    pub fn has_body(&self) -> bool {
        !self.body.is_empty()
    }

    /// Finalizes a `!definelong` function — if body has exactly one line, convert to legacy define.
    ///
    /// Ported from `TFunctionImpl.finalizeEnddefinelong`.
    pub fn finalize_enddefinelong(&mut self) {
        if self.function_type != TFunctionType::LegacyDefineLong {
            return;
        }
        if self.body.len() == 1 {
            self.function_type = TFunctionType::LegacyDefine;
            self.legacy_definition = Some(self.body[0].get_string().to_string());
        }
    }

    /// Returns `true` if the function contains a `!return` directive.
    #[must_use]
    pub fn does_contain_return(&self) -> bool {
        self.contains_return
    }

    fn get_new_memory(
        &self,
        memory: &dyn TMemory,
        values: &[TValue],
        named_arguments: &HashMap<String, TValue>,
    ) -> Box<dyn TMemory> {
        let mut result: HashMap<String, TValue> = HashMap::new();
        let mut ivalue = 0;
        for arg in &self.args {
            let value = named_arguments.get(arg.get_name()).map_or_else(|| if ivalue < values.len() {
                let v = values[ivalue].clone();
                ivalue += 1;
                v
            } else if let Some(def) = arg.get_optional_default_value() {
                def.clone()
            } else {
                TValue::from_string("")
            }, std::clone::Clone::clone);
            result.insert(arg.get_name().to_string(), value);
        }
        memory.fork_from_global(result)
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn execute_return_legacy_define(
        &self,
        location: &LineLocation,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        args: &[TValue],
    ) -> TValue {
        let legacy_def = self.legacy_definition.as_deref().unwrap_or("");
        let mut copy = self.get_new_memory(memory, args, &HashMap::new());
        let tmp = context.apply_functions_and_variables(
            copy.as_mut(),
            &StringLocated::new(legacy_def, location.clone()),
        );
        TValue::from_string(tmp.unwrap_or_default())
    }
}

impl TFunction for TFunctionImpl {
    fn get_signature(&self) -> &TFunctionSignature {
        &self.signature
    }

    fn can_cover(&self, nb_arg: i32, named_arguments: &HashSet<String>) -> bool {
        for n in named_arguments {
            if !self.signature.get_named_arguments().contains(n) {
                return false;
            }
        }
        if nb_arg > self.args.len() as i32 {
            return false;
        }
        let mut needed_argument = 0;
        for arg in &self.args {
            if named_arguments.contains(arg.get_name()) {
                continue;
            }
            if arg.get_optional_default_value().is_none() {
                needed_argument += 1;
            }
        }
        nb_arg >= needed_argument
    }

    fn get_function_type(&self) -> TFunctionType {
        self.function_type
    }

    fn execute_return_function(
        &self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<TValue, EaterException> {
        if self.function_type == TFunctionType::LegacyDefine {
            return Ok(self.execute_return_legacy_define(location.get_location(), context, memory, args));
        }
        if self.function_type != TFunctionType::ReturnFunction {
            return Err(EaterException::new(
                "Illegal call here. Is there a return directive in your function?",
                location,
            ));
        }
        let mut copy = self.get_new_memory(memory, args, named);
        let result = context.execute_lines(copy.as_mut(), &self.body, Some(TFunctionType::ReturnFunction), true)?;
        result.ok_or_else(|| EaterException::new("No return directive found in your function", location))
    }

    fn execute_procedure_internal(
        &self,
        context: &mut TContext,
        memory: &mut dyn TMemory,
        location: &StringLocated,
        args: &[TValue],
        named: &HashMap<String, TValue>,
    ) -> Result<(), EaterException> {
        if self.function_type != TFunctionType::Procedure && self.function_type != TFunctionType::LegacyDefineLong {
            return Err(EaterException::new("Illegal call", location));
        }
        let mut copy = self.get_new_memory(memory, args, named);
        context.execute_lines(copy.as_mut(), &self.body, Some(TFunctionType::Procedure), false)?;
        Ok(())
    }

    fn is_unquoted(&self) -> bool {
        self.unquoted
    }
}

impl std::fmt::Display for TFunctionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FUNCTION {} {:?} {:?}", self.signature, self.args, self.function_type)
    }
}

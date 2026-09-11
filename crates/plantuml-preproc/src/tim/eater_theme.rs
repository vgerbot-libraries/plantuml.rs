//! Eater for `!theme` directives.
//!
//! Ported from `net.sourceforge.plantuml.tim.EaterTheme`.

use super::eater::Eater;
use super::eater_exception::EaterException;
use super::t_context::TContext;
use super::t_memory::TMemory;
use crate::stubs::{PathSystem, Theme, ThemeUtils};
use crate::StringLocated;

/// Parses `!theme` directives.
///
/// Ported from `net.sourceforge.plantuml.tim.EaterTheme`.
pub struct EaterTheme {
    eater: Eater,
    path_system: PathSystem,
    real_name: Option<String>,
    name: Option<String>,
    from: Option<String>,
}

impl EaterTheme {
    /// Creates a new `EaterTheme`.
    #[must_use]
    pub fn new(s: StringLocated, path_system: PathSystem) -> Self {
        Self {
            eater: Eater::new(s),
            path_system,
            real_name: None,
            name: None,
            from: None,
        }
    }

    /// Analyzes the `!theme` directive.
    pub fn analyze(&mut self, context: &mut TContext, memory: &mut dyn TMemory) -> Result<(), EaterException> {
        self.eater.skip_spaces();
        self.eater.check_and_eat_str("!theme")?;
        self.eater.skip_spaces();
        let name = self.eater.eat_all_to_end();
        let lower = name.to_lowercase();
        if let Some(x) = lower.find(" from ") {
            let from_tmp = name[x + " from ".len()..].trim().to_string();
            self.from = context.apply_functions_and_variables(
                memory,
                &StringLocated::new(from_tmp, self.eater.get_line_location()),
            );
            self.name = Some(name[..x].trim().to_string());
        } else {
            self.name = Some(name);
        }
        self.real_name = context.apply_functions_and_variables(
            memory,
            &StringLocated::new(self.name.clone().unwrap_or_default(), self.eater.get_line_location()),
        );
        Ok(())
    }

    /// Returns the loaded theme, or throws an error.
    ///
    /// Ported from `EaterTheme.getTheme`.
    pub fn get_theme(&self) -> Result<Theme, EaterException> {
        let real_name = self.real_name.as_deref().unwrap_or("");
        let from = self.from.as_deref();
        let theme = ThemeUtils::load_theme(&self.path_system, real_name, from, self.eater.get_string_located());
        theme.ok_or_else(|| {
            let location = if from.is_some() { format!(" in {}", from.unwrap_or("")) } else { String::new() };
            EaterException::new(
                format!("Cannot load theme {real_name}{location}"),
                self.eater.get_string_located(),
            )
        })
    }

    /// Returns the theme name.
    #[must_use]
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the path system.
    #[must_use]
    pub fn get_new_imported_files(&self) -> &PathSystem {
        &self.path_system
    }
}

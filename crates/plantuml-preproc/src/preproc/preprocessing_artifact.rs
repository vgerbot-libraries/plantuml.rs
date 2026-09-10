//! Artifacts produced during preprocessing (warnings, options).
//!
//! Ported from `net.sourceforge.plantuml.preproc.PreprocessingArtifact`.

use std::collections::HashSet;

use crate::preproc::configuration_store::ConfigurationStore;
use crate::preproc::option_key::OptionKey;
use crate::stubs::{Warning, WarningHandler};

/// Artifacts produced during preprocessing: warnings and configuration options.
///
/// Ported from `net.sourceforge.plantuml.preproc.PreprocessingArtifact`.
#[derive(Debug, Clone)]
pub struct PreprocessingArtifact {
    option: ConfigurationStore<OptionKey>,
    warnings: HashSet<Warning>,
}

impl Default for PreprocessingArtifact {
    fn default() -> Self {
        Self::new()
    }
}

impl PreprocessingArtifact {
    /// Creates a new empty `PreprocessingArtifact`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            option: ConfigurationStore::create_empty(),
            warnings: HashSet::new(),
        }
    }

    /// Returns the configuration option store.
    #[must_use]
    pub fn get_option(&self) -> &ConfigurationStore<OptionKey> {
        &self.option
    }

    /// Returns a mutable reference to the configuration option store.
    pub fn get_option_mut(&mut self) -> &mut ConfigurationStore<OptionKey> {
        &mut self.option
    }
}

impl WarningHandler for PreprocessingArtifact {
    fn add_warning(&mut self, warning: Warning) {
        self.warnings.insert(warning);
    }

    fn get_warnings(&self) -> Vec<Warning> {
        self.warnings.iter().cloned().collect()
    }
}

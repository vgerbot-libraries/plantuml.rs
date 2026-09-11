//! `TimLoader` — the preprocessor entry point.
//!
//! Ported from `net.sourceforge.plantuml.tim.TimLoader`.

use super::t_context::TContext;
use super::t_memory_global::TMemoryGlobal;
use crate::preproc::PreprocessingArtifact;
use crate::StringLocated;

/// The preprocessor entry point. Loads source text, processes directives,
// and produces the preprocessed output.
///
/// Ported from `net.sourceforge.plantuml.tim.TimLoader`.
pub struct TimLoader {
    context: TContext,
    global: TMemoryGlobal,
    preprocessor_error: bool,
    result_list: Option<Vec<StringLocated>>,
    preprocessing_artifact: Option<PreprocessingArtifact>,
}

impl TimLoader {
    /// Creates a new `TimLoader`.
    ///
    /// Ported from `TimLoader` constructor.
    /// Creates a new `TimLoader` with the given defines.
    ///
    /// Ported from `TimLoader` constructor.
    #[must_use]
    pub fn new(defines: &crate::preproc::defines::Defines) -> Self {
        Self {
            context: TContext::new(defines),
            global: TMemoryGlobal::new(),
            preprocessor_error: false,
            result_list: None,
            preprocessing_artifact: None,
        }
    }

    /// Sets the current directory for file resolution (used by `!includesub`).
    pub fn set_current_dir(&mut self, dir: impl Into<std::path::PathBuf>) {
        self.context.set_current_dir(dir);
    }


    /// Loads and processes the given source lines.
    ///
    /// Ported from `TimLoader.load`.
    pub fn load(&mut self, raw: Vec<StringLocated>) {
        if self.preprocessor_error {
            return;
        }
        let result = self.context.execute_lines(&mut self.global, &raw, None, false);
        match result {
            Ok(_) => {
                self.result_list = Some(self.context.get_result_list().to_vec());
                self.preprocessing_artifact = Some(self.context.get_preprocessing_artifact().clone());
            }
            Err(e) => {
                self.preprocessor_error = true;
                self.result_list = Some(vec![StringLocated::with_error(
                    e.get_message().to_string(),
                    e.get_location().get_location().clone(),
                    e.get_message().to_string(),
                )]);
            }
        }
    }

    /// Returns the preprocessed result list.
    ///
    /// Ported from `TimLoader.getResultList`.
    #[must_use]
    pub fn get_result_list(&self) -> Option<&Vec<StringLocated>> {
        self.result_list.as_ref()
    }

    /// Returns the debug output.
    ///
    /// Ported from `TimLoader.getDebug`.
    #[must_use]
    pub fn get_debug(&self) -> &[StringLocated] {
        self.context.get_debug()
    }

    /// Returns `true` if a preprocessor error occurred.
    ///
    /// Ported from `TimLoader.isPreprocessorError`.
    #[must_use]
    pub fn is_preprocessor_error(&self) -> bool {
        self.preprocessor_error
    }

    /// Returns the preprocessing artifact.
    ///
    /// Ported from `TimLoader.getPreprocessingArtifact`.
    #[must_use]
    pub fn get_preprocessing_artifact(&self) -> Option<&PreprocessingArtifact> {
        self.preprocessing_artifact.as_ref()
    }

    /// Returns the context (mutable).
    pub fn context_mut(&mut self) -> &mut TContext {
        &mut self.context
    }

    /// Returns the global memory (mutable).
    pub fn global_mut(&mut self) -> &mut TMemoryGlobal {
        &mut self.global
    }
}

impl Default for TimLoader {
    fn default() -> Self {
        Self::new(&crate::preproc::defines::Defines::default())
    }
}

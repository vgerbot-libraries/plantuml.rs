//! Function type for preprocessor functions.
//!
//! Ported from `net.sourceforge.plantuml.tim.TFunctionType`.

/// The type of a preprocessor function.
///
/// Ported from `net.sourceforge.plantuml.tim.TFunctionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TFunctionType {
    Procedure,
    ReturnFunction,
    LegacyDefine,
    LegacyDefineLong,
}

impl TFunctionType {
    /// Returns `true` if this is a legacy define type.
    ///
    /// Ported from `TFunctionType.isLegacy`.
    #[must_use]
    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::LegacyDefine | Self::LegacyDefineLong)
    }
}

//! `LinkArg` — link label and styling arguments.
//!
//! Ported from: `net/sourceforge/plantuml/abel/LinkArg.java`

use plantuml_klimt::Display;

/// Arguments for a link: label, length, quantifiers, roles, etc.
///
/// Ported from: `net/sourceforge/plantuml/abel/LinkArg.java`
#[derive(Debug, Clone, Default)]
pub struct LinkArg {
    label: Display,
    quantifier1: Option<String>,
    quantifier2: Option<String>,
    role1: Option<String>,
    role2: Option<String>,
    labeldistance: Option<String>,
    labelangle: Option<String>,
    kal1: Option<String>,
    kal2: Option<String>,
    length: i32,
}

impl LinkArg {
    /// Builds a `LinkArg` with the given label and length.
    ///
    /// Ported from: `LinkArg.build(Display, int)`.
    #[must_use]
    pub fn build(label: Display, length: i32) -> Self {
        Self {
            label,
            length,
            ..Default::default()
        }
    }

    /// Creates a `LinkArg` with no display (null label).
    ///
    /// Ported from: `LinkArg.noDisplay(int)`.
    #[must_use]
    pub fn no_display(length: i32) -> Self {
        Self {
            label: Display::NULL,
            length,
            ..Default::default()
        }
    }

    /// Returns a new `LinkArg` with quantifiers set.
    ///
    /// Ported from: `LinkArg.withQuantifier(String, String)`.
    #[must_use]
    pub fn with_quantifier(self, q1: impl Into<String>, q2: impl Into<String>) -> Self {
        Self {
            quantifier1: Some(q1.into()),
            quantifier2: Some(q2.into()),
            ..self
        }
    }

    /// Returns a new `LinkArg` with roles set.
    ///
    /// Ported from: `LinkArg.withRole(String, String)`.
    #[must_use]
    pub fn with_role(self, r1: impl Into<String>, r2: impl Into<String>) -> Self {
        Self {
            role1: Some(r1.into()),
            role2: Some(r2.into()),
            ..self
        }
    }

    /// Returns a new `LinkArg` with distance and angle set.
    ///
    /// Ported from: `LinkArg.withDistanceAngle(String, String)`.
    #[must_use]
    pub fn with_distance_angle(
        self,
        distance: impl Into<String>,
        angle: impl Into<String>,
    ) -> Self {
        Self {
            labeldistance: Some(distance.into()),
            labelangle: Some(angle.into()),
            ..self
        }
    }

    /// Returns the inverted `LinkArg` (swaps quantifiers, roles, kals).
    ///
    /// Ported from: `LinkArg.getInv()`.
    #[must_use]
    pub fn get_inv(&self) -> Self {
        Self {
            label: self.label.clone(),
            length: self.length,
            quantifier1: self.quantifier2.clone(),
            quantifier2: self.quantifier1.clone(),
            role1: self.role2.clone(),
            role2: self.role1.clone(),
            labeldistance: self.labeldistance.clone(),
            labelangle: self.labelangle.clone(),
            kal1: self.kal2.clone(),
            kal2: self.kal1.clone(),
        }
    }

    pub const fn get_label(&self) -> &Display {
        &self.label
    }

    pub const fn get_length(&self) -> i32 {
        self.length
    }

    pub const fn set_length(&mut self, length: i32) {
        self.length = length;
    }

    pub fn get_quantifier1(&self) -> Option<&str> {
        self.quantifier1.as_deref()
    }

    pub fn get_quantifier2(&self) -> Option<&str> {
        self.quantifier2.as_deref()
    }

    pub fn get_role1(&self) -> Option<&str> {
        self.role1.as_deref()
    }

    pub fn get_role2(&self) -> Option<&str> {
        self.role2.as_deref()
    }
}

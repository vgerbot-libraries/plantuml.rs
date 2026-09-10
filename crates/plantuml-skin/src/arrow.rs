//! Arrow types — arrow head, body, part, decoration, dressing, configuration.
//!
//! Ported from: `net/sourceforge/plantuml/skin/` package

use crate::is_skin_param::ArrowDirection;
use plantuml_klimt::HColor;

/// Arrow head style.
///
/// Ported from: `net/sourceforge/plantuml/skin/ArrowHead.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowHead {
    Normal,
    CrossX,
    Async,
    None,
}

/// Arrow body style.
///
/// Ported from: `net/sourceforge/plantuml/skin/ArrowBody.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowBody {
    Normal,
    Dotted,
    Dashed,
    Hidden,
    Bold,
}

/// Arrow part (full, top, bottom).
///
/// Ported from: `net/sourceforge/plantuml/skin/ArrowPart.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowPart {
    Full,
    TopPart,
    BottomPart,
}

/// Arrow decoration (none, circle).
///
/// Ported from: `net/sourceforge/plantuml/skin/ArrowDecoration.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDecoration {
    None,
    Circle,
}

/// Arrow dressing — head + part combination.
///
/// Ported from: `net/sourceforge/plantuml/skin/ArrowDressing.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowDressing {
    head: ArrowHead,
    part: ArrowPart,
}

impl ArrowDressing {
    #[must_use]
    pub fn create() -> Self {
        Self {
            head: ArrowHead::None,
            part: ArrowPart::Full,
        }
    }

    #[must_use]
    pub fn with_head(self, head: ArrowHead) -> Self {
        Self { head, part: self.part }
    }

    #[must_use]
    pub fn with_part(self, part: ArrowPart) -> Self {
        Self { head: self.head, part }
    }

    #[must_use]
    pub fn head(&self) -> ArrowHead {
        self.head
    }

    #[must_use]
    pub fn part(&self) -> ArrowPart {
        self.part
    }
}


/// Immutable arrow configuration.
///
/// Ported from: `net/sourceforge/plantuml/skin/ArrowConfiguration.java`
#[derive(Debug, Clone)]
pub struct ArrowConfiguration {
    body: ArrowBody,
    dressing1: ArrowDressing,
    dressing2: ArrowDressing,
    decoration1: ArrowDecoration,
    decoration2: ArrowDecoration,
    color: Option<HColor>,
    is_self: bool,
    thickness: f64,
    reverse_define: bool,
    inclination: i32,
}

impl ArrowConfiguration {
    /// Creates a normal left-to-right arrow.
    #[must_use]
    pub fn with_direction_normal() -> Self {
        Self {
            body: ArrowBody::Normal,
            dressing1: ArrowDressing::create(),
            dressing2: ArrowDressing::create().with_head(ArrowHead::Normal),
            decoration1: ArrowDecoration::None,
            decoration2: ArrowDecoration::None,
            color: None,
            is_self: false,
            thickness: 1.0,
            reverse_define: false,
            inclination: 0,
        }
    }

    /// Creates a bidirectional arrow.
    #[must_use]
    pub fn with_direction_both() -> Self {
        Self {
            body: ArrowBody::Normal,
            dressing1: ArrowDressing::create().with_head(ArrowHead::Normal),
            dressing2: ArrowDressing::create().with_head(ArrowHead::Normal),
            decoration1: ArrowDecoration::None,
            decoration2: ArrowDecoration::None,
            color: None,
            is_self: false,
            thickness: 1.0,
            reverse_define: false,
            inclination: 0,
        }
    }

    /// Creates a self-referencing arrow.
    #[must_use]
    pub fn with_direction_self(reverse_define: bool) -> Self {
        Self {
            body: ArrowBody::Normal,
            dressing1: ArrowDressing::create(),
            dressing2: ArrowDressing::create().with_head(ArrowHead::Normal),
            decoration1: ArrowDecoration::None,
            decoration2: ArrowDecoration::None,
            color: None,
            is_self: true,
            thickness: 1.0,
            reverse_define,
            inclination: 0,
        }
    }

    /// Reverses the arrow direction.
    #[must_use]
    pub fn reverse(&self) -> Self {
        Self {
            body: self.body,
            dressing1: self.dressing2,
            dressing2: self.dressing1,
            decoration1: self.decoration2,
            decoration2: self.decoration1,
            color: self.color.clone(),
            is_self: self.is_self,
            thickness: self.thickness,
            reverse_define: self.reverse_define,
            inclination: self.inclination,
        }
    }

    /// Returns a copy with the given body.
    #[must_use]
    pub fn with_body(&self, body: ArrowBody) -> Self {
        Self { body, ..self.clone() }
    }

    /// Returns a copy with the given head on both dressings.
    #[must_use]
    pub fn with_head(&self, head: ArrowHead) -> Self {
        let new_dressing1 = if self.dressing1.head() == ArrowHead::None {
            self.dressing1
        } else {
            self.dressing1.with_head(head)
        };
        let new_dressing2 = if self.dressing2.head() == ArrowHead::None {
            self.dressing2
        } else {
            self.dressing2.with_head(head)
        };
        Self {
            dressing1: new_dressing1,
            dressing2: new_dressing2,
            ..self.clone()
        }
    }

    /// Returns a copy with the given head on dressing1.
    #[must_use]
    pub fn with_head1(&self, head: ArrowHead) -> Self {
        Self {
            dressing1: self.dressing1.with_head(head),
            ..self.clone()
        }
    }

    /// Returns a copy with the given head on dressing2.
    #[must_use]
    pub fn with_head2(&self, head: ArrowHead) -> Self {
        Self {
            dressing2: self.dressing2.with_head(head),
            ..self.clone()
        }
    }

    /// Returns a copy with the given part.
    #[must_use]
    pub fn with_part(&self, part: ArrowPart) -> Self {
        if self.dressing2.head() != ArrowHead::None {
            Self {
                dressing2: self.dressing2.with_part(part),
                ..self.clone()
            }
        } else {
            Self {
                dressing1: self.dressing1.with_part(part),
                ..self.clone()
            }
        }
    }

    /// Returns a copy with the given decoration1.
    #[must_use]
    pub fn with_decoration1(&self, decoration1: ArrowDecoration) -> Self {
        Self {
            decoration1,
            ..self.clone()
        }
    }

    /// Returns a copy with the given decoration2.
    #[must_use]
    pub fn with_decoration2(&self, decoration2: ArrowDecoration) -> Self {
        Self {
            decoration2,
            ..self.clone()
        }
    }

    /// Returns a copy with the given color.
    #[must_use]
    pub fn with_color(&self, color: HColor) -> Self {
        Self {
            color: Some(color),
            ..self.clone()
        }
    }

    /// Returns a copy with the given thickness.
    #[must_use]
    pub fn with_thickness(&self, thickness: f64) -> Self {
        Self { thickness, ..self.clone() }
    }

    /// Returns a copy with reverseDefine toggled.
    #[must_use]
    pub fn reverse_define(&self) -> Self {
        Self {
            reverse_define: !self.reverse_define,
            ..self.clone()
        }
    }

    /// Returns a copy with the given inclination.
    #[must_use]
    pub fn with_inclination(&self, inclination: i32) -> Self {
        Self { inclination, ..self.clone() }
    }

    #[must_use]
    pub fn decoration1(&self) -> ArrowDecoration {
        self.decoration1
    }

    #[must_use]
    pub fn decoration2(&self) -> ArrowDecoration {
        self.decoration2
    }

    #[must_use]
    pub fn arrow_direction(&self) -> ArrowDirection {
        if self.is_self {
            return ArrowDirection::Self_;
        }
        if self.dressing1.head() == ArrowHead::None && self.dressing2.head() != ArrowHead::None {
            return ArrowDirection::LeftToRightNormal;
        }
        if self.dressing1.head() != ArrowHead::None && self.dressing2.head() == ArrowHead::None {
            return ArrowDirection::RightToLeftReverse;
        }
        ArrowDirection::BothDirection
    }

    #[must_use]
    pub fn is_self_arrow(&self) -> bool {
        self.arrow_direction() == ArrowDirection::Self_
    }

    #[must_use]
    pub fn is_dotted(&self) -> bool {
        self.body == ArrowBody::Dotted
    }

    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.body == ArrowBody::Hidden
    }

    #[must_use]
    pub fn head(&self) -> ArrowHead {
        if self.dressing2.head() != ArrowHead::None {
            self.dressing2.head()
        } else {
            self.dressing1.head()
        }
    }

    #[must_use]
    pub fn is_async1(&self) -> bool {
        self.dressing1.head() == ArrowHead::Async
    }

    #[must_use]
    pub fn is_async2(&self) -> bool {
        self.dressing2.head() == ArrowHead::Async
    }

    #[must_use]
    pub fn part(&self) -> ArrowPart {
        if self.dressing2.head() != ArrowHead::None {
            self.dressing2.part()
        } else {
            self.dressing1.part()
        }
    }

    #[must_use]
    pub fn color(&self) -> Option<&HColor> {
        self.color.as_ref()
    }

    #[must_use]
    pub fn dressing1(&self) -> ArrowDressing {
        self.dressing1
    }

    #[must_use]
    pub fn dressing2(&self) -> ArrowDressing {
        self.dressing2
    }

    #[must_use]
    pub fn is_reverse_define(&self) -> bool {
        self.reverse_define
    }

    #[must_use]
    pub fn thickness(&self) -> f64 {
        self.thickness
    }

    #[must_use]
    pub fn inclination1(&self) -> i32 {
        if self.dressing2.head() == ArrowHead::None || self.dressing2.head() == ArrowHead::CrossX {
            self.inclination
        } else {
            0
        }
    }

    #[must_use]
    pub fn inclination2(&self) -> i32 {
        if self.dressing1.head() == ArrowHead::None || self.dressing1.head() == ArrowHead::CrossX {
            self.inclination
        } else {
            0
        }
    }
}

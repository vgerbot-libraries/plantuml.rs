//! Component types and drawing context.
//!
//! Ported from: `net/sourceforge/plantuml/skin/` package

/// Component type — identifies what kind of skin component to render.
///
/// Ported from: `net/sourceforge/plantuml/skin/ComponentType.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Arrow,
    ActorHead,
    ActorTail,
    BoundaryHead,
    BoundaryTail,
    ControlHead,
    ControlTail,
    EntityHead,
    EntityTail,
    QueueHead,
    QueueTail,
    DatabaseHead,
    DatabaseTail,
    CollectionsHead,
    CollectionsTail,
    ActivationBoxCloseClose,
    ActivationBoxCloseOpen,
    ActivationBoxOpenClose,
    ActivationBoxOpenOpen,
    DelayText,
    Destroy,
    DelayLine,
    ParticipantLine,
    GroupingElseLegacy,
    GroupingElseTeoz,
    GroupingHeaderLegacy,
    GroupingHeaderTeoz,
    GroupingSpace,
    Newpage,
    Note,
    NoteHexagonal,
    NoteBox,
    Divider,
    Reference,
    Englober,
    ParticipantHead,
    ParticipantTail,
}

impl ComponentType {
    /// Returns `true` if this is an arrow component.
    #[must_use]
    pub const fn is_arrow(&self) -> bool {
        matches!(self, Self::Arrow)
    }
}

/// Drawing area with dimension and text delta.
///
/// Ported from: `net/sourceforge/plantuml/skin/Area.java`
#[derive(Debug, Clone, Copy)]
pub struct Area {
    dimension_to_use: (f64, f64),
    text_delta_x: f64,
}

impl Area {
    /// Creates a new area with the given width and height.
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            dimension_to_use: (width, height),
            text_delta_x: 0.0,
        }
    }

    /// Creates a new area with width, height, and text delta X.
    #[must_use]
    pub fn with_text_delta(width: f64, height: f64, text_delta_x: f64) -> Self {
        Self {
            dimension_to_use: (width, height),
            text_delta_x,
        }
    }

    #[must_use]
    pub const fn width(&self) -> f64 {
        self.dimension_to_use.0
    }

    #[must_use]
    pub const fn height(&self) -> f64 {
        self.dimension_to_use.1
    }

    #[must_use]
    pub const fn text_delta_x(&self) -> f64 {
        self.text_delta_x
    }
}

/// 2D drawing context — indicates whether we're drawing background or foreground.
///
/// Ported from: `net/sourceforge/plantuml/skin/Context2D.java`
pub trait Context2D {
    /// Returns `true` if this is a background drawing pass.
    fn is_background(&self) -> bool;
}

/// Simple Context2D implementation.
///
/// Ported from: `net/sourceforge/plantuml/skin/SimpleContext2D.java`
#[derive(Debug, Clone, Copy)]
pub struct SimpleContext2D {
    background: bool,
}

impl SimpleContext2D {
    #[must_use]
    pub const fn new(background: bool) -> Self {
        Self { background }
    }
}

impl Context2D for SimpleContext2D {
    fn is_background(&self) -> bool {
        self.background
    }
}

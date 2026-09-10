//! Placeholder types for Java types not yet ported.
//!
//! These are minimal stubs that will be replaced by full ports in later phases.
//! Split from `is_skin_param.rs` to keep the main file under 800 lines.

use plantuml_klimt::HColor;

use crate::clockwise_top_right_bottom_left::ClockwiseTopRightBottomLeft;
use crate::value::HorizontalAlignment;

/// Stereotype placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Stereotype {
    pub labels: Vec<String>,
}

impl Stereotype {
    #[must_use]
    pub fn new(labels: Vec<String>) -> Self {
        Self { labels }
    }

    #[must_use]
    pub fn empty() -> Self {
        Self { labels: Vec::new() }
    }

    #[must_use]
    pub fn get_multiple_labels(&self) -> &[String] {
        &self.labels
    }

    #[must_use]
    pub fn get_label(&self, _separator: &str) -> String {
        self.labels.join("<<").to_string()
    }
}

/// Color parameter placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorParam {
    Background,
    ArrowHead,
    Hyperlink,
    ActivityBorder,
    ActivityBackground,
    NoteBorder,
    NoteBackground,
    ParticipantBorder,
    ParticipantBackground,
    SequenceParticipantBorder,
    SequenceParticipantBackground,
    SequenceLifeLineBorder,
    SequenceLifeLineBackground,
    SequenceGroupBorder,
    SequenceGroupBackground,
    SequenceGroupHeaderBackground,
    ArrowColor,
    StateBorder,
    StateBackground,
    ClassBorder,
    ClassBackground,
    PackageBorder,
    PackageBackground,
}

impl ColorParam {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::ArrowHead => "arrowHead",
            Self::Hyperlink => "hyperlink",
            Self::ActivityBorder => "activityBorder",
            Self::ActivityBackground => "activityBackground",
            Self::NoteBorder => "noteBorder",
            Self::NoteBackground => "noteBackground",
            Self::ParticipantBorder => "participantBorder",
            Self::ParticipantBackground => "participantBackground",
            Self::SequenceParticipantBorder => "sequenceParticipantBorder",
            Self::SequenceParticipantBackground => "sequenceParticipantBackground",
            Self::SequenceLifeLineBorder => "sequenceLifeLineBorder",
            Self::SequenceLifeLineBackground => "sequenceLifeLineBackground",
            Self::SequenceGroupBorder => "sequenceGroupBorder",
            Self::SequenceGroupBackground => "sequenceGroupBackground",
            Self::SequenceGroupHeaderBackground => "sequenceGroupHeaderBackground",
            Self::ArrowColor => "arrowColor",
            Self::StateBorder => "stateBorder",
            Self::StateBackground => "stateBackground",
            Self::ClassBorder => "classBorder",
            Self::ClassBackground => "classBackground",
            Self::PackageBorder => "packageBorder",
            Self::PackageBackground => "packageBackground",
        }
    }
}

/// Font parameter placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontParam {
    CircledCharacter,
    Default,
    Title,
    Caption,
    Legend,
    Header,
    Footer,
    Participant,
    SequenceParticipant,
    SequenceMessage,
    SequenceGroupHeader,
    SequenceLifeLine,
    Note,
    Activity,
    State,
    Class,
    Package,
    Arrow,
}

impl FontParam {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::CircledCharacter => "CIRCLED_CHARACTER",
            Self::Default => "DEFAULT",
            Self::Title => "TITLE",
            Self::Caption => "CAPTION",
            Self::Legend => "LEGEND",
            Self::Header => "HEADER",
            Self::Footer => "FOOTER",
            Self::Participant => "PARTICIPANT",
            Self::SequenceParticipant => "SEQUENCE_PARTICIPANT",
            Self::SequenceMessage => "SEQUENCE_MESSAGE",
            Self::SequenceGroupHeader => "SEQUENCE_GROUP_HEADER",
            Self::SequenceLifeLine => "SEQUENCE_LIFE_LINE",
            Self::Note => "NOTE",
            Self::Activity => "ACTIVITY",
            Self::State => "STATE",
            Self::Class => "CLASS",
            Self::Package => "PACKAGE",
            Self::Arrow => "ARROW",
        }
    }

    #[must_use]
    pub fn default_family(self) -> &'static str {
        "sans-serif"
    }

    #[must_use]
    pub fn default_size(self) -> i32 {
        14
    }

    #[must_use]
    pub fn default_color(self) -> Option<&'static str> {
        None
    }
}

/// Line parameter placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineParam {
    Activity,
    Class,
    Component,
    Object,
    Sequence,
    State,
    Usecase,
    Arrow,
}

impl LineParam {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Class => "class",
            Self::Component => "component",
            Self::Object => "object",
            Self::Sequence => "sequence",
            Self::State => "state",
            Self::Usecase => "usecase",
            Self::Arrow => "arrow",
        }
    }
}

/// Alignment parameter placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignmentParam {
    SequenceMessageAlignment,
    SequenceMessageTextAlignment,
    NoteTextAlignment,
    StateMessageAlignment,
    DefaultTextAlignment,
}

impl AlignmentParam {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SequenceMessageAlignment => "sequenceMessageAlignment",
            Self::SequenceMessageTextAlignment => "sequenceMessageTextAlignment",
            Self::NoteTextAlignment => "noteTextAlignment",
            Self::StateMessageAlignment => "stateMessageAlignment",
            Self::DefaultTextAlignment => "defaultTextAlignment",
        }
    }

    #[must_use]
    pub fn get_default_value(self) -> HorizontalAlignment {
        match self {
            Self::SequenceMessageAlignment => HorizontalAlignment::Left,
            Self::SequenceMessageTextAlignment => HorizontalAlignment::Left,
            Self::NoteTextAlignment => HorizontalAlignment::Left,
            Self::StateMessageAlignment => HorizontalAlignment::Center,
            Self::DefaultTextAlignment => HorizontalAlignment::Left,
        }
    }
}

/// Arrow direction placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowDirection {
    LeftToRightNormal,
    RightToLeftReverse,
    BothDirection,
    Self_,
}

/// Corner parameter placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerParam {
    Default,
    TitleBorder,
    Note,
    Activity,
    Class,
    State,
}

impl CornerParam {
    #[must_use]
    pub fn get_round_key(self) -> &'static str {
        match self {
            Self::Default => "roundcorner",
            Self::TitleBorder => "titleborderroundcorner",
            Self::Note => "noteroundcorner",
            Self::Activity => "activityroundcorner",
            Self::Class => "classroundcorner",
            Self::State => "stateroundcorner",
        }
    }

    #[must_use]
    pub fn get_diagonal_key(self) -> &'static str {
        match self {
            Self::Default => "diagonalcorner",
            Self::TitleBorder => "titleborderdiagonalcorner",
            Self::Note => "notediagonalcorner",
            Self::Activity => "activitydiagonalcorner",
            Self::Class => "classdiagonalcorner",
            Self::State => "statediagonalcorner",
        }
    }
}

/// Dot splines placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DotSplines {
    Splines,
    Polyline,
    Ortho,
}

/// Package style placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageStyle {
    Folder,
    Rectangle,
    Frame,
    Node,
    Card,
}

impl PackageStyle {
    #[must_use]
    pub fn from_string(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "folder" => Some(Self::Folder),
            "rectangle" => Some(Self::Rectangle),
            "frame" => Some(Self::Frame),
            "node" => Some(Self::Node),
            "card" => Some(Self::Card),
            _ => None,
        }
    }
}

/// Component style placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentStyle {
    Uml1,
    Uml2,
    Rectangle,
}

/// Condition style placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionStyle {
    InsideHexagon,
    OutsideHexagon,
    Diamond,
}

impl ConditionStyle {
    #[must_use]
    pub fn from_string(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "insidehexagon" => Some(Self::InsideHexagon),
            "outsidehexagon" => Some(Self::OutsideHexagon),
            "diamond" => Some(Self::Diamond),
            _ => None,
        }
    }
}

/// Condition end style placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionEndStyle {
    Diamond,
    Line,
    LineEndsArrow,
}

impl ConditionEndStyle {
    #[must_use]
    pub fn from_string(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "diamond" => Some(Self::Diamond),
            "line" => Some(Self::Line),
            "lineendsarrow" => Some(Self::LineEndsArrow),
            _ => None,
        }
    }
}

/// Rank direction placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rankdir {
    TopToBottom,
    LeftToRight,
    BottomToTop,
    RightToLeft,
}

impl Default for Rankdir {
    fn default() -> Self {
        Self::TopToBottom
    }
}

/// Guillemet character style placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Guillemet {
    None,
    DoubleComparator,
    Guillemet,
}

impl Guillemet {
    #[must_use]
    pub fn from_description(value: Option<&str>) -> Self {
        match value {
            Some(v) if v.eq_ignore_ascii_case("guillemet") => Self::Guillemet,
            _ => Self::None,
        }
    }
}

/// Split parameter placeholder.
#[derive(Debug, Clone, Default)]
pub struct SplitParam {
    pub border_color: Option<HColor>,
    pub external_color: Option<HColor>,
    pub margin: i32,
}

/// Padder placeholder.
#[derive(Debug, Clone, Default)]
pub struct Padder {
    pub padding: ClockwiseTopRightBottomLeft,
    pub margin: ClockwiseTopRightBottomLeft,
    pub border_color: Option<HColor>,
    pub background_color: Option<HColor>,
    pub round_corner: f64,
}

impl Padder {
    pub const NONE: Self = Self {
        padding: ClockwiseTopRightBottomLeft::none(),
        margin: ClockwiseTopRightBottomLeft::none(),
        border_color: None,
        background_color: None,
        round_corner: 0.0,
    };
}

/// Actor style placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorStyle {
    Stickman,
    Awesome,
    Hollow,
}

/// UStroke placeholder.
#[derive(Debug, Clone, PartialEq)]
pub struct UStroke {
    pub thickness: f64,
    pub dash_visible: f64,
    pub dash_space: f64,
}

impl UStroke {
    #[must_use]
    pub fn with_thickness(thickness: f64) -> Self {
        Self {
            thickness,
            dash_visible: 0.0,
            dash_space: 0.0,
        }
    }

    #[must_use]
    pub fn simple() -> Self {
        Self::with_thickness(1.0)
    }

    #[must_use]
    pub fn get_thickness(&self) -> f64 {
        self.thickness
    }

    #[must_use]
    pub fn get_dash_space(&self) -> f64 {
        self.dash_space
    }

    #[must_use]
    pub fn get_dash_visible(&self) -> f64 {
        self.dash_visible
    }
}

impl Default for UStroke {
    fn default() -> Self {
        Self::simple()
    }
}

/// Line break strategy placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LineBreakStrategy {
    pub value: Option<String>,
}

impl LineBreakStrategy {
    #[must_use]
    pub fn new(value: Option<String>) -> Self {
        Self { value }
    }
}

/// Padding parameter placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaddingParam {
    Default,
    Activity,
    Class,
    Note,
    Package,
    Participant,
    Sequence,
}

impl PaddingParam {
    #[must_use]
    pub fn get_skin_name(self) -> &'static str {
        match self {
            Self::Default => "padding",
            Self::Activity => "activityPadding",
            Self::Class => "classPadding",
            Self::Note => "notePadding",
            Self::Package => "packagePadding",
            Self::Participant => "participantPadding",
            Self::Sequence => "sequencePadding",
        }
    }
}

/// Arrows placeholder.
#[derive(Debug, Clone, Default)]
pub struct Arrows;

/// TikzFontDistortion placeholder.
#[derive(Debug, Clone, Default)]
pub struct TikzFontDistortion;

impl TikzFontDistortion {
    #[must_use]
    pub fn from_value(_value: Option<&str>) -> Self {
        Self
    }
}

/// Colors placeholder.
#[derive(Debug, Clone, Default)]
pub struct Colors {
    pub back: Option<HColor>,
    pub line: Option<HColor>,
    pub text: Option<HColor>,
}

impl Colors {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

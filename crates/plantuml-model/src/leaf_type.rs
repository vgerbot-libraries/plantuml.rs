//! Leaf type enum — entity type variants.
//!
//! Ported from: `net/sourceforge/plantuml/abel/LeafType.java`

/// Entity type variants for leaf entities (non-group entities).
///
/// Ported from: `net/sourceforge/plantuml/abel/LeafType.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeafType {
    EmptyPackage,
    AbstractClass,
    Class,
    Interface,
    Annotation,
    Protocol,
    Struct,
    Exception,
    Metaclass,
    Stereotype,
    LollipopFull,
    LollipopHalf,
    Note,
    Tips,
    Object,
    Map,
    Json,
    Association,
    Enum,
    Circle,
    Dataclass,
    Record,
    Usecase,
    UsecaseBusiness,
    Description,
    ArcCircle,
    Activity,
    Branch,
    SynchroBar,
    CircleStart,
    CircleEnd,
    PointForAssociation,
    ActivityConcurrent,
    State,
    StateConcurrent,
    PseudoState,
    DeepHistory,
    StateChoice,
    StateForkJoin,
    StateTransitionLabel,
    Block,
    Entity,
    Domain,
    Requirement,
    PortIn,
    PortOut,
    ChenEntity,
    ChenRelationship,
    ChenAttribute,
    ChenCircle,
    StillUnknown,
}

impl LeafType {
    /// Returns `true` if this type is class-like (can have methods/fields).
    ///
    /// Ported from: `LeafType.isLikeClass()`.
    #[must_use]
    pub const fn is_like_class(&self) -> bool {
        matches!(
            self,
            Self::AbstractClass
                | Self::Class
                | Self::Interface
                | Self::Annotation
                | Self::Protocol
                | Self::Struct
                | Self::Exception
                | Self::Metaclass
                | Self::Stereotype
                | Self::Enum
                | Self::Dataclass
                | Self::Record
        )
    }

    /// Parses a string to a `LeafType`.
    ///
    /// Ported from: `LeafType.getLeafType(String)`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "abstract class" => Some(Self::AbstractClass),
            "annotation" => Some(Self::Annotation),
            "class" => Some(Self::Class),
            "circle" => Some(Self::Circle),
            "circle_start" => Some(Self::CircleStart),
            "circle_end" => Some(Self::CircleEnd),
            "component" | "description" => Some(Self::Description),
            "usecase" => Some(Self::Usecase),
            "usecasebusiness" => Some(Self::UsecaseBusiness),
            "database" | "entity" => Some(Self::Entity),
            "enum" => Some(Self::Enum),
            "interface" => Some(Self::Interface),
            "metaclass" => Some(Self::Metaclass),
            "protocol" => Some(Self::Protocol),
            "struct" => Some(Self::Struct),
            "exception" => Some(Self::Exception),
            "stereotype" => Some(Self::Stereotype),
            "json" => Some(Self::Json),
            "map" => Some(Self::Map),
            "object" => Some(Self::Object),
            "note" => Some(Self::Note),
            "label" | "arc circle" => Some(Self::ArcCircle),
            "activity" => Some(Self::Activity),
            "branch" => Some(Self::Branch),
            "synchrobar" => Some(Self::SynchroBar),
            "point" | "pointforassociation" => Some(Self::PointForAssociation),
            "state" => Some(Self::State),
            "deephistory" => Some(Self::DeepHistory),
            "choice" | "statechoice" => Some(Self::StateChoice),
            "forkjoin" | "stateforkjoin" => Some(Self::StateForkJoin),
            "block" => Some(Self::Block),
            "domain" => Some(Self::Domain),
            "requirement" => Some(Self::Requirement),
            "portin" => Some(Self::PortIn),
            "portout" => Some(Self::PortOut),
            "dataclass" => Some(Self::Dataclass),
            "record" => Some(Self::Record),
            "association" => Some(Self::Association),
            "tips" => Some(Self::Tips),
            "lollipop_full" | "lollipopfull" => Some(Self::LollipopFull),
            "lollipop_half" | "lollipophalf" => Some(Self::LollipopHalf),
            "chen entity" | "chenentity" => Some(Self::ChenEntity),
            "chen relationship" | "chenrelationship" => Some(Self::ChenRelationship),
            "chen attribute" | "chenattribute" => Some(Self::ChenAttribute),
            "chen circle" | "chencircle" => Some(Self::ChenCircle),
            _ => None,
        }
    }
}

//! Entity-link source parser for CucaDiagram types.
//!
//! Ported from: `net/sourceforge/plantuml/command/CommandCreateEntity.java`,
//! `CommandLink.java`, and related command classes.
//!
//! Parses source lines like:
//! ```text
//! class Alice
//! class Bob
//! Alice --> Bob : knows
//! Bob --> Alice : knows
//! ```
//!
//! Supports: entity declarations, relationship arrows, labels, stereotypes,
//! and basic visibility modifiers.

use indexmap::IndexMap;

/// A parsed entity declaration.
#[derive(Debug, Clone)]
pub struct ParsedEntity {
    /// Entity name (identifier).
    pub name: String,
    /// Display label (may differ from name with `as` keyword).
    pub display: String,
    /// Entity kind (class, interface, abstract, enum, annotation, object, etc.).
    pub kind: EntityKind,
    /// Optional stereotype (`<<stereotype>>`).
    pub stereotype: Option<String>,
    /// Optional body lines (fields/methods between `{` and `}`).
    pub body: Vec<String>,
}

/// Kind of entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityKind {
    /// Regular class.
    #[default]
    Class,
    /// Interface.
    Interface,
    /// Abstract class.
    Abstract,
    /// Enum.
    Enum,
    /// Annotation (`@interface`).
    Annotation,
    /// Object instance (for object diagrams).
    Object,
    /// State (for state diagrams).
    State,
    /// Component (for description diagrams).
    Component,
    /// Deployment node.
    Node,
    /// Use case.
    Usecase,
    /// Actor.
    Actor,
    /// Database.
    Database,
    /// Rectangle/box.
    Rectangle,
    /// Cloud.
    Cloud,
    /// Package.
    Package,
    /// Folder.
    Folder,
    /// Frame.
    Frame,
}

impl EntityKind {
    /// Parses an entity kind from a keyword.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword.to_lowercase().as_str() {
            "class" => Some(Self::Class),
            "interface" => Some(Self::Interface),
            "abstract" | "abstractclass" => Some(Self::Abstract),
            "enum" => Some(Self::Enum),
            "annotation" => Some(Self::Annotation),
            "object" => Some(Self::Object),
            "state" => Some(Self::State),
            "component" => Some(Self::Component),
            "node" => Some(Self::Node),
            "usecase" => Some(Self::Usecase),
            "actor" => Some(Self::Actor),
            "database" => Some(Self::Database),
            "rectangle" | "box" => Some(Self::Rectangle),
            "cloud" => Some(Self::Cloud),
            "package" => Some(Self::Package),
            "folder" => Some(Self::Folder),
            "frame" => Some(Self::Frame),
            _ => None,
        }
    }

    /// Returns the default box shape for this kind.
    pub fn is_box(&self) -> bool {
        !matches!(self, Self::Actor | Self::Usecase)
    }
}

/// A parsed link (relationship) between two entities.
#[derive(Debug, Clone)]
pub struct ParsedLink {
    /// Source entity name.
    pub from: String,
    /// Target entity name.
    pub to: String,
    /// Arrow type (e.g. `-->`, `->`, `..>`, `*->`, `o->`).
    pub arrow: String,
    /// Optional label.
    pub label: Option<String>,
    /// Optional direction (left, right, both).
    pub direction: LinkDirection,
}

/// Link direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkDirection {
    /// No arrowhead (association).
    #[default]
    None,
    /// Right-pointing arrow.
    Right,
    /// Left-pointing arrow.
    Left,
    /// Bidirectional.
    Both,
}

/// Result of parsing a CucaDiagram source.
#[derive(Debug, Clone, Default)]
pub struct ParsedSource {
    /// Parsed entities, keyed by name.
    pub entities: IndexMap<String, ParsedEntity>,
    /// Parsed links.
    pub links: Vec<ParsedLink>,
    /// Notes (entity name → note text).
    pub notes: Vec<ParsedNote>,
    /// Package/group declarations.
    pub packages: Vec<ParsedPackage>,
}

/// A parsed note.
#[derive(Debug, Clone)]
pub struct ParsedNote {
    /// Note text.
    pub text: String,
    /// Target entity name (if attached to an entity).
    pub target: Option<String>,
    /// Position (left, right, top, bottom).
    pub position: NotePosition,
}

/// Note position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotePosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// A parsed package/group.
#[derive(Debug, Clone)]
pub struct ParsedPackage {
    /// Package name.
    pub name: String,
    /// Entity names within this package.
    pub entities: Vec<String>,
}

/// Parses CucaDiagram source lines into entities, links, and notes.
///
/// This is a simplified parser that handles the most common PlantUML syntax:
/// - Entity declarations: `class Alice`, `interface Bob`, `state Idle`
/// - Relationships: `A --> B`, `A -> B : label`
/// - Notes: `note left of Alice : text`
/// - Packages: `package "name" { ... }`
///
/// Ported from: `CommandCreateEntity`, `CommandLink`, `CommandNote`, `CommandPackage`.
#[must_use]
pub fn parse_entity_link_source(lines: &[&str]) -> ParsedSource {
    let mut result = ParsedSource::default();
    let mut current_package: Option<ParsedPackage> = None;
    let mut current_entity_body: Option<String> = None;

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines, comments, and directives.
        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("note ") && trimmed.contains(" of ") {
            // Handle notes separately below
        }
        if trimmed.starts_with('\'') || trimmed.starts_with("//") {
            continue;
        }

        // Handle multi-line entity body (between { and }).
        if let Some(ref mut entity_name) = current_entity_body {
            if trimmed.starts_with('}') {
                // End of body — save body lines to the entity.
                if let Some(_entity) = result.entities.get_mut(entity_name) {
                    // Body lines were already collected inline.
                }
                current_entity_body = None;
                continue;
            }
            // Add line to current entity's body.
            if let Some(entity) = result.entities.get_mut(entity_name) {
                entity.body.push(trimmed.to_string());
            }
            continue;
        }
        // Check for package/namespace declarations (before entity check,
        // since "package" is also an EntityKind).
        if trimmed.starts_with("package ") || trimmed.starts_with("namespace ") {
            let rest = trimmed.split_once(' ').map_or("", |(_, r)| r).trim();
            let name = rest
                .trim_matches('"')
                .trim_matches('\'')
                .trim_end_matches('{')
                .trim()
                .to_string();
            current_package = Some(ParsedPackage {
                name,
                entities: Vec::new(),
            });
            continue;
        }
        if trimmed == "}" && current_package.is_some() {
            if let Some(pkg) = current_package.take() {
                result.packages.push(pkg);
            }
            continue;
        }

        // Check for entity declarations.
        if let Some(entity) = parse_entity_declaration(trimmed) {
            let name = entity.name.clone();
            result.entities.insert(name.clone(), entity);
            if trimmed.contains('{') {
                current_entity_body = Some(name.clone());
            }
            if let Some(ref mut pkg) = current_package {
                pkg.entities.push(name);
            }
            continue;
        }

        // Check for relationship arrows.
        if let Some(link) = parse_link_line(trimmed) {
            result.links.push(link);
            continue;
        }

        // Check for notes.
        if let Some(note) = parse_note_line(trimmed) {
            result.notes.push(note);

        }
    }

    result
}

/// Parses an entity declaration line (e.g. `class Alice`, `interface Bob`).
fn parse_entity_declaration(line: &str) -> Option<ParsedEntity> {
    // Skip lines that look like relationships (contain arrows).
    if line.contains("-->") || line.contains("->") || line.contains("..>") || line.contains("..") {
        return None;
    }
    if line.contains("--") || line.contains("-|>") || line.contains("->") {
        return None;
    }

    // Split into keyword and rest.
    let (keyword, rest) = line.split_once(' ')?;

    let kind = EntityKind::from_keyword(keyword)?;
    let rest = rest.trim();

    // Parse name, optional `as` alias, optional stereotype.
    let (name_part, stereotype) = if let Some(start) = rest.find("<<") {
        if let Some(end) = rest.rfind(">>") {
            if end > start {
                let st = &rest[start + 2..end];
                (rest[..start].trim(), Some(st.to_string()))
            } else {
                (rest, None)
            }
        } else {
            (rest, None)
        }
    } else {
        (rest, None)
    };

    // Handle `as` alias: `Alice as "Display Name"`
    let (name, display) = if let Some((n, d)) = name_part.split_once(" as ") {
        (n.trim().to_string(), d.trim().trim_matches('"').to_string())
    } else {
        let n = name_part.trim().trim_end_matches('{').trim().to_string();
        (n.clone(), n)
    };

    if name.is_empty() {
        return None;
    }

    Some(ParsedEntity {
        name,
        display,
        kind,
        stereotype,
        body: Vec::new(),
    })
}

/// Arrow pattern characters for detection.
const ARROW_CHARS: &str = "-<>:=.*|ox+#0";

/// Arrow body characters (must start with one of these).
const ARROW_BODY: &str = "-.=";

/// Parses a relationship line (e.g. `A --> B : label`).
fn parse_link_line(line: &str) -> Option<ParsedLink> {
    // Find the arrow part (sequence of arrow characters between two identifiers).
    // Look for patterns like `A --> B`, `A -> B : label`, `A ..> B`

    // Split on colon for label.
    let (arrow_end_idx, label) = if let Some(idx) = line.find(" : ") {
        (idx, Some(line[idx + 3..].trim().to_string()))
    } else if let Some(idx) = line.find(": ") {
        let before = &line[..idx];
        if contains_arrow(before) {
            (idx, Some(line[idx + 2..].trim().to_string()))
        } else {
            return None;
        }
    } else {
        (line.len(), None)
    };
    let arrow_part = &line[..arrow_end_idx];

    // Find the arrow body (must start with -, ., or = — the arrow body chars).
    // Decorations like o, x, <, >, # can appear at the ends but not standalone.
    let body_start = arrow_part
        .char_indices()
        .find(|(_, c)| ARROW_CHARS.contains(*c) && ARROW_BODY.contains(*c))?
        .0;
    // Walk backwards from body_start to include leading decorations (o, x, <, etc).
    let mut arrow_start = body_start;
    for (i, c) in arrow_part[..body_start].char_indices().rev() {
        if ARROW_CHARS.contains(c) && !ARROW_BODY.contains(c) {
            arrow_start = i;
        } else {
            break;
        }
    }
    // Walk forward to include the full arrow (body + trailing decorations + whitespace).
    let mut arrow_end = arrow_start;
    let mut found_body = false;
    for (i, c) in arrow_part[arrow_start..].char_indices() {
        let abs_i = arrow_start + i;
        if c.is_whitespace() && !found_body {
            continue;
        }
        if ARROW_CHARS.contains(c) {
            if ARROW_BODY.contains(c) {
                found_body = true;
            }
            arrow_end = abs_i + c.len_utf8();
        } else if c.is_whitespace() && found_body {
            arrow_end = abs_i;
            break;
        } else {
            break;
        }
    }

    // Trim whitespace from arrow.
    let arrow = arrow_part[arrow_start..arrow_end].trim().to_string();
    if arrow.is_empty() || arrow.len() < 2 {
        return None;
    }

    let from = arrow_part[..arrow_start].trim().to_string();
    let to = arrow_part[arrow_end..].trim().to_string();

    if from.is_empty() || to.is_empty() {
        return None;
    }

    // Determine direction from arrow.
    let direction = if arrow.contains('>') && arrow.contains('<') {
        LinkDirection::Both
    } else if arrow.contains('>') {
        LinkDirection::Right
    } else if arrow.contains('<') {
        LinkDirection::Left
    } else {
        LinkDirection::None
    };

    Some(ParsedLink {
        from,
        to,
        arrow,
        label,
        direction,
    })
}

/// Checks if a string contains arrow characters.
fn contains_arrow(s: &str) -> bool {
    s.chars().any(|c| ARROW_CHARS.contains(c))
}

/// Parses a note line (e.g. `note left of Alice : text`).
fn parse_note_line(line: &str) -> Option<ParsedNote> {
    if !line.starts_with("note ") {
        return None;
    }

    let rest = &line[5..]; // after "note "

    // Parse position: left, right, top, bottom.
    let (position, rest) = if let Some(r) = rest.strip_prefix("left ") {
        (NotePosition::Left, r)
    } else if let Some(r) = rest.strip_prefix("right ") {
        (NotePosition::Right, r)
    } else if let Some(r) = rest.strip_prefix("top ") {
        (NotePosition::Top, r)
    } else if let Some(r) = rest.strip_prefix("bottom ") {
        (NotePosition::Bottom, r)
    } else {
        (NotePosition::Top, rest)
    };

    // Parse "of EntityName" or "of of EntityName".
    let (target, text) = if let Some(r) = rest.strip_prefix("of ") {
        if let Some(idx) = r.find(" : ") {
            (Some(r[..idx].trim().to_string()), r[idx + 3..].trim().to_string())
        } else {
            (Some(r.trim().to_string()), String::new())
        }
    } else if let Some(idx) = rest.find(" : ") {
        (None, rest[idx + 3..].trim().to_string())
    } else {
        (None, rest.trim().to_string())
    };

    if text.is_empty() && target.is_none() {
        return None;
    }

    Some(ParsedNote {
        text,
        target,
        position,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class_declaration() {
        let entity = parse_entity_declaration("class Alice").unwrap();
        assert_eq!(entity.name, "Alice");
        assert_eq!(entity.kind, EntityKind::Class);
    }

    #[test]
    fn test_parse_interface_with_stereotype() {
        let entity = parse_entity_declaration("interface Bob <<service>>").unwrap();
        assert_eq!(entity.name, "Bob");
        assert_eq!(entity.kind, EntityKind::Interface);
        assert_eq!(entity.stereotype, Some("service".to_string()));
    }

    #[test]
    fn test_parse_link() {
        let link = parse_link_line("Alice --> Bob : knows").unwrap();
        assert_eq!(link.from, "Alice");
        assert_eq!(link.to, "Bob");
        assert_eq!(link.label, Some("knows".to_string()));
        assert_eq!(link.direction, LinkDirection::Right);
    }

    #[test]
    fn test_parse_dashed_link() {
        let link = parse_link_line("Alice ..> Bob").unwrap();
        assert_eq!(link.from, "Alice");
        assert_eq!(link.to, "Bob");
        assert_eq!(link.direction, LinkDirection::Right);
    }

    #[test]
    fn test_parse_bidirectional_link() {
        let link = parse_link_line("Alice <-> Bob").unwrap();
        assert_eq!(link.direction, LinkDirection::Both);
    }

    #[test]
    fn test_parse_full_source() {
        let lines = vec![
            "class Alice",
            "class Bob",
            "Alice --> Bob : knows",
            "Bob --> Alice : knows too",
        ];
        let parsed = parse_entity_link_source(&lines);
        assert_eq!(parsed.entities.len(), 2);
        assert_eq!(parsed.links.len(), 2);
        assert!(parsed.entities.contains_key("Alice"));
        assert!(parsed.entities.contains_key("Bob"));
    }

    #[test]
    fn test_parse_note() {
        let lines = vec!["class Alice", "note left of Alice : important"];
        let parsed = parse_entity_link_source(&lines);
        assert_eq!(parsed.notes.len(), 1);
        assert_eq!(parsed.notes[0].target, Some("Alice".to_string()));
        assert_eq!(parsed.notes[0].position, NotePosition::Left);
        assert_eq!(parsed.notes[0].text, "important");
    }

    #[test]
    fn test_parse_package() {
        let lines = vec!["package Models {", "class Alice", "class Bob", "}"];
        let parsed = parse_entity_link_source(&lines);
        assert_eq!(parsed.packages.len(), 1);
        assert_eq!(parsed.packages[0].name, "Models");
        assert_eq!(parsed.packages[0].entities.len(), 2);
    }
}

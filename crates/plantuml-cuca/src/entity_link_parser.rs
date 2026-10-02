//! Entity-link source parser for CucaDiagram types.
//!
//! Ported from: `net/sourceforge/plantuml/command/CommandCreateEntity.java`,
//! `net/sourceforge/plantuml/descdiagram/command/CommandLinkElement.java`,
//! and related command classes.
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
//! nested groups (package/node/state), and implicit entity creation from
//! bracketed link endpoints (`[X]` component, `(X)` use case).

use indexmap::IndexMap;

/// A parsed entity declaration.
#[derive(Debug, Clone)]
pub struct ParsedEntity {
    /// Entity name / qualified key (e.g. `Application Server.Web App`).
    pub name: String,
    /// Display label (the simple, unqualified name).
    pub display: String,
    /// Entity kind (class, interface, state, component, start pseudo, etc.).
    pub kind: EntityKind,
    /// Optional stereotype (`<<stereotype>>`).
    pub stereotype: Option<String>,
    /// Attribute/method/field body lines (class/object between `{` and `}`).
    pub body: Vec<String>,
    /// Source line number (1-based, matching Java's LineLocation).
    pub source_line: usize,
    /// Key of the containing group or composite state, when nested.
    pub parent: Option<String>,
    /// True when the entity is a group (composite state, node folder, package).
    pub group: bool,
    /// Child entity keys of a group, in source order.
    pub members: Vec<String>,
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
    /// Deployment node (3D box), and an unfolded node folder group.
    Node,
    /// Use case.
    Usecase,
    /// Actor.
    Actor,
    /// Database.
    Database,
    /// Initial pseudo-state (`[*]` as link source), filled circle.
    Start,
    /// Final pseudo-state (`[*]` as link target), ring with inner circle.
    End,
    /// Rectangle/box.
    Rectangle,
    /// Cloud.
    Cloud,
    /// Package (folder tab). Also a package group.
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
}

/// A parsed link (relationship) between two endpoints.
///
/// Endpoint strings keep the form as written in the source (`[Client]`,
/// `[*]`); they are resolved to qualified entity keys at layout time via
/// [`resolve_endpoint`].
#[derive(Debug, Clone)]
pub struct ParsedLink {
    /// Source endpoint as written.
    pub from: String,
    /// Target endpoint as written.
    pub to: String,
    /// Arrow type (e.g. `-->`, `->`, `..>`, `--`, `*->`, `o->`).
    pub arrow: String,
    /// Optional label.
    pub label: Option<String>,
    /// Source line number (1-based, matching Java's LineLocation).
    pub source_line: usize,
}

/// Result of parsing a CucaDiagram source.
#[derive(Debug, Clone, Default)]
pub struct ParsedSource {
    /// Parsed entities, keyed by qualified name.
    pub entities: IndexMap<String, ParsedEntity>,
    /// Parsed links (endpoints as written).
    pub links: Vec<ParsedLink>,
    /// Parsed notes.
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

/// Resolves a link endpoint (as written) to a qualified entity key.
///
/// Ported from: `DescriptionDiagram.cleanId` and `quarkInContext` lookup.
/// The state pseudo node `[*]` resolves to `.start.` for a source endpoint
/// and `.end.` for a target endpoint.
#[must_use]
pub fn resolve_endpoint(endpoint: &str, is_source: bool, entities: &IndexMap<String, ParsedEntity>) -> String {
    let e = endpoint.trim();
    if e == "[*]" {
        return if is_source { ".start.".to_string() } else { ".end.".to_string() };
    }
    let base = endpoint_base(e);
    if entities.contains_key(base) {
        return base.to_string();
    }
    // Nested: match a qualified key ending in ".base".
    let dotted = format!(".{base}");
    for key in entities.keys() {
        if key.ends_with(&dotted) {
            return key.clone();
        }
    }
    base.to_string()
}

/// Strips quotes and a single pair of shape delimiters from an endpoint,
/// returning the inner identifier.
///
/// `(Login)` → `Login` (use case), `[Web Server]` → `Web Server`
/// (component), `:Actor:` → `Actor`, `"X"` → `X`.
fn endpoint_base(e: &str) -> &str {
    let e = e.trim();
    let e = e.trim_matches('"').trim_matches('\'');
    let bytes = e.as_bytes();
    if bytes.len() >= 2 {
        match (bytes[0], bytes[bytes.len() - 1]) {
            (b'(', b')') | (b'[', b']') => return &e[1..e.len() - 1],
            _ => {}
        }
    }
    if bytes.len() >= 2 && bytes[0] == b':' && bytes[bytes.len() - 1] == b':' {
        return &e[1..e.len() - 1];
    }
    e
}

/// Parses CucaDiagram source lines into entities, groups, links, and notes.
///
/// Ported from: `CommandCreateEntity`, `CommandLinkElement`, `CommandNote`,
/// and the package/group command factories.
#[must_use]
pub fn parse_entity_link_source(lines: &[&str]) -> ParsedSource {
    let mut result = ParsedSource::default();
    // Stack of open group keys (package, node folder, composite state).
    let mut group_stack: Vec<String> = Vec::new();
    let mut body_entity: Option<String> = None;

    let mut source_line = 0;
    for line in lines {
        let trimmed = line.trim();

        // @start/@end directives are not counted (Java: first body line = 1).
        if trimmed.starts_with("@start") || trimmed.starts_with("@end") {
            continue;
        }
        source_line += 1;

        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
            continue;
        }

        // Multi-line entity body (between { and }).
        if body_entity.is_some() {
            if trimmed.starts_with('}') {
                body_entity = None;
                continue;
            }
            if let Some(name) = &body_entity {
                if let Some(entity) = result.entities.get_mut(name) {
                    entity.body.push(trimmed.to_string());
                }
            }
            continue;
        }

        // Closing brace for a group.
        if trimmed.starts_with('}') {
            if let Some(g) = group_stack.pop() {
                result
                    .entities
                    .get_mut(&g)
                    .map(|_| ());
            }
            continue;
        }

        // Group / entity declaration.
        if let Some(decl) = parse_entity_declaration(trimmed) {
            let (keyword, rest_full) = match trimmed.split_once(' ') {
                Some(kv) => kv,
                None => continue,
            };
            let opens_group = trimmed.ends_with('{') || trimmed.contains(" {");
            let parent = group_stack.last().cloned();

            // Qualify the name with the enclosing group.
            let simple_name = decl.name.clone();
            let qualified = match &parent {
                Some(p) => format!("{p}.{simple_name}"),
                None => simple_name.clone(),
            };

            // Decide whether this declaration creates a group.
            let is_group = opens_group
                && matches!(
                    keyword.to_lowercase().as_str(),
                    "package" | "node" | "state" | "folder" | "namespace"
                );

            let display = decl.display.clone();
            let mut entity = decl;
            entity.name = qualified.clone();
            entity.display = display;
            entity.parent = parent.clone();
            entity.group = is_group;
            entity.source_line = source_line;

            // Inline body (class/object "{ ... }") without a group.
            let has_inline_body = trimmed.contains('{') && !is_group;

            result.entities.insert(qualified.clone(), entity);
            if let Some(p) = parent {
                if let Some(pe) = result.entities.get_mut(&p) {
                    pe.members.push(qualified.clone());
                }
            }
            if is_group {
                group_stack.push(qualified.clone());
            } else if has_inline_body {
                body_entity = Some(qualified.clone());
            }
            let _ = rest_full;
            continue;
        }

        // Relationship line — create implicit entities first, then the link.
        if let Some(mut link) = parse_link_line(trimmed) {
            create_implicit(&link.from, true, source_line, &group_stack, &mut result);
            create_implicit(&link.to, false, source_line, &group_stack, &mut result);
            link.from = normalize_endpoint(&link.from);
            link.to = normalize_endpoint(&link.to);
            result.links.push(ParsedLink {
                source_line,
                ..link
            });
            continue;
        }

        // Notes.
        if let Some(note) = parse_note_line(trimmed) {
            result.notes.push(note);
        }
    }

    result
}

/// Strips the anonymous-container wrapper from a link endpoint, mapping it
/// to the declared entity name (`[Client]` → `Client`, `(Login)` → `Login`).
/// The pseudo node `[*]` is preserved.
fn normalize_endpoint(endpoint: &str) -> String {
    let e = endpoint.trim();
    if e == "[*]" {
        return e.to_string();
    }
    let b = e.as_bytes();
    if b.len() >= 3
        && ((b[0] == b'[' && b[b.len() - 1] == b']')
            || (b[0] == b'(' && b[b.len() - 1] == b')'))
    {
        return e[1..e.len() - 1].trim().to_string();
    }
    e.to_string()
}

/// Creates an implicit entity for a bracketed link endpoint.
///
/// Ported from: `CommandLinkElement.getDummy`. `[X]` creates a component,
/// `(X)` a use case; the pseudo node `[*]` and already-known endpoints are
/// skipped. Nested implicit entities are qualified and attached to the open
/// group.
fn create_implicit(
    endpoint: &str,
    is_source: bool,
    source_line: usize,
    group_stack: &[String],
    result: &mut ParsedSource,
) {
    let e = endpoint.trim();
    if e == "[*]" {
        // Ensure .start. / .end. pseudo entities exist.
        let key = if is_source { ".start." } else { ".end." };
        let kind = if is_source { EntityKind::Start } else { EntityKind::End };
        if !result.entities.contains_key(key) {
            result.entities.insert(
                key.to_string(),
                ParsedEntity {
                    name: key.to_string(),
                    display: key.to_string(),
                    kind,
                    source_line,
                    ..empty_entity()
                },
            );
        }
        return;
    }

    let bytes = e.as_bytes();
    if bytes.len() < 3 {
        return;
    }
    let kind = match (bytes[0], bytes[bytes.len() - 1]) {
        (b'[', b']') => Some(EntityKind::Component),
        (b'(', b')') => Some(EntityKind::Usecase),
        _ => None,
    };
    let Some(kind) = kind else { return };

    let simple = e[1..e.len() - 1].trim().to_string();
    if simple.is_empty() {
        return;
    }
    let parent = group_stack.last().cloned();
    let qualified = match &parent {
        Some(p) => format!("{p}.{simple}"),
        None => simple.clone(),
    };
    // Skip if the entity already exists (declared or a prior endpoint).
    if result.entities.contains_key(&qualified)
        || result.entities.keys().any(|k| k.ends_with(&format!(".{simple}")))
    {
        return;
    }
    result.entities.insert(
        qualified.clone(),
        ParsedEntity {
            name: qualified.clone(),
            display: simple.clone(),
            kind,
            source_line,
            parent: parent.clone(),
            ..empty_entity()
        },
    );
    if let Some(p) = parent {
        if let Some(pe) = result.entities.get_mut(&p) {
            pe.members.push(qualified);
        }
    }
}

fn empty_entity() -> ParsedEntity {
    ParsedEntity {
        name: String::new(),
        display: String::new(),
        kind: EntityKind::Class,
        stereotype: None,
        body: Vec::new(),
        source_line: 0,
        parent: None,
        group: false,
        members: Vec::new(),
    }
}

/// Parses an entity declaration line (e.g. `class Alice`, `interface Bob`).
///
/// Returns the entity with the simple (unqualified) name; the caller qualifies
/// it with the enclosing group.
fn parse_entity_declaration(line: &str) -> Option<ParsedEntity> {
    // Relationship lines are not declarations.
    if line.contains("->") || line.contains("..>") {
        return None;
    }

    let (keyword, rest) = line.split_once(' ')?;
    let kind = EntityKind::from_keyword(keyword)?;
    let rest = rest.trim();

    // Split off stereotype.
    let (name_part, stereotype) = if let Some(start) = rest.find("<<") {
        if let Some(end) = rest.rfind(">>") {
            if end > start {
                let st = rest[start + 2..end].trim();
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

    // Handle `as` alias: `"User API" as API` → name API, display User API.
    // The left side is the (possibly quoted) display label, the right side
    // the internal entity name.
    let (name, display) = if let Some((label, alias)) = name_part.split_once(" as ") {
        let label = label.trim().trim_end_matches('{').trim();
        let alias = alias.trim().trim_matches('"').trim_matches('\'');
        let name = endpoint_base(alias).to_string();
        let unquoted = label.trim_matches('"').trim_matches('\'');
        let display = if unquoted.is_empty() {
            name.clone()
        } else {
            unquoted.to_string()
        };
        (name, display)
    } else {
        let n = name_part.trim_end_matches('{').trim();
        let base = endpoint_base(n).to_string();
        (base.clone(), base)
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
        source_line: 0,
        parent: None,
        group: false,
        members: Vec::new(),
    })
}

// ── Link parsing ─────────────────────────────────────────────────────────

/// Arrow body characters (must include one of these).
const ARROW_BODY: &str = "-.=";
/// Characters allowed within an arrow.
const ARROW_CHARS: &str = "-<>:=.*|ox+#0";

fn contains_arrow(s: &str) -> bool {
    s.chars().any(|c| ARROW_CHARS.contains(c))
}

/// Parses a relationship line (e.g. `A --> B : label`).
///
/// Endpoints keep their written form; the caller resolves them and creates
/// implicit entities.
fn parse_link_line(line: &str) -> Option<ParsedLink> {
    // Split label on " : " or ": ".
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

    // Locate the arrow body start.
    let body_start = arrow_part
        .char_indices()
        .find(|(_, c)| ARROW_CHARS.contains(*c) && ARROW_BODY.contains(*c))?
        .0;
    let mut arrow_start = body_start;
    for (i, c) in arrow_part[..body_start].char_indices().rev() {
        if ARROW_CHARS.contains(c) && !ARROW_BODY.contains(c) {
            arrow_start = i;
        } else {
            break;
        }
    }
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

    let arrow = arrow_part[arrow_start..arrow_end].trim().to_string();
    if arrow.len() < 2 {
        return None;
    }
    let from = arrow_part[..arrow_start].trim().to_string();
    let to = arrow_part[arrow_end..].trim().to_string();
    if from.is_empty() || to.is_empty() {
        return None;
    }

    Some(ParsedLink {
        from,
        to,
        arrow,
        label,
        source_line: 0,
    })
}

/// Parses a note line (e.g. `note left of Alice : text`).
fn parse_note_line(line: &str) -> Option<ParsedNote> {
    if !line.starts_with("note ") {
        return None;
    }
    let rest = &line[5..];

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

    Some(ParsedNote { text, target, position })
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
    }

    #[test]
    fn test_implicit_component() {
        let lines = ["[Client] ..> API : uses"];
        let parsed = parse_entity_link_source(&lines);
        assert!(parsed.entities.contains_key("Client"));
        assert_eq!(parsed.entities["Client"].kind, EntityKind::Component);
        assert_eq!(parsed.links[0].from, "[Client]");
    }

    #[test]
    fn test_star_creates_pseudo_entities() {
        let lines = ["[*] --> Idle", "Idle --> [*]"];
        let parsed = parse_entity_link_source(&lines);
        assert!(parsed.entities.contains_key(".start."));
        assert!(parsed.entities.contains_key(".end."));
    }

    #[test]
    fn test_nested_group_qualifies() {
        let lines = [
            "node \"Application Server\" {",
            "component [Web App]",
            "database \"Cache\"",
            "}",
        ];
        let parsed = parse_entity_link_source(&lines);
        assert!(parsed.entities.contains_key("Application Server"));
        assert!(parsed.entities.contains_key("Application Server.Web App"));
        assert!(parsed.entities.contains_key("Application Server.Cache"));
        assert_eq!(
            parsed.entities["Application Server"].members,
            vec![
                "Application Server.Web App".to_string(),
                "Application Server.Cache".to_string()
            ]
        );
    }

    #[test]
    fn test_source_line_skips_start_end_directives() {
        let lines = ["@startuml", "class Alice", "interface Bob", "@enduml"];
        let parsed = parse_entity_link_source(&lines);
        assert_eq!(parsed.entities["Alice"].source_line, 1);
        assert_eq!(parsed.entities["Bob"].source_line, 2);
    }
}

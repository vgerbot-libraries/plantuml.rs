//! JSON and YAML diagram factories.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/jsondiagram/JsonDiagramFactory.java`
//! - `net/sourceforge/plantuml/yaml/YamlDiagramFactory.java`
//!
//! These factories live in `plantuml-engine` (not `plantuml-json`) to avoid
//! circular dependencies: `plantuml-engine` depends on `plantuml-json` for
//! the diagram model and renderer, while `plantuml-json` only depends on
//! `plantuml-core` and `plantuml-svg`.

use plantuml_core::{Diagram, DiagramType, PSystemError};
use plantuml_json::{parse_json, parse_yaml_to_json, Highlight, JsonDiagram};
use serde_json::Value;

use crate::p_system_factory::PSystemFactory;
use crate::uml_source::UmlSource;

// ── JSON ─────────────────────────────────────────────────────────────────

/// Factory for JSON diagrams.
///
/// Ported from: `net/sourceforge/plantuml/jsondiagram/JsonDiagramFactory.java`.
pub struct JsonDiagramFactory;

impl PSystemFactory for JsonDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Json
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let (json_str, highlights) = extract_body(source, "@endjson");

        let root = if json_str.trim().is_empty() {
            Some(Value::Array(vec![Value::String(String::new())]))
        } else {
            parse_json(&json_str)
                .map(normalize_root)
        };

        Ok(Box::new(JsonDiagram::new(root, highlights, DiagramType::Json)))
    }
}

// ── YAML ─────────────────────────────────────────────────────────────────

/// Factory for YAML diagrams.
///
/// Ported from: `net/sourceforge/plantuml/yaml/YamlDiagramFactory.java`.
pub struct YamlDiagramFactory;

impl PSystemFactory for YamlDiagramFactory {
    fn get_diagram_type(&self) -> DiagramType {
        DiagramType::Yaml
    }

    fn create_system(&self, source: &UmlSource) -> Result<Box<dyn Diagram>, PSystemError> {
        let (yaml_str, highlights) = extract_body(source, "@endyaml");

        let root = if yaml_str.trim().is_empty() {
            Some(Value::Array(vec![Value::String(String::new())]))
        } else {
            parse_yaml_to_json(&yaml_str).map(normalize_root)
        };

        Ok(Box::new(JsonDiagram::new(root, highlights, DiagramType::Yaml)))
    }
}

// ── Shared helpers ───────────────────────────────────────────────────────

/// Normalizes the root value to match Java's `JsonDiagram` constructor logic.
///
/// Ported from: `JsonDiagram` constructor lines 80-89.
fn normalize_root(value: Value) -> Value {
    match &value {
        Value::String(_) | Value::Bool(_) | Value::Number(_) | Value::Null => {
            Value::Array(vec![value])
        }
        Value::Array(arr) if arr.is_empty() => {
            Value::Array(vec![Value::String(String::new())])
        }
        Value::Object(obj) if obj.is_empty() => {
            Value::Array(vec![Value::String(String::new())])
        }
        _ => value,
    }
}

/// Extracts the body text and highlight directives from the source.
///
/// Skips the `@start<type>` line (already skipped by `body_iter`) and the
/// `@end<type>` line. Extracts `#highlight` directives.
///
/// Ported from: `JsonDiagramFactory.createSystem()` lines 73-92.
fn extract_body(source: &UmlSource, end_keyword: &str) -> (String, Vec<Highlight>) {
    let mut body = String::new();
    let mut highlights = Vec::new();

    for line in source.body_iter() {
        let text = line.get_string();

        // Skip @end line.
        if text.trim().to_lowercase().starts_with(end_keyword) {
            break;
        }

        // Check for highlight directives.
        if text.trim().starts_with('#') {
            if Highlight::matches_definition(text) {
                if let Some(h) = Highlight::build(text) {
                    highlights.push(h);
                }
                continue;
            }
            // Other # directives (style, title, etc.) — skip for now.
            continue;
        }

        body.push_str(text);
        body.push('\n');
    }

    (body, highlights)
}

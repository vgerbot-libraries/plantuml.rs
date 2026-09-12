//! YAML to JSON conversion support.
//!
//! Ported from: `net/sourceforge/plantuml/yaml/` package.
//!
//! Uses `serde_yaml` instead of porting the Java `SimpleYamlParser`.
//! YAML is parsed into `serde_yaml::Value` then converted to `serde_json::Value`
//! to reuse the JSON diagram renderer.

use serde_json::Value;

/// Parses a YAML string and converts it to a `serde_json::Value`.
///
/// Ported from: `net/sourceforge/plantuml/yaml/SimpleYamlParser.java` +
/// `MononomorphToJson.java`.
///
/// Returns `None` if parsing fails.
pub fn parse_yaml_to_json(yaml_str: &str) -> Option<Value> {
    if yaml_str.trim().is_empty() {
        return Some(Value::Array(vec![Value::String(String::new())]));
    }

    // Parse YAML → serde_yaml::Value → serde_json::Value.
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;

    // Convert via JSON string round-trip (serde_yaml → JSON string → serde_json).
    let json_str = serde_json::to_string(&yaml_value).ok()?;
    serde_json::from_str(&json_str).ok()
}

/// Parses a JSON string into a `serde_json::Value`.
///
/// Returns `None` if parsing fails.
pub fn parse_json(json_str: &str) -> Option<Value> {
    serde_json::from_str::<Value>(json_str).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_yaml() {
        let yaml = "name: Alice\nage: 30\n";
        let json = parse_yaml_to_json(yaml).unwrap();
        assert!(json.is_object());
        assert_eq!(json["name"], "Alice");
        assert_eq!(json["age"], 30);
    }

    #[test]
    fn test_yaml_with_nested() {
        let yaml = "person:\n  name: Bob\n  city: NYC\n";
        let json = parse_yaml_to_json(yaml).unwrap();
        assert!(json.is_object());
        assert!(json["person"].is_object());
        assert_eq!(json["person"]["name"], "Bob");
    }

    #[test]
    fn test_yaml_with_array() {
        let yaml = "items:\n  - apple\n  - banana\n";
        let json = parse_yaml_to_json(yaml).unwrap();
        assert!(json["items"].is_array());
        assert_eq!(json["items"][0], "apple");
    }

    #[test]
    fn test_empty_yaml() {
        let json = parse_yaml_to_json("").unwrap();
        assert!(json.is_array());
    }

    #[test]
    fn test_invalid_yaml() {
        assert!(parse_yaml_to_json("{invalid: : : yaml").is_none());
    }
}

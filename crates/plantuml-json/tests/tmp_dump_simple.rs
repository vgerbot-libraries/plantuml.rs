use plantuml_json::json_renderer::render_json_svg;
use serde_json::Value;

#[test]
fn dump_simple() {
    let json: Value =
        serde_json::from_str("{\n  \"key\": \"value\",\n  \"count\": 42\n}").unwrap();
    let svg = render_json_svg(&json, &[], "JSON");
    std::fs::write("/tmp/json_simple.rust.svg", svg).unwrap();
}

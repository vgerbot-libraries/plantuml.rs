//! Tests for the PSystemBuilder pipeline (Phase 0 verification).
//!
//! Verifies that the new PSystemBuilder dispatch produces identical SVG
//! output to the legacy bypass pipeline for sequence diagrams.

use plantuml_core::FileFormat;
use plantuml_engine::render_svg;

/// A simple sequence diagram that the bypass pipeline can parse.
const SIMPLE_SEQUENCE: &str = "@startuml\nAlice -> Bob: hello\n@enduml";

/// Renders via the new `render_svg` (which tries PSystemBuilder pipeline first,
/// falls back to bypass) and verifies it produces valid SVG.
#[test]
fn test_pipeline_renders_sequence_svg() {
    let svg = render_svg(SIMPLE_SEQUENCE).expect("render_svg should succeed");

    // Must be valid SVG.
    assert!(svg.contains("<svg"), "output must contain <svg: {svg}");
    assert!(svg.contains("</svg>"), "output must contain </svg: {svg}");
}

/// Verifies that the PSystemBuilder pipeline and the bypass pipeline produce
/// identical SVG for a simple sequence diagram.
#[test]
fn test_pipeline_matches_bypass() {
    // Render via the unified API (uses PSystemBuilder pipeline first).
    let pipeline_svg = render_svg(SIMPLE_SEQUENCE).expect("pipeline render should succeed");

    // Render via the bypass directly.
    let parsed = plantuml_engine::sequence_renderer::parse_simple_sequence(SIMPLE_SEQUENCE)
        .expect("bypass parse should succeed");
    let bypass_svg = parsed.render();

    assert_eq!(
        pipeline_svg, bypass_svg,
        "PSystemBuilder pipeline SVG must match bypass pipeline SVG"
    );
}

/// Verifies that the `render` function dispatches SVG correctly.
#[test]
fn test_render_dispatches_svg() {
    let result = plantuml_engine::render(SIMPLE_SEQUENCE, FileFormat::Svg);
    assert!(result.is_ok(), "render(Svg) should succeed");
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "must produce SVG");
}

/// Verifies that JSON diagrams are now supported via the pipeline.
#[test]
fn test_json_renders_svg() {
    let source = "@startjson\n{\"key\": \"value\"}\n@endjson";
    let result = render_svg(source);
    assert!(result.is_ok(), "JSON should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "JSON must produce SVG");
    assert!(svg.contains("key"), "JSON SVG must contain the key");
}

/// Verifies that YAML diagrams are now supported via the pipeline.
#[test]
fn test_yaml_renders_svg() {
    let source = "@startyaml\nkey: value\n@endyaml";
    let result = render_svg(source);
    assert!(result.is_ok(), "YAML should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "YAML must produce SVG");
}

/// Verifies that mindmap diagrams render via the pipeline.
#[test]
fn test_mindmap_renders_svg() {
    let source = "@startmindmap\n* root\n** a\n*** a1\n** b\n@endmindmap";
    let result = render_svg(source);
    assert!(result.is_ok(), "Mindmap should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Mindmap must produce SVG");
    assert!(svg.contains("root"), "Mindmap SVG must contain root label");
}

/// Verifies that WBS diagrams render via the pipeline.
#[test]
fn test_wbs_renders_svg() {
    let source = "@startwbs\n* Project\n** Planning\n*** Define scope\n** Implementation\n@endwbs";
    let result = render_svg(source);
    assert!(result.is_ok(), "WBS should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "WBS must produce SVG");
    assert!(svg.contains("Project"), "WBS SVG must contain Project label");
}

/// Verifies that class diagrams render via the pipeline.
#[test]
fn test_class_renders_svg() {
    let source = "@startuml\nclass Alice\nclass Bob\nAlice --> Bob : knows\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Class diagram should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Class diagram must produce SVG");
    assert!(svg.contains("Alice"), "Class SVG must contain Alice label");
    assert!(svg.contains("Bob"), "Class SVG must contain Bob label");
}

/// Verifies that object diagrams render via the pipeline.
#[test]
fn test_object_renders_svg() {
    let source = "@startuml\nobject alice\nobject bob\nalice --> bob : link\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Object diagram should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Object diagram must produce SVG");
    assert!(svg.contains("alice"), "Object SVG must contain alice label");
}

/// Verifies that class diagrams with body content (fields/methods) render.
#[test]
fn test_class_with_body_renders_svg() {
    let source = "@startuml\nclass Foo {\n+ field: int\n+ method(): void\n}\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Class with body should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Class with body must produce SVG");
    assert!(svg.contains("field"), "Class SVG must contain field");
    assert!(svg.contains("method"), "Class SVG must contain method");
}

/// Verifies that state diagrams render via the pipeline.
#[test]
fn test_state_renders_svg() {
    let source = "@startuml\nstate Idle\nstate Active\nIdle --> Active : start\nActive --> Idle : stop\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "State diagram should render via pipeline: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "State diagram must produce SVG");
    assert!(svg.contains("Idle"), "State SVG must contain Idle label");
    assert!(svg.contains("Active"), "State SVG must contain Active label");
}

/// Verifies that component diagrams render via the pipeline.
#[test]
fn test_component_renders_svg() {
    let source = "@startuml\ncomponent [Web Server]\ncomponent [Database]\n[Web Server] --> [Database] : queries\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Component diagram should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Component diagram must produce SVG");
}

/// Verifies that use case diagrams render via the pipeline.
#[test]
fn test_usecase_renders_svg() {
    let source = "@startuml\nactor User\nusecase (Login)\nusecase (Logout)\nUser --> (Login)\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "UseCase diagram should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "UseCase diagram must produce SVG");
    assert!(svg.contains("User"), "UseCase SVG must contain User");
}

/// Verifies that deployment diagrams render via the pipeline.
#[test]
fn test_deployment_renders_svg() {
    let source = "@startuml\nnode \"Server\" {\n  database \"DB\"\n}\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Deployment diagram should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Deployment diagram must produce SVG");
}

/// Verifies that timing diagrams render via the pipeline.
#[test]
fn test_timing_renders_svg() {
    let source = "@starttiming\nbinary A\nbinary B\nA = 0\nB = 0\nA = 1\nB = 1\n@endtiming";
    let result = render_svg(source);
    assert!(result.is_ok(), "Timing diagram should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Timing diagram must produce SVG");
    assert!(svg.contains("A"), "Timing SVG must contain signal A");
    assert!(svg.contains("B"), "Timing SVG must contain signal B");
}

/// Verifies that gantt diagrams render via the pipeline.
#[test]
fn test_gantt_renders_svg() {
    let source = "@startgantt\ntask [Task1] lasts 5 days\ntask [Task2] lasts 3 days\n[Task2] depends on [Task1]\n@endgantt";
    let result = render_svg(source);
    assert!(result.is_ok(), "Gantt diagram should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Gantt diagram must produce SVG");
    assert!(svg.contains("Task1"), "Gantt SVG must contain Task1");
    assert!(svg.contains("Task2"), "Gantt SVG must contain Task2");
}

/// Verifies that activity diagrams render via the pipeline.
#[test]
fn test_activity_renders_svg() {
    let source = "@startuml\nstart\n:Do something;\nstop\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Activity diagram should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Activity diagram must produce SVG");
    assert!(svg.contains("Do something"), "Activity SVG must contain action label");
}

/// Verifies that activity diagrams with if/else render.
#[test]
fn test_activity_if_else_renders_svg() {
    let source = "@startuml\nstart\nif (cond?) then (yes)\n:yes action;\nelse (no)\n:no action;\nendif\nstop\n@enduml";
    let result = render_svg(source);
    assert!(result.is_ok(), "Activity if/else should render: {:?}", result);
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Activity if/else must produce SVG");
    assert!(svg.contains("cond?"), "Activity SVG must contain condition");
    assert!(svg.contains("yes action"), "Activity SVG must contain yes branch");
}

/// Verifies that a truly unsupported diagram type returns ParseFailed.
#[test]
fn test_unsupported_type_returns_parse_failed() {
    let source = "@startditaa\nsome ascii art\n@endditaa";
    let result = render_svg(source);
    // No factory handles ditaa, and bypass can't parse it either.
    assert!(result.is_err(), "ditaa should fail (no factory)");
}

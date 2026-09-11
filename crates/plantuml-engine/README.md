# plantuml-engine

The main PlantUML engine — top-level pipeline and unified render API.

Ported from `net.sourceforge.plantuml` package (root).

## Overview

This crate is the public API entry point for parsing and rendering PlantUML diagrams. It implements the full pipeline: splitting source into `@startuml`/`@enduml` blocks (`BlockUmlBuilder`), lazy preprocessing per block (`BlockUml`), diagram-type dispatch (`PSystemBuilder`), and rendering to the requested output format.

Currently supports SVG (sequence diagrams) and PREPROC (preprocessed text) output formats.

## Modules

| Module | Description |
|--------|-------------|
| `render` | Unified render API: `render`, `render_svg`, `render_preproc`, `RenderError` |
| `block_uml_builder` | `BlockUmlBuilder` — splits source into `@start`/`@end` blocks |
| `block_uml` | `BlockUml` — one block; lazy preprocessing; `get_diagram()` dispatch |
| `source_string_reader` | `SourceStringReader` — programmatic API entry point |
| `sequence_renderer` | Sequence diagram parsing and SVG rendering |
| `uml_source` | `UmlSource` — preprocessed source lines + candidate diagram types |
| `p_system_factory` | `PSystemFactory` trait — interface for diagram-type factories |
| `p_system_builder` | `PSystemBuilder` — factory registry + dispatch to first matching factory |
| `p_system_command_factory` | `PSystemCommandFactory<D>` — generic command-based factory |
| `sequence_factory` | `SequenceDiagramFactory` — wraps bypass pipeline as `PSystemFactory` |
| `definitions_container` | `DefinitionsContainer` — shared definitions state |
| `error_uml` | `ErrorUml` / `ErrorUmlType` — error diagram rendering |

## Key Exports

- `render_svg(source: &str) -> Result<String, RenderError>` — render to SVG (tries `PSystemBuilder` pipeline first, falls back to bypass)
- `render_preproc(source: &str) -> Result<String, RenderError>` — render to preprocessed text
- `render(source: &str, format: FileFormat) -> Result<String, RenderError>` — dispatch by format
- `SourceStringReader` — programmatic API entry point
- `BlockUmlBuilder` / `BlockUml` — block splitting and lazy preprocessing
- `UmlSource` — preprocessed source lines + candidate diagram types
- `PSystemFactory` — trait for diagram-type factories
- `PSystemBuilder` — factory registry + dispatch
- `PSystemCommandFactory<D>` — generic command-based factory for `Command<D>`-based diagram types
- `RenderError` — error enum (`ParseFailed`, `UnsupportedFormat`, `Utf8`, `Export`)

## Usage

```rust
use plantuml_engine::render_svg;

let svg = render_svg("@startuml\nAlice -> Bob: hello\n@enduml")?;
```

## License

MIT License (per workspace `LICENSE` file).

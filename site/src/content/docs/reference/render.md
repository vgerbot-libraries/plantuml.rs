---
title: render
description: Unified render API dispatching by FileFormat
---

# render

Renders PlantUML `source` to the requested `format`. Dispatches to the appropriate renderer based on the `FileFormat` value.

## Signature

```rust
pub fn render(source: &str, format: FileFormat) -> Result<String, RenderError>
```

## Usage

```rust
use plantuml_engine::render;
use plantuml_core::FileFormat;

let svg = render("@startuml\nAlice -> Bob: hello\n@enduml", FileFormat::Svg)?;
let preproc = render("@startuml\nAlice -> Bob: hello\n@enduml", FileFormat::Preproc)?;
```

## Supported formats

Currently supports `FileFormat::Svg` and `FileFormat::Preproc`. All other formats return `RenderError::UnsupportedFormat`.

## Errors

- `RenderError::ParseFailed` — the source could not be parsed as a supported diagram type.
- `RenderError::UnsupportedFormat(FileFormat)` — the requested format is not yet implemented.
- `RenderError::Utf8` — the rendered output was not valid UTF-8.

---
title: Rust API
description: Using the plantuml-engine render API from Rust
---

# Rust API

The `plantuml-engine` crate provides the unified render API — the same entry point used by all language bindings.

## `render_svg`

Renders PlantUML source to an SVG string. Returns `RenderError::ParseFailed` when the source cannot be parsed as a supported diagram type (currently sequence diagrams only).

```rust
use plantuml_engine::render_svg;

let svg = render_svg("@startuml\nAlice -> Bob: hello\n@enduml")?;
```

Signature:

```rust
pub fn render_svg(source: &str) -> Result<String, RenderError>
```

## `render_preproc`

Renders PlantUML source to PREPROC (preprocessed) text — the output of the TIM engine after applying `!include`, `!define`, variables, and conditionals.

```rust
use plantuml_engine::render_preproc;

let text = render_preproc("@startuml\n!define FOO bar\nAlice -> Bob: FOO\n@enduml")?;
```

Signature:

```rust
pub fn render_preproc(source: &str) -> Result<String, RenderError>
```

## `render`

Dispatches to the appropriate renderer based on the requested `FileFormat`. Currently supports `FileFormat::Svg` and `FileFormat::Preproc`; all other formats return `RenderError::UnsupportedFormat`.

```rust
use plantuml_engine::render;
use plantuml_core::FileFormat;

let svg = render("@startuml\nAlice -> Bob: hello\n@enduml", FileFormat::Svg)?;
```

Signature:

```rust
pub fn render(source: &str, format: FileFormat) -> Result<String, RenderError>
```

## Error handling

`RenderError` is the error type returned by all three functions:

```rust
pub enum RenderError {
    ParseFailed,
    UnsupportedFormat(FileFormat),
    Utf8(std::string::FromUtf8Error),
}
```

- `ParseFailed` — the source could not be parsed as a supported diagram type.
- `UnsupportedFormat(FileFormat)` — the requested format is not yet implemented.
- `Utf8` — the rendered output was not valid UTF-8.

## Add to your project

```sh
cargo add plantuml-engine plantuml-core
```

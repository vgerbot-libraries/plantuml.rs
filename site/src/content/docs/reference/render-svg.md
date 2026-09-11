---
title: renderSvg
description: Render PlantUML source to SVG
---

# renderSvg

Renders PlantUML `source` to an SVG string.

## Signature

```rust
pub fn render_svg(source: &str) -> Result<String, RenderError>
```

## Usage

```rust
use plantuml_engine::render_svg;

let svg = render_svg("@startuml\nAlice -> Bob: hello\n@enduml")?;
println!("{svg}");
```

## Errors

Returns `RenderError::ParseFailed` when the source cannot be parsed as a supported diagram type. The function tries the `PSystemBuilder` pipeline first (preprocess → factory dispatch → `Diagram::export_diagram`), then falls back to the legacy bypass pipeline for sequence diagrams. Returns `RenderError::Utf8` if the rendered output is not valid UTF-8. Returns `RenderError::Export` if the diagram factory succeeds but export fails.

## TypeScript equivalent

```typescript
import { renderSvg } from '@vgerbot/plantuml';
const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
```

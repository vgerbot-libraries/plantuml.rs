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

Returns `RenderError::ParseFailed` when the source cannot be parsed as a supported diagram type (currently sequence diagrams only). Returns `RenderError::Utf8` if the rendered output is not valid UTF-8.

## TypeScript equivalent

```typescript
import { renderSvg } from '@vgerbot/plantuml';
const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
```

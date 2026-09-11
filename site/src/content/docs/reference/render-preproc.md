---
title: renderPreproc
description: Render PlantUML source to preprocessed text
---

# renderPreproc

Renders PlantUML `source` to PREPROC (preprocessed) text — the output of the TIM engine after applying `!include`, `!define`, variables, and conditionals.

## Signature

```rust
pub fn render_preproc(source: &str) -> Result<String, RenderError>
```

## Usage

```rust
use plantuml_engine::render_preproc;

let text = render_preproc("@startuml\n!define FOO bar\nAlice -> Bob: FOO\n@enduml")?;
println!("{text}");
```

## Errors

Returns `RenderError::Utf8` if the rendered output is not valid UTF-8.

## TypeScript equivalent

```typescript
import { renderPreproc } from '@vgerbot/plantuml';
const text = await renderPreproc('@startuml\n!define FOO bar\nAlice -> Bob: FOO\n@enduml');
```

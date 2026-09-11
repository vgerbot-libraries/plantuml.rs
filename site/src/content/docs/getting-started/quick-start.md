---
title: Quick Start
description: Render your first PlantUML diagram in Rust, TypeScript, and Java
---

# Quick Start

Render a sequence diagram using the binding of your choice.

## Rust

```sh
cargo add plantuml-engine
```

```rust
use plantuml_engine::render_svg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg = render_svg("@startuml\nAlice -> Bob: hello\nBob --> Alice: hi\n@enduml")?;
    println!("{svg}");
    Ok(())
}
```

The `render` function dispatches by `FileFormat`:

```rust
use plantuml_engine::render;
use plantuml_core::FileFormat;

let svg = render("@startuml\nAlice -> Bob: hello\n@enduml", FileFormat::Svg)?;
```

## TypeScript (browser or Node.js)

```sh
npm install @vgerbot/plantuml
```

```typescript
import { renderSvg } from '@vgerbot/plantuml';

const svg = await renderSvg('@startuml\nAlice -> Bob: hello\nBob --> Alice: hi\n@enduml');
console.log(svg);
```

The binding auto-detects the environment: in the browser it uses the `web` WASM target (ES module + `fetch`); in Node.js it uses the `nodejs` target (`require` + `fs`).

## Java

Maven coordinates:

```xml
<dependency>
    <groupId>com.vgerbot.plantuml</groupId>
    <artifactId>plantuml-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

```java
import com.vgerbot.plantuml.PlantUml;

String svg = PlantUml.renderSvg("@startuml\nAlice -> Bob: hello\nBob --> Alice: hi\n@enduml");
System.out.println(svg);
```

## Python

Planned (PyO3 + maturin). Not yet implemented.

## Next

- [Language Reference](/language/) — learn the diagram syntax
- [Playground](/playground/) — experiment in the browser

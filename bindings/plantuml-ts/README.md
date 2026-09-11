# @vgerbot/plantuml

Rust-powered PlantUML renderer for browser and Node.js via WebAssembly.

## Build

```sh
npm install
npm run build
```

This builds the WASM crate (browser + Node.js targets) and compiles the
TypeScript wrapper.

## Usage

```typescript
import { renderSvg } from '@vgerbot/plantuml';

const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
```

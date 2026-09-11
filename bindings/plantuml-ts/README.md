# @vgerbot/plantuml

Rust-powered PlantUML renderer for browser and Node.js via WebAssembly.

## Overview

This is the TypeScript binding for [plantuml.rs](../../README.md). It wraps the Rust `plantuml-wasm` crate (compiled to WebAssembly via `wasm-bindgen`) with an environment-detecting loader. The package ships pre-built WASM modules for both browser (`--target web`) and Node.js (`--target nodejs`) targets inside the npm package — no external paths or build steps needed at runtime.

## Installation

```sh
npm install @vgerbot/plantuml
```

## API

```typescript
// Render PlantUML source to SVG.
export function renderSvg(source: string): Promise<string>;

// Render PlantUML source to PREPROC (preprocessed) text.
export function renderPreproc(source: string): Promise<string>;
```

Both functions auto-detect the environment (browser vs Node.js) on first call and load the appropriate WASM module. Subsequent calls reuse the loaded module.

## Usage

### Browser (ES module)

```typescript
import { renderSvg } from '@vgerbot/plantuml';

const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
document.getElementById('diagram').innerHTML = svg;
```

### Node.js

```typescript
import { renderSvg } from '@vgerbot/plantuml';

const svg = await renderSvg('@startuml\nAlice -> Bob: hello\n@enduml');
console.log(svg);
```

## Package Contents

```
@vgerbot/plantuml/
├── dist/           # Compiled TypeScript (index.js, index.d.ts)
└── wasm/
    ├── web/        # Browser WASM module (wasm-pack --target web)
    └── node/        # Node.js WASM module (wasm-pack --target nodejs)
```

## Build

```sh
npm install
npm run build
```

This runs `wasm-pack` for both targets (web + nodejs), cleans generated `.gitignore` files, and compiles TypeScript via `tsc`.

## Version

`0.1.0` (see `package.json`)

## License

MIT License (per workspace `LICENSE` file).

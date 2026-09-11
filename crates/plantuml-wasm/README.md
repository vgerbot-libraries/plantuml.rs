# plantuml-wasm

WASM bindings for browser and Node.js.

This crate has no Java source equivalent — it is new code that exposes the `plantuml-engine` render API via `wasm-bindgen` for use in JavaScript/TypeScript.

## Overview

Compiles the Rust engine to WebAssembly using `wasm-bindgen`. Exposes thin wrapper functions that are callable from JavaScript. Parse failures return an empty string (not an error), keeping the JS API simple.

## Exports

```typescript
// Render PlantUML source to SVG. Returns empty string on parse failure.
export function render_svg(source: string): string;

// Render PlantUML source to PREPROC text. Returns empty string on failure.
export function render_preproc(source: string): string;
```

## crate-type

`cdylib`, `rlib` — produces a WASM module loadable in browser and Node.js, plus an rlib for Rust consumers.

## Build

```sh
# Browser target (ES module + fetch)
wasm-pack build --target web

# Node.js target (require + fs)
wasm-pack build --target nodejs
```

The TypeScript binding package (`bindings/plantuml-ts`) runs both targets and wraps them with an environment-detecting loader. See [`bindings/plantuml-ts/README.md`](../../bindings/plantuml-ts/README.md) for the npm package.

## License

MIT License (per workspace `LICENSE` file).

---
title: WASM Build
description: Building the WASM module from source with wasm-pack
---

# WASM Build

The TypeScript binding ships pre-built WASM, but you can rebuild it from source with `wasm-pack`.

## Prerequisites

```sh
cargo install wasm-pack
```

## Build

From the `bindings/plantuml-ts` directory:

```sh
npm run build
```

This runs three steps:

1. `wasm-pack build --target web` — builds the browser WASM module into `wasm/web/`
2. `wasm-pack build --target nodejs` — builds the Node.js WASM module into `wasm/node/`
3. `tsc` — compiles the TypeScript wrapper into `dist/`

## Build a single target

To build only the browser target:

```sh
npm run build:wasm:web
```

To build only the Node.js target:

```sh
npm run build:wasm:node
```

## Targets

| Target | Output | Use case |
|--------|--------|----------|
| `web` | `wasm/web/plantuml_wasm.js` + `.wasm` | Browser (ES module, `fetch`-based init) |
| `nodejs` | `wasm/node/plantuml_wasm.js` + `.wasm` | Node.js (`require`-based, auto-init) |

The `web` target exports `render_svg`, `render_preproc`, and a `default()` init function that accepts a URL to the `.wasm` file. The `nodejs` target exports `render_svg` and `render_preproc` directly (auto-initialized on require).

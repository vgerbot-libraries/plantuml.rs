// Auto-detects environment and loads the appropriate WASM build.
// Browser: uses the `web` target (ES module + fetch).
// Node.js: uses the `nodejs` target (require + fs).
//
// The WASM modules are pre-built into ./wasm/{web,node} by `npm run build`
// and shipped inside the npm package, so no external paths are needed.

import * as path from 'path';

/** Minimal interface for the wasm-pack generated module. */
interface WasmModule {
  render_svg(source: string): string;
  render_preproc(source: string): string;
}

let wasmModule: WasmModule | null = null;

async function loadWasm(): Promise<WasmModule> {
  if (wasmModule) return wasmModule;
  // wasm/ sits next to dist/ inside the package root.
  const wasmDir = path.resolve(__dirname, '..', 'wasm');
  if (typeof window !== 'undefined') {
    // Browser — dynamic import is required: the `web` target module uses
    // `import.meta.url` and `fetch`, which only exist in browser scope.
    const wasmPath = path.join(wasmDir, 'web', 'plantuml_wasm.js');
    const wasm = (await import(wasmPath)) as unknown as WasmModule & { default(): Promise<void> };
    await wasm.default();
    wasmModule = wasm;
  } else {
    // Node.js
    const wasmPath = path.join(wasmDir, 'node', 'plantuml_wasm.js');
    const wasm = require(wasmPath) as WasmModule;
    wasmModule = wasm;
  }
  return wasmModule;
}

export async function renderSvg(source: string): Promise<string> {
  const wasm = await loadWasm();
  return wasm.render_svg(source);
}

export async function renderPreproc(source: string): Promise<string> {
  const wasm = await loadWasm();
  return wasm.render_preproc(source);
}

import { cpSync, mkdirSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const srcDir = resolve(__dirname, '../../bindings/plantuml-ts/wasm/web');
const destDir = resolve(__dirname, '../public/wasm');

export function syncWasm() {
  const jsSrc = resolve(srcDir, 'plantuml_wasm.js');
  const wasmSrc = resolve(srcDir, 'plantuml_wasm_bg.wasm');
  if (!existsSync(jsSrc) || !existsSync(wasmSrc)) {
    console.warn('[sync-wasm] WASM files not found, skipping sync. Run `cargo build -p plantuml-wasm` first.');
    return;
  }
  mkdirSync(destDir, { recursive: true });
  cpSync(jsSrc, resolve(destDir, 'plantuml_wasm.js'));
  cpSync(wasmSrc, resolve(destDir, 'plantuml_wasm_bg.wasm'));
  console.log('WASM files synced to public/wasm/');
}

syncWasm();

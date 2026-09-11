import { cpSync, mkdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const srcDir = resolve(__dirname, '../../bindings/plantuml-ts/wasm/web');
const destDir = resolve(__dirname, '../public/wasm');

mkdirSync(destDir, { recursive: true });
cpSync(resolve(srcDir, 'plantuml_wasm.js'), resolve(destDir, 'plantuml_wasm.js'));
cpSync(resolve(srcDir, 'plantuml_wasm_bg.wasm'), resolve(destDir, 'plantuml_wasm_bg.wasm'));
console.log('WASM files synced to public/wasm/');

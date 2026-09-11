// Type declarations for the wasm-pack generated modules.
// The actual modules are generated at build time by `wasm-pack`.

declare module '*/plantuml_wasm.js' {
  export function render_svg(source: string): string;
  export function render_preproc(source: string): string;
  const _default: () => Promise<void>;
  export default _default;
}

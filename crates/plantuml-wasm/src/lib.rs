#![allow(unexpected_cfgs)]
//! WASM bindings for browser and Node.js.
//!
//! Exposes thin wrappers around the `plantuml-engine` render API that are
//! callable from JavaScript via `wasm-bindgen`.
//!
//! Ported from: plantuml-wasm (new, no Java source).

use wasm_bindgen::prelude::*;

/// Renders `PlantUML` source to SVG. Returns an empty string on parse failure.
#[must_use]
#[wasm_bindgen]
pub fn render_svg(source: &str) -> String {
    plantuml_engine::render_svg(source).unwrap_or_default()
}

/// Renders `PlantUML` source to PREPROC text. Returns an empty string on failure.
#[must_use]
#[wasm_bindgen]
pub fn render_preproc(source: &str) -> String {
    plantuml_engine::render_preproc(source).unwrap_or_default()
}

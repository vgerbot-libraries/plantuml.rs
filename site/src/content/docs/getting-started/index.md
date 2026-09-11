---
title: Overview
description: What plantuml.rs is and its current status
---

# Overview

**plantuml.rs** is a from-scratch Rust reimplementation of [PlantUML](https://plantuml.com/), the widely-used Java library for generating diagrams from plain-text descriptions. The goal is behavioral parity with the Java original, plus language bindings for Java, TypeScript, and Python via FFI.

## Current status

**In progress.** The following features are working and tested against the Java reference:

- Sequence diagram parsing (`plantuml-sequence`)
- SVG rendering backend (`plantuml-svg`)
- PREPROC output (preprocessed text via the TIM engine, `plantuml-preproc`)
- Unified render API: `render_svg`, `render_preproc`, `render(source, format)` (`plantuml-engine`)
- Java binding via JNI — `PlantUml.renderSvg` / `PlantUml.renderPreproc`
- TypeScript binding via WASM — `renderSvg` / `renderPreproc` (browser + Node.js)
- 1D constraint solver for layout positioning (`plantuml-real`)
- Custom regex engine (`plantuml-regex`)
- Preprocessor: `!include`, `!define`, variables, conditionals (`plantuml-preproc`)
- Command parsing framework (`plantuml-command`)
- 2D graphics primitives (`plantuml-klimt`)
- Skin / style / theme system (`plantuml-skin`)

## Planned

- Class, activity, use case, and other diagram types
- PNG, PDF, LaTeX/TikZ rendering backends
- Layout engines (Graphviz / ELK)
- CLI binary (`plantuml-cli`)
- Python binding (PyO3 + maturin)
- C FFI shared library for additional language bindings

## Next

- [Installation](/getting-started/installation/) — set up the toolchain and build
- [Quick Start](/getting-started/quick-start/) — render your first diagram

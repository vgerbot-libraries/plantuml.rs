---
title: Installation
description: Prerequisites and build instructions for plantuml.rs
---

# Installation

## Prerequisites

- **Rust toolchain** (edition 2021, `resolver = "2"`) — install via [rustup](https://rustup.rs/)
- **Node.js** 18+ — required for the TypeScript binding
- **Java 17+** — required only for the Java binding
- **wasm-pack** — required only to rebuild the WASM module (`cargo install wasm-pack`)

## Build the core library

```sh
git clone https://github.com/vgerbot/plantuml.rs.git
cd plantuml.rs
cargo build
```

## Run the tests

Tests are ported directly from the PlantUML Java reference:

```sh
cargo test --workspace
```

## Build the TypeScript binding

The TypeScript binding (`@vgerbot/plantuml`) compiles the Rust core to WASM and wraps it with a TypeScript API.

```sh
cd bindings/plantuml-ts
npm install
npm run build
```

This runs `wasm-pack` for both `web` and `nodejs` targets, then compiles the TypeScript wrapper. The output lands in `bindings/plantuml-ts/dist/` and `bindings/plantuml-ts/wasm/`.

## Build the Java binding

```sh
cd bindings/plantuml-java
./gradlew build
```

This produces a JAR with JNI bindings exposing `com.vgerbot.plantuml.PlantUml`.

## Lint

```sh
cargo clippy --workspace -- -D warnings
```

## Next

- [Quick Start](/getting-started/quick-start/) — render your first diagram

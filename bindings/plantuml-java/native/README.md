# plantuml-jni

JNI bridge for Java bindings.

This crate has no Java source equivalent — it is new code that bridges the `plantuml-engine` render API to Java via the JNI (Java Native Interface).

## Overview

Implements the native methods declared by `com.vgerbot.plantuml.PlantUml`. Each call extracts the Java `String` argument, renders via `plantuml-engine`, and returns a new Java `String`. Parse failures yield an empty string.

## Native Methods

| JNI Symbol | Java Declaration | Description |
|-----------|-------------------|-------------|
| `Java_com_vgerbot_plantuml_PlantUml_renderSvg` | `static native String renderSvg(String)` | Render to SVG |
| `Java_com_vgerbot_plantuml_PlantUml_renderPreproc` | `static native String renderPreproc(String)` | Render to PREPROC text |

## crate-type

`cdylib` — produces a native shared library (`libplantuml_jni.so` / `.dylib` / `plantuml_jni.dll`) loaded by the JVM.

## Build

```sh
cargo build --release
```

The Java binding's `build.gradle` automates this: it runs `cargo build --release` and packages the resulting native library into the JAR under `native/`. See [`bindings/plantuml-java/README.md`](../README.md) for the full Java binding.

## License

MIT License (per workspace `LICENSE` file).

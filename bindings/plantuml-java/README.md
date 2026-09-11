# plantuml-java

Java binding for [plantuml.rs](../../README.md) — Rust-powered PlantUML rendering via JNI.

## Overview

This package provides a Java JAR that loads a native Rust library (built from [`plantuml-jni`](native/README.md)) and exposes static methods for rendering PlantUML source to SVG or PREPROC text. The native library is extracted from the JAR at runtime, so no separate installation is needed in production.

## Maven Coordinates

```xml
<dependency>
    <groupId>com.vgerbot.plantuml</groupId>
    <artifactId>plantuml-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

## Requirements

- Java 17+ (source and target compatibility)
- Native library is platform-specific: `libplantuml_jni.so` (Linux), `libplantuml_jni.dylib` (macOS), `plantuml_jni.dll` (Windows)

## API

```java
package com.vgerbot.plantuml;

public class PlantUml {
    public static native String renderSvg(String source);
    public static native String renderPreproc(String source);
}
```

## Usage

```java
import com.vgerbot.plantuml.PlantUml;

String svg = PlantUml.renderSvg("@startuml\nAlice -> Bob: hello\n@enduml");
System.out.println(svg);
```

## Native Library Loading

The library is loaded in the `PlantUml` static initializer:

1. **`java.library.path`** — tries `System.loadLibrary("plantuml_jni")` first (development / system-installed).
2. **JAR extraction** — falls back to extracting the native library from `/native/` resources inside the JAR to a temp file, then loading it (production / packaged).

## Build

```sh
./gradlew build
```

This:
1. Runs `cargo build --release` in `native/` (builds the JNI native library).
2. Copies the resulting `libplantuml_jni.so` / `.dylib` / `.dll` into `build/resources/main/native/`.
3. Compiles Java sources.
4. Packages everything into a JAR.

## Project Structure

```
bindings/plantuml-java/
├── build.gradle                    # Gradle build (Java + native)
├── src/main/java/com/vgerbot/plantuml/
│   └── PlantUml.java                # Java API (native method declarations)
└── native/                          # Rust JNI crate (plantuml-jni)
    ├── Cargo.toml
    ├── src/lib.rs
    └── README.md
```

## Version

`0.1.0` (see `build.gradle`)

## License

MIT License (per workspace `LICENSE` file).

# Reference Repo Usage Rules

Rules for using the `temp/plantuml/` reference repository.

## Location

- `temp/plantuml/` — a shallow clone (`--depth 1`) of the PlantUML reference repo.
- This directory is gitignored and is NOT part of the plantuml.rs project. It exists solely as a read-only reference.

## Purpose

- Read-only reference for porting Java code to Rust.
- **Never modify** files in `temp/`. Treat it as a frozen upstream snapshot.
- When porting a Java class, read the Java source, understand its responsibility, then write idiomatic Rust that preserves the same observable behavior. Do not mechanically translate Java syntax.

## Key Paths

| Path | Contents |
|------|----------|
| `temp/plantuml/src/main/java/net/sourceforge/plantuml/` | Main source tree (112 sub-packages) |
| `temp/plantuml/src/main/java/net/sourceforge/plantuml/SourceStringReader.java` | Programmatic API entry point |
| `temp/plantuml/src/main/java/net/sourceforge/plantuml/BlockUml.java` | Pipeline unit (a `@start`/`@end` block) |
| `temp/plantuml/src/main/java/net/sourceforge/plantuml/PSystemBuilder.java` | Diagram-type dispatcher (factory + registry) |
| `temp/plantuml/src/main/java/net/sourceforge/plantuml/FileFormat.java` | Output format enum (SVG, PNG, PDF, LaTeX, EPS, etc.) |
| `temp/plantuml/src/main/java/com/plantuml/ubrex/` | Custom regex engine (to be ported as `plantuml-regex`) |
| `temp/plantuml/src/main/java/gen/lib/` | Generated Graphviz C-to-Java port |
| `temp/plantuml/build.gradle.kts` | Build config and dependency list |
| `temp/plantuml/gradle/libs.versions.toml` | Version catalog (Java dependency versions) |

## How to Reference

1. **Before porting a class**: read its Java source to understand its responsibility, collaborators, and role in the pipeline.
2. **During porting**: cite the Java source path in a doc comment at the top of the Rust file:
   ```rust
   /// Ported from: net/sourceforge/plantuml/SourceStringReader.java
   ```
3. **After porting**: verify the Rust implementation produces the same observable behavior for representative inputs.

## Updating the Reference

- To update to the latest PlantUML: `git -C temp/plantuml pull`
- To deepen the shallow clone (if history is needed): `git -C temp/plantuml fetch --unshallow`
- To re-clone from scratch: `rm -rf temp/plantuml && git clone --depth 1 https://github.com/plantuml/plantuml.git temp/plantuml`

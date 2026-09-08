# Java-to-Rust Porting Rules

Rules for porting Java PlantUML code to Rust. These rules govern every porting task in the plantuml.rs project.

## Naming Conventions

- **File names**: Java `PascalCase.java` → Rust `snake_case.rs` (e.g., `SourceStringReader.java` → `source_string_reader.rs`).
- **Type names**: Java PascalCase type names stay PascalCase in Rust (e.g., `SourceStringReader` → `SourceStringReader`).
- **Method names**: Java `camelCase` methods → Rust `snake_case` functions (e.g., `getDiagram()` → `get_diagram()`).
- **Constants**: Java `UPPER_SNAKE_CASE` static finals → Rust `UPPER_SNAKE_CASE` associated constants.
- **Package → crate path**: Java package `net.sourceforge.plantuml.foo.bar` → Rust crate path `foo/bar/` (strip `net.sourceforge.plantuml` prefix).

## File Mapping

- **1:1 mapping**: Each Java source file maps to exactly one Rust source file. The Rust file preserves the same logical responsibility.
- **Exclude**: `package-info.java` and `module-info.java` have no Rust equivalent — skip them.
- **Rust-only files**: `mod.rs` (module declarations) and `lib.rs` / `main.rs` (crate roots) have no Java counterpart.
- **One file per class**: If a Java file contains multiple top-level classes (common with inner classes), extract inner classes into separate files or keep as nested modules, depending on size and cohesion.

## File Size Governance

| Lines | Action |
|-------|--------|
| ≤ 500 | Normal — no action needed. |
| 501–800 | Review for cohesion. If the file has clear sub-responsibilities, split. If cohesive, keep with a `// REVIEW: large file, cohesive` note. |
| > 800 | Must split. Break into smaller modules before merging. |

## OOP → Rust Conversion

| Java Construct | Rust Equivalent |
|----------------|-----------------|
| `interface` | `trait` |
| `abstract class` | `trait` with default method implementations |
| `class` | `struct` (or `enum` if it's a closed hierarchy) |
| `enum` (simple) | Rust `enum` |
| `enum` (with data) | Rust `enum` with variants |
| `static` methods | Associated functions (`impl Type { fn ... }`) |
| `static` fields | Associated `const` or `static` |
| Overloaded constructors | Builder pattern, or `From`/`Default` traits |
| `instanceof` checks | `match` on enum variant, or `downcast` via `Any` (rare) |
| Inner classes | Separate file or nested `mod` |
| Anonymous classes | Closures (`Fn`/`FnMut`/`FnOnce`) |
| `final` variables | `let` bindings (immutable by default) |
| `this` | `self` / `&self` / `&mut self` |

## Error Handling

- Java exceptions → Rust `Result<T, E>`.
- Library crates: use `thiserror` for error enums. No panics in library code.
- CLI/application crates: use `anyhow` for error aggregation.
- Java `RuntimeException` (unchecked) → Rust `Result` variant (do not panic).
- Java `try`/`catch`/`finally` → Rust `?` operator and `match`; `finally` → `Drop` trait or explicit cleanup.
- Never use `unwrap()` or `expect()` in library code except in `#[cfg(test)]` modules.

## Generics

- Java `? extends T` (upper bound wildcard) → Rust `+ T` bound (e.g., `fn foo<T: SomeTrait>()`).
- Java `? super T` (lower bound wildcard) → Rust: restructure the API (Rust has no direct equivalent; use a different trait bound or callback pattern).
- Java raw types (erased generics) → Rust concrete types with `Box<dyn Trait>` where dynamic dispatch is needed.
- Java `T extends Comparable<T>` → Rust `T: Ord` or `T: PartialOrd`.

## Collections

| Java Type | Rust Equivalent |
|-----------|----------------|
| `List<T>` | `Vec<T>` |
| `ArrayList<T>` | `Vec<T>` |
| `LinkedList<T>` | `std::collections::LinkedList<T>` or `VecDeque<T>` |
| `Map<K, V>` | `HashMap<K, V>` |
| `HashMap<K, V>` | `HashMap<K, V>` |
| `TreeMap<K, V>` (ordered) | `BTreeMap<K, V>` |
| `Set<T>` | `HashSet<T>` |
| `TreeSet<T>` (ordered) | `BTreeSet<T>` |
| `Collection<T>` | `&[T]` or `impl Iterator<Item = T>` |

## Null Handling

- Java `null` → Rust `Option<T>`.
- Java `Optional<T>` → Rust `Option<T>`.
- Never use `unwrap()` / `expect()` in library code except in tests.
- Prefer `?` operator for propagation; use `ok_or` / `ok_or_else` to convert `Option` to `Result`.

## Concurrency

| Java Construct | Rust Equivalent |
|----------------|-----------------|
| `synchronized` method/block | `Mutex<T>` or `RwLock<T>` |
| `volatile` field | `std::sync::atomic::Atomic*` types |
| `AtomicInteger` / `AtomicLong` | `AtomicI32` / `AtomicI64` |
| `Thread` | `std::thread::spawn` |
| `ExecutorService` | Rayon thread pool or `std::thread` |
| `CountDownLatch` | `std::sync::Barrier` or `Condvar` |
| `ConcurrentHashMap` | `dashmap::DashMap` or `Mutex<HashMap>` |

## Reference Workflow

1. **Read the Java source** in `temp/plantuml/` before porting. Understand the class's responsibility, its collaborators, and its role in the pipeline.
2. **Write idiomatic Rust** that preserves the same observable behavior. Do not mechanically translate Java syntax.
3. **Cite the Java source** in a doc comment at the top of the Rust file:
   ```rust
   /// Ported from: net/sourceforge/plantuml/SourceStringReader.java
   ```
4. **Test against Java behavior**: where possible, write a test that exercises the same input/output as the Java original.
5. **Update `mod.rs`**: add the new module to its parent's `mod.rs` declaration.

# Test Parity Rules

Rules governing test cases, test data, and the behavioral parity goal.

## Goal

The ultimate objective is to pass **all** PlantUML Java test cases in the Rust port. Every test case and test data file in the Rust project must be directly ported from the Java reference implementation in `temp/plantuml/`.

## Test Sourcing

- **Direct port**: All test cases and test data must be ported directly from `temp/plantuml/src/test/`. Do not invent new test scenarios, inputs, or expected outputs that do not exist in the Java reference. The Rust test suite is a mirror of the Java test suite, not a replacement.
- **Test data**: Copy all test resource files (`.puml`, `.svg`, `.tex`, `.txt`, expected-output files, etc.) from `temp/plantuml/src/test/resources/` into the corresponding Rust crate's `tests/` or `testdata/` directory, preserving the directory structure.
- **1:1 mapping**: Each Java test class maps to one Rust test module or integration test file. Preserve the test name, the input diagram source, and the expected output exactly.

## Test Categories

The Java reference has three test categories. Port each as follows:

### 1. Non-regression tests (nonreg/)

- Java: `src/test/java/nonreg/` — `BasicTest` subclasses with embedded diagram source (triple-quoted `"""` blocks) and companion `*Result.java` expected-output files.
- Rust: Port each `Xxx_Test.java` as a Rust integration test in `crates/<crate>/tests/nonreg/`. Embed the diagram source and expected output as string constants (extracted from the Java triple-quoted blocks). Render via the Rust `SourceStringReader` equivalent with `FileFormat::Debug` and assert text equality.
- Port `BasicTest` logic into a shared test helper module (e.g., `tests/common/basic_test.rs`).

### 2. Vega data-driven tests (test/vega/)

- Java: `src/test/java/test/vega/` — `VegaTest` discovers `.puml` files in `src/test/resources/vega/`, renders each, and compares against the companion expected-output file.
- Rust: Copy the entire `src/test/resources/vega/` directory into the Rust test data path (e.g., `testdata/vega/`). Port `VegaTest` as a data-driven Rust integration test that discovers `.puml` files, parses the YAML header, renders via the Rust pipeline, and compares normalized text output against the expected file.
- Port `VegaChecker` subclasses and `VegaInputFile` parsing logic into Rust test helpers.

### 3. Unit and misc tests (test/, dev/, com/)

- Java: `src/test/java/test/`, `dev/`, `com/` — JUnit tests for specific APIs, utilities, and edge cases.
- Rust: Port each as a `#[test]` function in the corresponding crate's `#[cfg(test)] mod tests` block or integration test. Preserve the test name, input, and assertion logic.

## Test Data Location

- Rust test data root: `testdata/` at the workspace root (or per-crate `tests/testdata/` if preferred — see Assumptions).
- Mirror the Java directory structure: `testdata/vega/<category>/<name>.puml` + expected output files.
- Test data files are committed to the repo (not gitignored). They are part of the test suite.

## Parity Verification

- A test is "passing" when the Rust output matches the Java expected output exactly (after line-ending normalization, matching the Java `normalizeLineEndings` behavior: `\r\n` → `\n`, `\r` → `\n`).
- The end goal: `cargo test --workspace` passes all ported tests, covering every Java test case.
- When a Java test fails in Rust, fix the Rust implementation — not the test. The test data is the source of truth.

## Citing Java Test Sources

- Each Rust test file must cite the Java test source in a doc comment:
  ```rust
  /// Test ported from: nonreg/simple/A0002_Test.java
  ```
- For test data files copied verbatim, add a `testdata/vega/README.md` (or header comment) noting the source: `Copied from temp/plantuml/src/test/resources/vega/`.

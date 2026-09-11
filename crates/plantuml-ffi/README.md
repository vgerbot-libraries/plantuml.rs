# plantuml-ffi

C FFI shared library for language bindings.

This crate has no Java source equivalent — it is new code that exposes the `plantuml-engine` render API via a C ABI so that non-Rust languages can call it.

## Overview

Provides stateless C ABI functions that render PlantUML source to SVG or PREPROC text. Each call creates a fresh engine instance, renders, and returns a newly allocated C string that the caller must free with `plantuml_free_string`.

## C API

```c
// Render PlantUML source to SVG.
// Returns 0 on success (stores SVG in *out), -1 on parse error/null input,
// -2 on invalid UTF-8, -3 if output contains a NUL byte.
int plantuml_render_svg(const char *source, char **out);

// Render PlantUML source to PREPROC text. Same contract as plantuml_render_svg.
int plantuml_render_preproc(const char *source, char **out);

// Free a string previously returned by plantuml_render_svg or plantuml_render_preproc.
// Passing NULL is a no-op.
void plantuml_free_string(char *ptr);
```

## crate-type

`cdylib`, `rlib` — produces a shared library (`libplantuml_ffi.so` / `.dylib` / `.dll`) loadable via C FFI, plus an rlib for Rust consumers.

## Usage from C

```c
#include "plantuml_ffi.h"

char *svg = NULL;
int rc = plantuml_render_svg("@startuml\nAlice -> Bob: hello\n@enduml", &svg);
if (rc == 0) {
    printf("%s\n", svg);
    plantuml_free_string(svg);
}
```

A C header (`plantuml_ffi.h`) is generated at build time via `cbindgen`.

## License

MIT License (per workspace `LICENSE` file).

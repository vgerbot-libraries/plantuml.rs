---
title: YAML
description: YAML syntax and examples in plantuml.rs
---

# YAML

YAML diagrams render YAML data structures as a visual tree. plantuml.rs supports the `@startyaml`/`@endyaml` block containing raw YAML content.

## Simple key-value

Place YAML content between the `@startyaml` and `@endyaml` markers.

```plantuml
@startyaml
key: value
count: 42
@endyaml
```

## Nested mapping

Indent child keys to create nested levels in the visual tree.

```plantuml
@startyaml
name: Alice
address:
  city: NYC
  zip: "10001"
@endyaml
```

## Lists

Use YAML list syntax to render sequence elements as children of the enclosing node.

```plantuml
@startyaml
users:
  - name: Alice
    role: admin
  - name: Bob
    role: user
active: true
@endyaml
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.

---
title: Mindmap
description: Mindmap syntax and examples in plantuml.rs
---

# Mindmap

Mindmaps describe hierarchical trees of ideas radiating from a central concept. plantuml.rs supports the `@startmindmap`/`@endmindmap` block with org-mode style indentation.

## Basic tree

Use asterisk indentation to define the hierarchy. One `*` is the root, two `**` are children, three `***` are grandchildren.

```plantuml
@startmindmap
* Root idea
** First branch
*** Sub idea
*** Another sub idea
** Second branch
*** Detail
@endmindmap
```

## Plus syntax

Use `+` prefixes instead of `*` for an alternative tree syntax. Each additional `+` deepens the level.

```plantuml
@startmindmap
+ Root
++ Child A
++ Child B
+++ Grandchild
@endmindmap
```

## Left and right branches

Use `--` to grow a branch to the left of the root, and `+`/`*` to grow to the right. This produces a bidirectional mindmap.

```plantuml
@startmindmap
* Central topic
-- Left idea
--- Deeper left
++ Right idea
+++ Deeper right
@endmindmap
```

plantuml.rs uses a simplified layout algorithm for this diagram type. Pixel-perfect parity with the Java original is deferred.

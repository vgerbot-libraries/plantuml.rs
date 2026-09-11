---
title: Java Binding
description: Using the JNI binding from Java
---

# Java Binding

The Java binding exposes the Rust core via JNI. The published artifact is `com.vgerbot.plantuml:plantuml-java`.

## Maven

```xml
<dependency>
    <groupId>com.vgerbot.plantuml</groupId>
    <artifactId>plantuml-java</artifactId>
    <version>0.1.0</version>
</dependency>
```

## Usage

```java
import com.vgerbot.plantuml.PlantUml;

String svg = PlantUml.renderSvg("@startuml\nAlice -> Bob: hello\nBob --> Alice: hi\n@enduml");
System.out.println(svg);

String preproc = PlantUml.renderPreproc("@startuml\n!define FOO bar\nAlice -> Bob: FOO\n@enduml");
System.out.println(preproc);
```

## Build from source

```sh
cd bindings/plantuml-java
./gradlew build
```

This compiles the Rust native library via Cargo and packages it with the Java JNI shim into a JAR. Java 17+ is required.

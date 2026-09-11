/**
 * Standalone JNI smoke test — loads the native library directly and calls
 * the JNI functions without going through the PlantUml class (which has a
 * static initializer that extracts the library from a JAR).
 *
 * The JNI function names are Java_com_vgerbot_plantuml_PlantUml_renderSvg
 * and Java_com_vgerbot_plantuml_PlantUml_renderPreproc, so the native
 * methods must be declared in class com.vgerbot.plantuml.PlantUml.
 */
package com.vgerbot.plantuml;

public class PlantUmlTest {

    public static void main(String[] args) {
        String libPath = args.length > 0
            ? args[0]
            : "../../target/release/libplantuml_jni.so";
        System.load(libPath);

        String svg = PlantUml.renderSvg("@startuml\nAlice -> Bob: hello\n@enduml");
        if (!svg.startsWith("<svg")) {
            throw new RuntimeException("Expected SVG, got: "
                + svg.substring(0, Math.min(80, svg.length())));
        }
        System.out.println("Java JNI SVG OK");
        System.out.println(svg.substring(0, 80));

        String preproc = PlantUml.renderPreproc("@startuml\nAlice -> Bob: hello\n@enduml");
        if (preproc.isEmpty()) throw new RuntimeException("Expected non-empty PREPROC");
        System.out.println("Java JNI PREPROC OK");
    }
}

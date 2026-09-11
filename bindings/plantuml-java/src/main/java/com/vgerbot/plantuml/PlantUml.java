package com.vgerbot.plantuml;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * Rust-backed PlantUML renderer. Loads the native library from the JAR
 * and delegates to the Rust engine via JNI.
 */
public class PlantUml {
    static {
        try {
            loadNativeLibrary();
        } catch (IOException e) {
            throw new RuntimeException("Failed to load plantuml native library", e);
        }
    }

    private static void loadNativeLibrary() throws IOException {
        String osName = System.getProperty("os.name").toLowerCase();
        String libName;
        if (osName.contains("linux")) libName = "libplantuml_jni.so";
        else if (osName.contains("mac")) libName = "libplantuml_jni.dylib";
        else if (osName.contains("windows")) libName = "plantuml_jni.dll";
        else throw new IOException("Unsupported OS: " + osName);

        // Try java.library.path first (development / system-installed).
        try {
            System.loadLibrary("plantuml_jni");
            return;
        } catch (UnsatisfiedLinkError ignored) {
            // Fall through to JAR extraction.
        }

        // Extract from JAR resource (production / packaged).
        try (InputStream in = PlantUml.class.getResourceAsStream("/native/" + libName)) {
            if (in == null) throw new IOException("Native library not found: " + libName);
            Path temp = Files.createTempFile("plantuml_jni", libName.substring(libName.lastIndexOf('.')));
            temp.toFile().deleteOnExit();
            Files.copy(in, temp, StandardCopyOption.REPLACE_EXISTING);
            System.load(temp.toString());
        }
    }

    public static native String renderSvg(String source);

    public static native String renderPreproc(String source);
}

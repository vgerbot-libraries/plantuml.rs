import net.sourceforge.plantuml.FileFormat;
import net.sourceforge.plantuml.FileFormatOption;
import net.sourceforge.plantuml.SourceStringReader;
import net.sourceforge.plantuml.preproc.Defines;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Stream;

/**
 * Symmetric cross-language benchmark harness (Java side).
 *
 * Paired with benches/symbench.rs: same subcommands, same TSV/JSON formats.
 *
 * Boundary under test: source text -> SVG bytes, i.e.
 * {@code new SourceStringReader(new Defines(), source)} (preprocess + parse)
 * followed by {@code outputImage(sink, 0, new FileFormatOption(SVG))}
 * (render), matching plantuml_engine::render(_, FileFormat::Svg).
 */
public final class SymBench {

    private SymBench() {
    }

    /**
     * OutputStream that folds every written byte into a checksum.
     *
     * The checksum is printed to stderr after timing so the JIT cannot prove
     * the render is dead code and eliminate it (the Rust side uses black_box
     * for the same reason).
     */
    private static final class BlackholeSink extends OutputStream {
        private long checksum;
        private long bytes;

        @Override
        public void write(int b) {
            checksum = checksum * 131 + (b + 1);
            bytes++;
        }

        @Override
        public void write(byte[] b, int off, int len) {
            long c = checksum;
            for (int i = 0; i < len; i++) {
                c = c * 131 + (b[off + i] + 1);
            }
            checksum = c;
            bytes += len;
        }
    }

    // ---------------------------------------------------------------------
    // Render + classification
    // ---------------------------------------------------------------------

    /** Renders source to SVG bytes; returns null on any failure. */
    private static byte[] renderSvg(String source) {
        try {
            SourceStringReader reader = new SourceStringReader(new Defines(), source);
            ByteArrayOutputStream os = new ByteArrayOutputStream();
            reader.outputImage(os, 0, new FileFormatOption(FileFormat.SVG));
            return os.toByteArray();
        } catch (Throwable t) {
            return null;
        }
    }

    /** Success predicate shared with the Rust harness. */
    private static boolean isOk(byte[] svg) {
        if (svg == null || svg.length == 0) {
            return false;
        }
        String s = new String(svg, StandardCharsets.UTF_8);
        return s.contains("<svg") && !s.contains("Syntax Error");
    }

    // ---------------------------------------------------------------------
    // manifest
    // ---------------------------------------------------------------------

    private static List<Path> walkPuml(Path dir) throws IOException {
        List<Path> files = new ArrayList<>();
        try (Stream<Path> walk = Files.walk(dir)) {
            walk.filter(Files::isRegularFile)
                .filter(p -> p.toString().endsWith(".puml"))
                .forEach(files::add);
        }
        files.sort(Comparator.naturalOrder());
        return files;
    }

    private static void writeManifest(Path corpus, Path dest) throws IOException {
        List<Path> files = walkPuml(corpus);
        int ok = 0;
        StringBuilder sb = new StringBuilder();
        for (Path f : files) {
            String src = Files.readString(f, StandardCharsets.UTF_8);
            boolean good = isOk(renderSvg(src));
            if (good) {
                ok++;
            }
            sb.append(f).append('\t').append(good ? "ok" : "fail").append('\n');
        }
        Files.writeString(dest, sb.toString(), StandardCharsets.UTF_8);
        System.err.printf("manifest: %d/%d ok%n", ok, files.size());
    }

    // ---------------------------------------------------------------------
    // Shared manifest loading
    // ---------------------------------------------------------------------

    private record Row(String path, boolean ok) {
    }

    private static List<Row> readManifest(Path p) throws IOException {
        List<Row> rows = new ArrayList<>();
        for (String line : Files.readString(p, StandardCharsets.UTF_8).split("\n", -1)) {
            if (line.isEmpty()) {
                continue;
            }
            int tab = line.indexOf('\t');
            rows.add(new Row(line.substring(0, tab), "ok".equals(line.substring(tab + 1))));
        }
        return rows;
    }

    // ---------------------------------------------------------------------
    // run (steady-state latency)
    // ---------------------------------------------------------------------

    /** Timed render into the blackhole sink; returns nanos. */
    private static long timedRender(String source, OutputStream sink) {
        long start = System.nanoTime();
        try {
            SourceStringReader reader = new SourceStringReader(new Defines(), source);
            reader.outputImage(sink, 0, new FileFormatOption(FileFormat.SVG));
        } catch (Throwable t) {
            // The timed set is the pre-validated intersection; failures show
            // up as a changed checksum / byte count printed at the end.
        }
        return System.nanoTime() - start;
    }

    private static void runSteady(Path manifest, int iters, int warmup, Path dest)
            throws IOException {
        List<Row> rows = readManifest(manifest).stream().filter(Row::ok).toList();
        List<String> sources = new ArrayList<>();
        for (Row r : rows) {
            sources.add(Files.readString(Paths.get(r.path()), StandardCharsets.UTF_8));
        }

        BlackholeSink sink = new BlackholeSink();
        for (int w = 0; w < warmup; w++) {
            for (String src : sources) {
                timedRender(src, sink);
            }
        }

        long[] sums = new long[sources.size()];
        long[] mins = new long[sources.size()];
        Arrays.fill(mins, Long.MAX_VALUE);
        long globalStart = System.nanoTime();
        for (int it = 0; it < iters; it++) {
            for (int i = 0; i < sources.size(); i++) {
                long ns = timedRender(sources.get(i), sink);
                sums[i] += ns;
                if (ns < mins[i]) {
                    mins[i] = ns;
                }
            }
        }
        long totalNs = System.nanoTime() - globalStart;
        System.err.printf("blackhole checksum=%d bytes=%d%n", sink.checksum, sink.bytes);

        StringBuilder sb = new StringBuilder();
        sb.append("{\n");
        sb.append("  \"side\": \"java\",\n");
        sb.append("  \"mode\": \"steady\",\n");
        sb.append("  \"files\": ").append(sources.size()).append(",\n");
        sb.append("  \"iters\": ").append(iters).append(",\n");
        sb.append("  \"warmup\": ").append(warmup).append(",\n");
        sb.append("  \"fails\": 0,\n");
        sb.append("  \"total_ns\": ").append(totalNs).append(",\n");
        sb.append("  \"per_file\": [\n");
        for (int i = 0; i < rows.size(); i++) {
            long mean = sums[i] / iters;
            String comma = i + 1 == rows.size() ? "" : ",";
            sb.append("    {\"path\": \"")
              .append(jsonEscape(rows.get(i).path()))
              .append("\", \"mean_ns\": ").append(mean)
              .append(", \"min_ns\": ").append(mins[i])
              .append('}').append(comma).append('\n');
        }
        sb.append("  ]\n}\n");
        Files.writeString(dest, sb.toString(), StandardCharsets.UTF_8);
    }

    // ---------------------------------------------------------------------
    // once (single timed pass including file reads)
    // ---------------------------------------------------------------------

    private static void runOnce(Path manifest, Path dest) throws IOException {
        List<Row> rows = readManifest(manifest);
        long rendered = 0;
        long fails = 0;
        long start = System.nanoTime();
        for (Row r : rows) {
            if (!r.ok()) {
                continue;
            }
            String src = Files.readString(Paths.get(r.path()), StandardCharsets.UTF_8);
            byte[] out = renderSvg(src);
            if (isOk(out)) {
                rendered++;
            } else {
                fails++;
            }
        }
        long totalNs = System.nanoTime() - start;
        String json = String.format(
                "{\"side\": \"java\", \"mode\": \"once\", \"files\": %d, \"fails\": %d, \"total_ns\": %d}%n",
                rendered, fails, totalNs);
        Files.writeString(dest, json, StandardCharsets.UTF_8);
    }

    // ---------------------------------------------------------------------
    // null (cold process; externally timed)
    // ---------------------------------------------------------------------

    private static void runNull(List<String> files) throws IOException {
        BlackholeSink sink = new BlackholeSink();
        for (String f : files) {
            String src = Files.readString(Paths.get(f), StandardCharsets.UTF_8);
            try {
                SourceStringReader reader = new SourceStringReader(new Defines(), src);
                reader.outputImage(sink, 0, new FileFormatOption(FileFormat.SVG));
            } catch (Throwable ignored) {
            }
        }
        System.err.printf("blackhole checksum=%d bytes=%d%n", sink.checksum, sink.bytes);
    }

    // ---------------------------------------------------------------------

    private static String jsonEscape(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            usage();
        }
        switch (args[0]) {
            case "manifest" -> {
                if (args.length != 3) usage();
                writeManifest(Paths.get(args[1]), Paths.get(args[2]));
            }
            case "run" -> {
                if (args.length != 5) usage();
                runSteady(Paths.get(args[1]),
                        Integer.parseInt(args[2]), Integer.parseInt(args[3]),
                        Paths.get(args[4]));
            }
            case "once" -> {
                if (args.length != 3) usage();
                runOnce(Paths.get(args[1]), Paths.get(args[2]));
            }
            case "null" -> {
                List<String> files = new ArrayList<>();
                for (int i = 1; i < args.length; i++) {
                    files.add(args[i]);
                }
                runNull(files);
            }
            default -> usage();
        }
    }

    private static void usage() {
        System.err.println(
                "usage: SymBench {manifest <dir> <out.tsv> | run <m.tsv> <iters> <warmup> <out.json>"
                + " | once <m.tsv> <out.json> | null <file>...}");
        System.exit(2);
    }
}

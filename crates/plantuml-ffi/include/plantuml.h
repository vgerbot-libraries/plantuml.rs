#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Renders PlantUML source to SVG.
 *
 * Returns `0` on success, storing the SVG string in `*out` (newly allocated;
 * caller must free with [`plantuml_free_string`]). Returns `-1` on parse
 * error or null input, `-2` on invalid UTF-8 input, `-3` if the output
 * contains a NUL byte.
 */
int plantuml_render_svg(const char *source, char **out);

/**
 * Renders PlantUML source to PREPROC text.
 *
 * Same contract as [`plantuml_render_svg`].
 */
int plantuml_render_preproc(const char *source, char **out);

/**
 * Frees a string previously returned by [`plantuml_render_svg`] or
 * [`plantuml_render_preproc`]. Passing `null` is a no-op.
 *
 * # Safety
 *
 * `ptr` must be a pointer previously returned by [`plantuml_render_svg`] or
 * [`plantuml_render_preproc`], or null. The caller must not have already
 * freed it.
 */
void plantuml_free_string(char *ptr);

//! C FFI shared library for language bindings.
//!
//! Exposes stateless C ABI functions that render `PlantUML` source to SVG or
//! PREPROC text. Each call creates a fresh engine instance, renders, and
//! returns a newly allocated C string that the caller must free with
//! [`plantuml_free_string`].
//!
//! Ported from: plantuml-ffi (new, no Java source).

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Renders `PlantUML` source to SVG.
///
/// Returns `0` on success, storing the SVG string in `*out` (newly allocated;
/// caller must free with [`plantuml_free_string`]). Returns `-1` on parse
/// error or null input, `-2` on invalid UTF-8 input, `-3` if the output
/// contains a NUL byte.
#[no_mangle]
pub extern "C" fn plantuml_render_svg(source: *const c_char, out: *mut *mut c_char) -> c_int {
    render_to_c(source, out, plantuml_engine::render_svg)
}

/// Renders `PlantUML` source to PREPROC text.
///
/// Same contract as [`plantuml_render_svg`].
#[no_mangle]
pub extern "C" fn plantuml_render_preproc(source: *const c_char, out: *mut *mut c_char) -> c_int {
    render_to_c(source, out, plantuml_engine::render_preproc)
}

/// Frees a string previously returned by [`plantuml_render_svg`] or
/// [`plantuml_render_preproc`]. Passing `null` is a no-op.
///
/// # Safety
///
/// `ptr` must be a pointer previously returned by [`plantuml_render_svg`] or
/// [`plantuml_render_preproc`], or null. The caller must not have already
/// freed it.
#[no_mangle]
pub unsafe extern "C" fn plantuml_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: `ptr` was produced by `CString::into_raw` in `render_to_c`
        // and has not been freed yet. Reconstructing and dropping is safe.
        drop(CString::from_raw(ptr));
    }
}

fn render_to_c(
    source: *const c_char,
    out: *mut *mut c_char,
    f: fn(&str) -> Result<String, plantuml_engine::RenderError>,
) -> c_int {
    if source.is_null() || out.is_null() {
        return -1;
    }
    // SAFETY: caller guarantees `source` is a valid NUL-terminated C string.
    let source = unsafe { CStr::from_ptr(source) };
    let Ok(source) = source.to_str() else {
        return -2;
    };
    f(source).map_or(-1, |s| {
        CString::new(s).map_or(-3, |cs| {
            // SAFETY: `out` is non-null (checked above) and points to a
            // writable `*mut c_char` owned by the caller.
            unsafe {
                *out = cs.into_raw();
            }
            0
        })
    })
}

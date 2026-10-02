//! Exact AWT text metrics not provided by `StringBounderSvg`.
//!
//! The shared `StringBounderSvg` ignores font weight, so bold runs fall
//! back to plain widths. The Gantt month header is bold size 12; this
//! module reproduces the bold `FontMetrics.getStringBounds()` widths at
//! the jar's fractional-metrics `FontRenderContext`.
//!
//! Port reference:
//! `net/sourceforge/plantuml/klimt/drawing/svg/DriverTextSvg.java` and
//! `gantt/draw/header/TimeHeaderDaily.java`.

/// Per-character bold SansSerif size-12 advance widths for ASCII
/// 0x20–0x7E, measured from Java AWT `getStringBounds` with the jar's
/// exact fractional `FontRenderContext`.
#[allow(clippy::unreadable_literal)]
const AWT_BOLD_12_PLAIN: [f64; 95] = [
    3.12002563, 3.43202209, 5.66404724, 7.75205994, 6.86404419, 10.81207275, 9.00006104, 3.19201660, 4.06802368, 4.06802368,
    6.54003906, 6.86404419, 3.42002869, 3.86402893, 3.42002869, 4.95603943, 6.86404419, 6.86404419, 6.86404419, 6.86404419,
    6.86404419, 6.86404419, 6.86404419, 6.86404419, 6.86404419, 6.86404419, 3.42002869, 3.42002869, 6.86404419, 6.86404419,
    6.86404419, 5.72404480, 10.76406860, 8.28005981, 8.06405640, 7.64405823, 8.88006592, 6.72004700, 6.58804321, 8.68806458,
    9.18006897, 4.66802979, 3.97203064, 7.96806335, 6.78004456, 11.31608582, 9.75607300, 9.55206299, 7.53605652, 9.55206299,
    7.92005920, 6.61204529, 6.94804382, 9.07206726, 7.80004883, 11.60408020, 8.00405884, 7.48805237, 6.94804382, 3.97203064,
    4.95603943, 3.97203064, 6.86404419, 4.93203735, 4.34402466, 7.24804688, 7.59605408, 6.16804504, 7.59605408, 7.09205627,
    4.64402771, 7.59605408, 7.88404846, 3.66001892, 3.66001892, 7.44004822, 3.66001892, 11.78408813, 7.88404846, 7.42805481,
    7.59605408, 7.59605408, 5.44804382, 5.96403503, 5.20803833, 7.88404846, 6.82804871, 10.27207947, 6.93605042, 6.82804871,
    5.85604858, 4.72802734, 6.61204529, 4.72802734, 6.86404419,
];

/// Returns the exact bold size-12 advance width of ASCII `text`.
///
/// For text containing non-ASCII code points returns `None`; callers then
/// fall back to the plain bounder. The per-character widths are exact
/// advances, so their sum equals AWT's whole-string width.
#[must_use]
pub fn bold12_width(text: &str) -> Option<f64> {
    let mut width = 0.0_f64;
    for cp in text.chars().map(|c| c as u32) {
        if !(0x20..=0x7e).contains(&cp) {
            return None;
        }
        width += AWT_BOLD_12_PLAIN[(cp - 0x20) as usize];
    }
    Some(width)
}

//! WASM-safe wall-clock helpers.
//!
//! `std::time::SystemTime::now()` panics on `wasm32-unknown-unknown`
//! ("time not implemented on this platform"), which crashes every render
//! because preprocessing always evaluates the `%date` builtin. On WASM we
//! fall back to a fixed epoch (0) — the affected builtins (`%date`, `%now`,
//! `%random` seed) are cosmetic for diagram rendering and a zero timestamp
//! is acceptable. On native targets the real clock is used.

/// Returns the current Unix time in seconds, or `0` on WASM.
#[must_use]
pub fn now_secs() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

/// Returns the current Unix time in nanoseconds, or `1` on WASM (used as a
/// PRNG seed; must be non-zero).
#[must_use]
pub fn now_nanos() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    }
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
}

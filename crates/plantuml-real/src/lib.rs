#![allow(clippy::suboptimal_flops, clippy::imprecise_flops, clippy::manual_midpoint)]
//! Real constraint system — 1D constraint solver for layout positioning.
//!
//! Ported from: `net/sourceforge/plantuml/real/` package (13 files)
//!
//! The Real system models positions as constraint variables connected by
//! positive forces. `RealLine::compile()` iterates all forces until a fixed
//! point is reached, ensuring all minimum-distance constraints are satisfied.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

// ── RealLine ──────────────────────────────────────────────────────────────

/// The constraint solver that holds all forces and iterates until fixed point.
///
/// Ported from: `net/sourceforge/plantuml/real/RealLine.java`
#[derive(Debug)]
pub struct RealLine {
    forces: Vec<PositiveForce>,
    min: f64,
    max: f64,
}

impl RealLine {
    /// Creates a new empty constraint line.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            forces: Vec::new(),
            min: 0.0,
            max: 0.0,
        }
    }

    fn add_force(&mut self, force: PositiveForce) {
        self.forces.push(force);
    }

    /// Iterates all forces until no more changes occur (fixed point).
    ///
    /// Ported from: `RealLine.compile()`.
    pub fn compile(&mut self) {
        let mut cpt = 0;
        loop {
            let mut done = true;
            for f in &self.forces {
                if f.apply() {
                    done = false;
                }
            }
            if done {
                self.min = 0.0;
                self.max = 0.0;
                return;
            }
            cpt += 1;
            assert!(cpt <= 99_999, "Infinite Loop in RealLine::compile?");
        }
    }

    /// Clears all forces, breaking reference cycles so `Rc` objects can be freed.
    pub fn clear_forces(&mut self) {
        self.forces.clear();
    }

    #[must_use]
    pub const fn get_absolute_min(&self) -> f64 {
        self.min
    }

    #[must_use]
    pub const fn get_absolute_max(&self) -> f64 {
        self.max
    }
}

impl Default for RealLine {
    fn default() -> Self {
        Self::new()
    }
}

// ── PositiveForce ─────────────────────────────────────────────────────────

/// A constraint ensuring `moving_point >= fixed_point + min_distance`.
///
/// Ported from: `net/sourceforge/plantuml/real/PositiveForce.java`
#[derive(Debug)]
struct PositiveForce {
    fixed_point: Rc<dyn Real>,
    moving_point: Rc<dyn Real>,
    min_distance: f64,
}

impl PositiveForce {
    fn new(fixed_point: Rc<dyn Real>, moving_point: Rc<dyn Real>, min_distance: f64) -> Self {
        Self {
            fixed_point,
            moving_point,
            min_distance,
        }
    }

    /// Applies the force: if `moving < fixed + min_distance`, moves `moving` up.
    /// Returns `true` if a change was made.
    fn apply(&self) -> bool {
        let moving_val = self.moving_point.get_current_value();
        let fixed_val = self.fixed_point.get_current_value();
        let distance = moving_val - fixed_val;
        let diff = distance - self.min_distance;
        let epsilon = 0.000_001_f64.max(
            1000.0 * (moving_val.abs().max(fixed_val.abs())).ulp().unwrap_or(0.0),
        );
        if diff >= -epsilon {
            return false;
        }
        self.moving_point.move_value(-diff);
        true
    }
}

// ── Real trait ────────────────────────────────────────────────────────────

/// A 1D position variable in the constraint system.
///
/// Ported from: `net/sourceforge/plantuml/real/Real.java`
pub trait Real: std::fmt::Debug {
    /// Returns the current resolved value.
    fn get_current_value(&self) -> f64;

    /// Returns the name (for debugging).
    fn get_name(&self) -> &str;

    /// Moves this real by `delta`. Only effective for movable reals (`RealImpl`).
    ///
    /// Ported from: `RealMoveable.move(double)`.
    fn move_value(&self, _delta: f64) {}

    /// Returns the `RealLine` this real belongs to.
    fn get_line(&self) -> &Rc<RefCell<RealLine>>;

    /// If this is a `RealDelta`, returns the delegated real and the diff.
    /// Used by `ensure_bigger_than` to mirror Java's
    /// `RealDelta.ensureBiggerThan()` delegation, which preserves f64
    /// cancellation: constraining `delegated + diff >= other` as
    /// `delegated >= other - diff` avoids accumulating separate rounding
    /// errors on both sides of the inequality.
    fn as_delta(&self) -> Option<(&Rc<dyn Real>, f64)> {
        None
    }
}

// ── RealImpl ──────────────────────────────────────────────────────────────

/// A mutable real value — the only type that can be directly moved by forces.
///
/// Ported from: `net/sourceforge/plantuml/real/RealImpl.java`
#[derive(Debug)]
pub struct RealImpl {
    line: Rc<RefCell<RealLine>>,
    name: String,
    current_value: Cell<f64>,
}

impl RealImpl {
    /// Creates a new mutable real at `current_value`.
    ///
    /// Ported from: `RealImpl(String, RealLine, double)`.
    #[must_use]
    pub fn new(name: impl Into<String>, line: Rc<RefCell<RealLine>>, current_value: f64) -> Self {
        Self {
            line,
            name: name.into(),
            current_value: Cell::new(current_value),
        }
    }
}

impl Real for RealImpl {
    fn get_current_value(&self) -> f64 {
        self.current_value.get()
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn move_value(&self, delta: f64) {
        self.current_value.set(self.current_value.get() + delta);
    }

    fn get_line(&self) -> &Rc<RefCell<RealLine>> {
        &self.line
    }
}

// ── RealDelta ─────────────────────────────────────────────────────────────

/// A real that is always `delegated + diff`.
///
/// Ported from: `net/sourceforge/plantuml/real/RealDelta.java`
#[derive(Debug)]
pub struct RealDelta {
    line: Rc<RefCell<RealLine>>,
    delegated: Rc<dyn Real>,
    diff: f64,
}

impl RealDelta {
    #[must_use]
    pub fn new(delegated: Rc<dyn Real>, diff: f64) -> Self {
        let line = delegated.get_line().clone();
        Self {
            line,
            delegated,
            diff,
        }
    }
}

impl Real for RealDelta {
    fn get_current_value(&self) -> f64 {
        self.delegated.get_current_value() + self.diff
    }

    fn get_name(&self) -> &'static str {
        "[Delegated]"
    }

    fn move_value(&self, delta: f64) {
        self.delegated.move_value(delta);
    }

    fn get_line(&self) -> &Rc<RefCell<RealLine>> {
        &self.line
    }

    fn as_delta(&self) -> Option<(&Rc<dyn Real>, f64)> {
        Some((&self.delegated, self.diff))
    }
}

// ── RealDeltaLive ─────────────────────────────────────────────────────────

/// A real that is always `delegated + delta()` where `delta()` is a live closure.
///
/// Ported from: `net/sourceforge/plantuml/real/RealDeltaLive.java`
pub struct RealDeltaLive {
    line: Rc<RefCell<RealLine>>,
    delegated: Rc<dyn Real>,
    delta: Box<dyn Fn() -> f64>,
}

impl std::fmt::Debug for RealDeltaLive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealDeltaLive")
            .field("delegated", &self.delegated)
            .finish_non_exhaustive()
    }
}

impl RealDeltaLive {
    #[must_use]
    pub fn new(delegated: Rc<dyn Real>, delta: Box<dyn Fn() -> f64>) -> Self {
        let line = delegated.get_line().clone();
        Self {
            line,
            delegated,
            delta,
        }
    }
}

impl Real for RealDeltaLive {
    fn get_current_value(&self) -> f64 {
        self.delegated.get_current_value() + (self.delta)()
    }

    fn get_name(&self) -> &'static str {
        "[DelegatedLive]"
    }

    fn move_value(&self, delta: f64) {
        self.delegated.move_value(delta);
    }

    fn get_line(&self) -> &Rc<RefCell<RealLine>> {
        &self.line
    }
}

// ── RealMin ──────────────────────────────────────────────────────────────

/// The minimum of multiple reals.
///
/// Ported from: `net/sourceforge/plantuml/real/RealMin.java`
#[derive(Debug)]
pub struct RealMin {
    line: Rc<RefCell<RealLine>>,
    all: Vec<Rc<dyn Real>>,
}

impl RealMin {
    #[must_use]
    pub fn new(reals: Vec<Rc<dyn Real>>) -> Self {
        let line = reals[0].get_line().clone();
        Self { line, all: reals }
    }
}

impl Real for RealMin {
    fn get_current_value(&self) -> f64 {
        self.all
            .iter()
            .map(|r| r.get_current_value())
            .fold(f64::MAX, f64::min)
    }

    fn get_name(&self) -> &'static str {
        "min"
    }

    fn get_line(&self) -> &Rc<RefCell<RealLine>> {
        &self.line
    }
}

// ── RealMax ──────────────────────────────────────────────────────────────

/// The maximum of multiple reals.
///
/// Ported from: `net/sourceforge/plantuml/real/RealMax.java`
#[derive(Debug)]
pub struct RealMax {
    line: Rc<RefCell<RealLine>>,
    all: Vec<Rc<dyn Real>>,
}

impl RealMax {
    #[must_use]
    pub fn new(reals: Vec<Rc<dyn Real>>) -> Self {
        let line = reals[0].get_line().clone();
        Self { line, all: reals }
    }
}

impl Real for RealMax {
    fn get_current_value(&self) -> f64 {
        self.all
            .iter()
            .map(|r| r.get_current_value())
            .fold(f64::MIN, f64::max)
    }

    fn get_name(&self) -> &'static str {
        "max"
    }

    fn get_line(&self) -> &Rc<RefCell<RealLine>> {
        &self.line
    }
}

// ── RealMiddle2 ───────────────────────────────────────────────────────────

/// The midpoint of two movable reals.
///
/// Ported from: `net/sourceforge/plantuml/real/RealMiddle2.java`
#[derive(Debug)]
pub struct RealMiddle2 {
    line: Rc<RefCell<RealLine>>,
    p1: Rc<dyn Real>,
    p2: Rc<dyn Real>,
}

impl RealMiddle2 {
    #[must_use]
    pub fn new(p1: Rc<dyn Real>, p2: Rc<dyn Real>) -> Self {
        let line = p1.get_line().clone();
        Self { line, p1, p2 }
    }
}

impl Real for RealMiddle2 {
    fn get_current_value(&self) -> f64 {
        (self.p1.get_current_value() + self.p2.get_current_value()) / 2.0
    }

    fn get_name(&self) -> &'static str {
        "middle"
    }

    fn move_value(&self, delta: f64) {
        self.p1.move_value(delta / 2.0);
        self.p2.move_value(delta / 2.0);
    }

    fn get_line(&self) -> &Rc<RefCell<RealLine>> {
        &self.line
    }
}

// ── Free functions (RealUtils) ─────────────────────────────────────────────

/// Creates a new origin real at position 0, with its own constraint line.
///
/// Ported from: `RealUtils.createOrigin()`.
#[must_use]
pub fn create_origin() -> Rc<dyn Real> {
    let line = Rc::new(RefCell::new(RealLine::new()));
    Rc::new(RealImpl::new("O", line, 0.0))
}

/// Creates a real at `real + delta` (fixed offset, no constraint).
///
/// Ported from: `RealMoveable.addFixed(double)`.
#[must_use]
pub fn add_fixed(real: &Rc<dyn Real>, delta: f64) -> Rc<dyn Real> {
    Rc::new(RealDelta::new(real.clone(), delta))
}

/// Creates a mutable real at `real + delta`, with a force ensuring it stays
/// at least `delta` ahead of `real`.
///
/// Ported from: `RealImpl.addAtLeast(double)`.
#[must_use]
pub fn add_at_least(real: &Rc<dyn Real>, delta: f64) -> Rc<dyn Real> {
    let line = real.get_line().clone();
    let current = real.get_current_value();
    let result = Rc::new(RealImpl::new(
        format!("addAtLeast{delta}"),
        line.clone(),
        current + delta,
    ));
    line.borrow_mut()
        .add_force(PositiveForce::new(real.clone(), result.clone(), delta));
    result
}

/// Adds a constraint ensuring `real >= other + 0`.
///
/// Ported from: `RealImpl.ensureBiggerThan(Real)`.
pub fn ensure_bigger_than(real: &Rc<dyn Real>, other: &Rc<dyn Real>) {
    // Mirror Java's RealDelta.ensureBiggerThan(): when `real` is a RealDelta
    // (delegated + diff), delegate to `delegated.ensureBiggerThan(other - diff)`.
    // This preserves f64 cancellation: constraining `delegated + diff >= other`
    // as `delegated >= other - diff` avoids accumulating separate rounding
    // errors on both sides of the inequality, which would otherwise shift
    // the solver's fixed point by a few ULPs and flip the last digit of
    // formatted SVG coordinates.
    if let Some((delegated, diff)) = real.as_delta() {
        let adjusted_other = add_fixed(other, -diff);
        ensure_bigger_than(delegated, &adjusted_other);
        return;
    }
    let line = real.get_line().clone();
    line.borrow_mut()
        .add_force(PositiveForce::new(other.clone(), real.clone(), 0.0));
}

/// Creates a real at the midpoint of two reals.
///
/// Ported from: `RealUtils.middle(Real, Real)`.
#[must_use]
pub fn middle(r1: &Rc<dyn Real>, r2: &Rc<dyn Real>) -> Rc<dyn Real> {
    Rc::new(RealMiddle2::new(r1.clone(), r2.clone()))
}

/// Creates a real at the maximum of multiple reals.
///
/// Ported from: `RealUtils.max(Real...)`.
#[must_use]
pub fn max(reals: &[Rc<dyn Real>]) -> Rc<dyn Real> {
    Rc::new(RealMax::new(reals.to_vec()))
}

/// Creates a real at the minimum of multiple reals.
///
/// Ported from: `RealUtils.min(Real...)`.
#[must_use]
pub fn min(reals: &[Rc<dyn Real>]) -> Rc<dyn Real> {
    Rc::new(RealMin::new(reals.to_vec()))
}

/// Creates a real with a live offset (closure re-evaluated each time).
///
/// Ported from: `RealUtils.withLiveOffset(Real, DoubleSupplier)`.
#[must_use]
pub fn with_live_offset(real: &Rc<dyn Real>, delta: Box<dyn Fn() -> f64>) -> Rc<dyn Real> {
    Rc::new(RealDeltaLive::new(real.clone(), delta))
}

/// Compiles all constraints on the given line.
///
/// Ported from: `RealOrigin.compileNow()`.
pub fn compile_now(line: &Rc<RefCell<RealLine>>) {
    line.borrow_mut().compile();
}

/// Clears all forces on the given line, breaking reference cycles.
pub fn clear_forces(line: &Rc<RefCell<RealLine>>) {
    line.borrow_mut().clear_forces();
}

// ── ULP helper ────────────────────────────────────────────────────────────

/// Extension trait for `f64` ULP (unit in the last place) computation.
trait F64Ulp {
    fn ulp(&self) -> Option<f64>;
}

impl F64Ulp for f64 {
    fn ulp(&self) -> Option<f64> {
        if self.is_nan() || self.is_infinite() {
            return None;
        }
        // Math.ulp(x) in Java: the distance to the next representable value.
        let bits = self.to_bits();
        let next = Self::from_bits(bits + 1);
        Some((next - self).abs())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_starts_at_zero() {
        let origin = create_origin();
        assert!((origin.get_current_value()).abs() < 1e-10);
    }

    #[test]
    fn test_add_fixed() {
        let origin = create_origin();
        let pos = add_fixed(&origin, 10.0);
        assert!((pos.get_current_value() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_add_at_least_satisfied() {
        let origin = create_origin();
        let pos = add_fixed(&origin, 10.0);
        let next = add_at_least(&pos, 10.0);
        compile_now(origin.get_line());
        assert!((next.get_current_value() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_add_at_least_needs_move() {
        let origin = create_origin();
        let pos = add_fixed(&origin, 10.0);
        // Create next at origin + 5 (too close)
        let line = origin.get_line().clone();
        let next = Rc::new(RealImpl::new("next", line.clone(), 5.0));
        line.borrow_mut()
            .add_force(PositiveForce::new(pos, next.clone(), 10.0));
        compile_now(&line);
        assert!((next.get_current_value() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_middle() {
        let origin = create_origin();
        let p1 = add_fixed(&origin, 10.0);
        let p2 = add_fixed(&origin, 20.0);
        let mid = middle(&p1, &p2);
        assert!((mid.get_current_value() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_max() {
        let origin = create_origin();
        let p1 = add_fixed(&origin, 10.0);
        let p2 = add_fixed(&origin, 30.0);
        let p3 = add_fixed(&origin, 20.0);
        let m = max(&[p1, p2, p3]);
        assert!((m.get_current_value() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_min() {
        let origin = create_origin();
        let p1 = add_fixed(&origin, 10.0);
        let p2 = add_fixed(&origin, 30.0);
        let p3 = add_fixed(&origin, 20.0);
        let m = min(&[p1, p2, p3]);
        assert!((m.get_current_value() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_ensure_bigger_than() {
        let origin = create_origin();
        let p1 = add_fixed(&origin, 10.0);
        let line = origin.get_line().clone();
        let p2: Rc<dyn Real> = Rc::new(RealImpl::new("p2", line.clone(), 5.0));
        ensure_bigger_than(&p2, &p1);
        compile_now(&line);
        assert!((p2.get_current_value() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_chain_of_add_fixed() {
        let origin = create_origin();
        let a = add_fixed(&origin, 10.0);
        let b = add_fixed(&a, 44.45);
        let c = add_at_least(&b, 10.0);
        compile_now(origin.get_line());
        assert!((a.get_current_value() - 10.0).abs() < 1e-10);
        assert!((b.get_current_value() - 54.45).abs() < 1e-10);
        assert!((c.get_current_value() - 64.45).abs() < 1e-10);
    }
}

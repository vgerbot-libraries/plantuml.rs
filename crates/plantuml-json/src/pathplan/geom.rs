//! Point/edge geometry and the cubic root solver.
//!
//! Ported from graphviz 14.1.2:
//! - `lib/pathplan/pathgeom.h` (`Ppoint_t`, `Pedge_t`)
//! - `lib/pathplan/solvers.c` (`solve3`, `solve2`, `solve1`)

#![allow(clippy::similar_names)]

/// Ported from: lib/pathplan/pathgeom.h (`Ppoint_t`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }
    pub fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }
    pub fn scale(self, c: f64) -> Self {
        Self::new(self.x * c, self.y * c)
    }
}

pub fn dot(a: Point, b: Point) -> f64 {
    a.x * b.x + a.y * b.y
}

pub fn dist(a: Point, b: Point) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

/// Bit-exact float equality, mirroring C's `==` on `double` coordinates.
pub fn feq(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

/// Ported from: lib/pathplan/pathgeom.h (`Pedge_t`).
#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub a: Point,
    pub b: Point,
}

// ── Orientation ─────────────────────────────────────────────────────────

pub const ISCW: i32 = 1;
pub const ISCCW: i32 = 2;
pub const ISON: i32 = 3;

/// Ported from: lib/pathplan/triang.c (`ccw`).
pub fn ccw(p1: Point, p2: Point, p3: Point) -> i32 {
    let d = (p1.y - p2.y) * (p3.x - p2.x) - (p3.y - p2.y) * (p1.x - p2.x);
    if d > 0.0 {
        ISCW
    } else if d < 0.0 {
        ISCCW
    } else {
        ISON
    }
}

// ── Cubic solver ──────────────────────────────────────────────────────────

const POLY_EPS: f64 = 1e-7;

fn aeq0(x: f64) -> bool {
    x < POLY_EPS && x > -POLY_EPS
}

/// Ported from: lib/pathplan/solvers.c (`solve3`, `solve2`, `solve1`).
/// Returns roots in `roots`; `coeff` is constant..cubic order.
pub fn solve3(coeff: [f64; 4], roots: &mut [f64; 3]) -> usize {
    let (c3, c2, c1, c0) = (coeff[3], coeff[2], coeff[1], coeff[0]);
    if aeq0(c3) {
        return solve2(coeff, roots);
    }
    let shift = c2 / (3.0 * c3);
    let c1_over = c1 / c3;
    let c0_over = c0 / c3;

    let mut pp = shift * shift;
    let qq = 2.0 * shift * pp - shift * c1_over + c0_over;
    pp = c1_over / 3.0 - pp;
    let disc = qq * qq + 4.0 * pp * pp * pp;

    let rootn = if disc < 0.0 {
        let rad = 0.5 * (-disc + qq * qq).sqrt();
        let theta = (-disc).sqrt().atan2(-qq);
        let temp = 2.0 * rad.cbrt();
        roots[0] = temp * (theta / 3.0).cos();
        roots[1] = temp * ((theta + std::f64::consts::TAU) / 3.0).cos();
        roots[2] = temp * ((theta - std::f64::consts::TAU) / 3.0).cos();
        3
    } else {
        let alpha = 0.5 * (disc.sqrt() - qq);
        let beta = -qq - alpha;
        roots[0] = alpha.cbrt() + beta.cbrt();
        if disc > 0.0 {
            1
        } else {
            roots[1] = -0.5 * roots[0];
            roots[2] = roots[1];
            3
        }
    };

    for rr in roots.iter_mut().take(rootn) {
        *rr -= shift;
    }
    rootn
}

fn solve2(coeff: [f64; 4], roots: &mut [f64; 3]) -> usize {
    let (a, b, c) = (coeff[2], coeff[1], coeff[0]);
    if aeq0(a) {
        return solve1(coeff, roots);
    }
    let b_over_2a = b / (2.0 * a);
    let c_over_a = c / a;
    let disc = b_over_2a * b_over_2a - c_over_a;
    if disc < 0.0 {
        0
    } else if disc > 0.0 {
        roots[0] = -b_over_2a + disc.sqrt();
        roots[1] = -2.0 * b_over_2a - roots[0];
        2
    } else {
        roots[0] = -b_over_2a;
        1
    }
}

fn solve1(coeff: [f64; 4], roots: &mut [f64; 3]) -> usize {
    let (a, b) = (coeff[1], coeff[0]);
    if aeq0(a) {
        if aeq0(b) {
            // C returns 4 (identity polynomial); the intersection code skips
            // the edge in that case. Model with a sentinel.
            roots[0] = f64::NAN;
            4
        } else {
            0
        }
    } else {
        roots[0] = -b / a;
        1
    }
}

/// Wrapper adapting the 4-slot coefficient/root arrays; handles the identity
/// sentinel from `solve1`.
pub fn solve3_coeff(coeff: &[f64; 4], roots: &mut [f64; 3]) -> usize {
    let n = solve3(*coeff, roots);
    if n == 4 {
        return 4;
    }
    n
}

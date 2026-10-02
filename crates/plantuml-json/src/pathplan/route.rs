//! Cubic Bezier spline fitting to an input polyline.
//!
//! Ported from graphviz 14.1.2 `lib/pathplan/route.c` (`Proutespline`,
//! `reallyroutespline`, `mkspline`, `splinefits`, `splineisinside`,
//! `splineintersectsline`).

#![allow(clippy::similar_names)]

use super::geom::{dist, dot, feq, solve3_coeff, Edge, Point};

const EPSILON1: f64 = 1e-3;
const EPSILON2: f64 = 1e-6;

struct SplineOut {
    ops: Vec<Point>,
}

impl SplineOut {
    fn grow(&mut self, newn: usize) {
        if newn > self.ops.len() {
            self.ops.resize(newn, Point::new(0.0, 0.0));
        }
    }
}

struct Tna {
    t: f64,
    a: [Point; 2],
}

fn normv(mut v: Point) -> Point {
    let d = dot(v, v);
    if d > 1e-6 {
        let d = d.sqrt();
        v.x /= d;
        v.y /= d;
    }
    v
}

// Bernstein basis.
fn b0(t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u
}
fn b1(t: f64) -> f64 {
    3.0 * t * (1.0 - t) * (1.0 - t)
}
fn b2(t: f64) -> f64 {
   3.0 * t * t * (1.0 - t)
}
fn b3(t: f64) -> f64 {
    t * t * t
}
fn b01(t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * (u + 3.0 * t)
}
fn b23(t: f64) -> f64 {
    let u = 1.0 - t;
    t * t * (3.0 * u + t)
}

/// Fit a cubic Bezier spline to `input`, avoiding `barriers`, with endpoint
/// tangent directions `ev0`/`ev1` (unconstrained ends pass a zero vector).
pub fn route_spline(barriers: &[Edge], input: &[Point], ev0: Point, ev1: Point) -> Vec<Point> {
    if input.len() < 2 {
        return input.to_vec();
    }
    let mut out = SplineOut { ops: Vec::new() };
    out.grow(4);
    out.ops[0] = input[0];
    let mut opl = 1usize;
    really_route(barriers, input, ev0, ev1, &mut out, &mut opl);
    out.ops.truncate(opl);
    out.ops
}

fn really_route(
    edges: &[Edge],
    inps: &[Point],
    ev0: Point,
    ev1: Point,
    out: &mut SplineOut,
    opl: &mut usize,
) {
    let n = inps.len();
    let mut tnas: Vec<Tna> = (0..n)
        .map(|_| Tna { t: 0.0, a: [Point::new(0.0, 0.0), Point::new(0.0, 0.0)] })
        .collect();
    for i in 1..n {
        tnas[i].t = tnas[i - 1].t + dist(inps[i], inps[i - 1]);
    }
    for i in 1..n {
        tnas[i].t /= tnas[n - 1].t;
    }
    for tna in &mut tnas {
        tna.a[0] = ev0.scale(b1(tna.t));
        tna.a[1] = ev1.scale(b2(tna.t));
    }

    let ends = mk_spline(inps, &tnas, ev0, ev1);
    let fit = spline_fits(edges, &ends, inps, out, opl);
    if fit > 0 {
        return;
    }

    let cp1 = ends.p1.add(ends.v1.scale(1.0 / 3.0));
    let cp2 = ends.p2.sub(ends.v2.scale(1.0 / 3.0));
    let mut maxi = 0usize;
    let mut maxd = -1.0;
    for i in 1..n - 1 {
        let t = tnas[i].t;
        let p = Point::new(
            b0(t) * ends.p1.x + b1(t) * cp1.x + b2(t) * cp2.x + b3(t) * ends.p2.x,
            b0(t) * ends.p1.y + b1(t) * cp1.y + b2(t) * cp2.y + b3(t) * ends.p2.y,
        );
        let d = dist(p, inps[i]);
        if d > maxd {
            maxd = d;
            maxi = i;
        }
    }

    let split_v1 = normv(inps[maxi].sub(inps[maxi - 1]));
    let split_v2 = normv(inps[maxi + 1].sub(inps[maxi]));
    let split_v = normv(split_v1.add(split_v2));
    really_route(edges, &inps[..=maxi], ev0, split_v, out, opl);
    really_route(edges, &inps[maxi..], split_v, ev1, out, opl);
}

struct SplineEnds {
    p1: Point,
    v1: Point,
    p2: Point,
    v2: Point,
}

fn mk_spline(inps: &[Point], tnas: &[Tna], ev0: Point, ev1: Point) -> SplineEnds {
    let n = inps.len();
    let mut c = [[0.0f64; 2]; 2];
    let mut x = [0.0f64; 2];
    for i in 0..n {
        c[0][0] += dot(tnas[i].a[0], tnas[i].a[0]);
        c[0][1] += dot(tnas[i].a[0], tnas[i].a[1]);
        c[1][1] += dot(tnas[i].a[1], tnas[i].a[1]);
        let tmp = inps[i].sub(
            inps[0]
                .scale(b01(tnas[i].t))
                .add(inps[n - 1].scale(b23(tnas[i].t))),
        );
        x[0] += dot(tnas[i].a[0], tmp);
        x[1] += dot(tnas[i].a[1], tmp);
    }
    c[1][0] = c[0][1];
    let det = c[0][0] * c[1][1] - c[1][0] * c[0][1];
    let (mut scale0, mut scale3) = if det.abs() >= 1e-6 {
        (
            (x[0] * c[1][1] - x[1] * c[0][1]) / det,
            (c[0][0] * x[1] - c[0][1] * x[0]) / det,
        )
    } else {
        (0.0, 0.0)
    };
    if det.abs() < 1e-6 || scale0 <= 0.0 || scale3 <= 0.0 {
        let d01 = dist(inps[0], inps[n - 1]) / 3.0;
        scale0 = d01;
        scale3 = d01;
    }
    SplineEnds {
        p1: inps[0],
        v1: ev0.scale(scale0),
        p2: inps[n - 1],
        v2: ev1.scale(scale3),
    }
}

fn spline_fits(
    edges: &[Edge],
    ends: &SplineEnds,
    inps: &[Point],
    out: &mut SplineOut,
    opl: &mut usize,
) -> i32 {
    let SplineEnds { p1: pa, v1: va, p2: pb, v2: vb } = *ends;
    let force = inps.len() == 2;
    let mut a = 4.0f64;
    let mut first = true;
    loop {
        let sps = [pa, pa.add(va.scale(a / 3.0)), pb.sub(vb.scale(a / 3.0)), pb];

        if first && polyline_len(&sps) < polyline_len(inps) - EPSILON1 {
            return 0;
        }
        first = false;

        if spline_is_inside(edges, &sps) {
            out.grow(*opl + 4);
            for p in sps.iter().skip(1) {
                out.ops[*opl] = *p;
                *opl += 1;
            }
            return 1;
        }
        if a < 0.005 {
            if force {
                out.grow(*opl + 4);
                for p in sps.iter().skip(1) {
                    out.ops[*opl] = *p;
                    *opl += 1;
                }
                return 1;
            }
            return 0;
        }
        if a > 0.01 {
            a /= 2.0;
        } else {
            a = 0.0;
        }
    }
}

fn polyline_len(p: &[Point]) -> f64 {
    p.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn spline_is_inside(edges: &[Edge], sps: &[Point; 4]) -> bool {
    for e in edges {
        let lps = [e.a, e.b];
        let mut roots = [0.0f64; 3];
        let rootn = spline_intersects_line(sps, &lps, &mut roots);
        if rootn == 4 {
            continue;
        }
        for r in roots.iter().take(rootn) {
            if *r < EPSILON2 || *r > 1.0 - EPSILON2 {
                continue;
            }
            let t = *r;
            let td = t * t * t;
            let tc = 3.0 * t * t * (1.0 - t);
            let tb = 3.0 * t * (1.0 - t) * (1.0 - t);
            let ta = (1.0 - t).powi(3);
            let ip = Point::new(
                ta * sps[0].x + tb * sps[1].x + tc * sps[2].x + td * sps[3].x,
                ta * sps[0].y + tb * sps[1].y + tc * sps[2].y + td * sps[3].y,
            );
            let d0 = (ip.x - lps[0].x).powi(2) + (ip.y - lps[0].y).powi(2);
            let d1 = (ip.x - lps[1].x).powi(2) + (ip.y - lps[1].y).powi(2);
            if d0 < EPSILON1 || d1 < EPSILON1 {
                continue;
            }
            return false;
        }
    }
    true
}

fn spline_intersects_line(sps: &[Point; 4], lps: &[Point; 2], roots: &mut [f64; 3]) -> usize {
    let xcoeff = [lps[0].x, lps[1].x - lps[0].x];
    let ycoeff = [lps[0].y, lps[1].y - lps[0].y];
    let mut rootn = 0usize;
    let mut xr = [0.0f64; 3];
    let mut yr = [0.0f64; 3];
    let mut cx4 = [0.0f64; 4];
    let mut cy4 = [0.0f64; 4];

    macro_rules! add_root {
        ($r:expr) => {{
            let r = $r;
            if (0.0..=1.0).contains(&r) {
                roots[rootn] = r;
                rootn += 1;
            }
        }};
    }

    if xcoeff[1] == 0.0 {
        if ycoeff[1] == 0.0 {
            points2coeff([sps[0].x, sps[1].x, sps[2].x, sps[3].x], &mut cx4);
            cx4[0] -= xcoeff[0];
            let xrn = solve3_coeff(&cx4, &mut xr);
            points2coeff([sps[0].y, sps[1].y, sps[2].y, sps[3].y], &mut cy4);
            cy4[0] -= ycoeff[0];
            let yrn = solve3_coeff(&cy4, &mut yr);
            if xrn == 4 {
                if yrn == 4 {
                    return 4;
                }
                for &yj in yr.iter().take(yrn) {
                    add_root!(yj);
                }
            } else if yrn == 4 {
                for &xi in xr.iter().take(xrn) {
                    add_root!(xi);
                }
            } else {
                for &xi in xr.iter().take(xrn) {
                    for &yj in yr.iter().take(yrn) {
                        if feq(xi, yj) {
                            add_root!(xi);
                        }
                    }
                }
            }
            return rootn;
        }
        points2coeff([sps[0].x, sps[1].x, sps[2].x, sps[3].x], &mut cx4);
        cx4[0] -= xcoeff[0];
        let xrn = solve3_coeff(&cx4, &mut xr);
        if xrn == 4 {
            return 4;
        }
        for &tv in xr.iter().take(xrn) {
            if (0.0..=1.0).contains(&tv) {
                let mut coeff = [0.0f64; 4];
                points2coeff([sps[0].x, sps[1].x, sps[2].x, sps[3].x], &mut coeff);
                let sv = coeff[0] + tv * (coeff[1] + tv * (coeff[2] + tv * coeff[3]));
                let sv = (sv - ycoeff[0]) / ycoeff[1];
                if (0.0..=1.0).contains(&sv) {
                    add_root!(tv);
                }
            }
        }
        return rootn;
    }

    let rat = ycoeff[1] / xcoeff[1];
    let mut coeff = [0.0f64; 4];
    points2coeff(
        [
            sps[0].y - rat * sps[0].x,
            sps[1].y - rat * sps[1].x,
            sps[2].y - rat * sps[2].x,
            sps[3].y - rat * sps[3].x,
        ],
        &mut coeff,
    );
    coeff[0] += rat * xcoeff[0] - ycoeff[0];
    let xrn = solve3_coeff(&coeff, &mut xr);
    if xrn == 4 {
        return 4;
    }
    for &tv in xr.iter().take(xrn) {
        if (0.0..=1.0).contains(&tv) {
            points2coeff([sps[0].x, sps[1].x, sps[2].x, sps[3].x], &mut coeff);
            let sv = coeff[0] + tv * (coeff[1] + tv * (coeff[2] + tv * coeff[3]));
            let sv = (sv - xcoeff[0]) / xcoeff[1];
            if (0.0..=1.0).contains(&sv) {
                add_root!(tv);
            }
        }
    }
    rootn
}

fn points2coeff(v: [f64; 4], coeff: &mut [f64; 4]) {
    coeff[3] = v[3] + 3.0 * v[1] - (v[0] + 3.0 * v[2]);
    coeff[2] = 3.0 * v[0] + 3.0 * v[2] - 6.0 * v[1];
    coeff[1] = 3.0 * (v[1] - v[0]);
    coeff[0] = v[0];
}

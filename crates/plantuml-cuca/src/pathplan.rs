//! Pure-Rust port of graphviz's self-contained `libpathplan` plus the
//! box-to-polygon machinery used by dot's edge routing.
//!
//! Sources (graphviz 14.1.2), each item cites the original file:
//! - `lib/pathplan/solvers.c` — cubic root solver
//! - `lib/pathplan/triang.c` — ear-clipping triangulation
//! - `lib/pathplan/shortest.c` — shortest path inside a simple polygon
//! - `lib/pathplan/route.c` — cubic Bezier spline fitting
//! - `lib/common/routespl.c` — box list to polygon
//!
//! Coordinates are dotsvg pixels (y grows upward), matching the values the
//! Java Smetana integration feeds the solver.

#![allow(clippy::similar_names)]

const EPSILON1: f64 = 1e-3;
const EPSILON2: f64 = 1e-6;

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
    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }
    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }
    fn scale(self, c: f64) -> Self {
        Self::new(self.x * c, self.y * c)
    }
}

fn dot(a: Point, b: Point) -> f64 {
    a.x * b.x + a.y * b.y
}

fn dist(a: Point, b: Point) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

/// Bit-exact float equality, mirroring C's `==` on `double` coordinates.
fn feq(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

/// Ported from: lib/pathplan/pathgeom.h (`Pedge_t`).
#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub a: Point,
    pub b: Point,
}


// ── Orientation ─────────────────────────────────────────────────────────

const ISCW: i32 = 1;
const ISCCW: i32 = 2;
const ISON: i32 = 3;

/// Ported from: lib/pathplan/triang.c (`ccw`).
fn ccw(p1: Point, p2: Point, p3: Point) -> i32 {
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
/// Returns roots in `[0, f64]` slots; `coeff` is constant..cubic order.
fn solve3(coeff: [f64; 4], roots: &mut [f64; 3]) -> usize {
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

// ── Ear-clipping triangulation (triang.c) ─────────────────────────────────

/// `between`: is pb between pa and pc?
fn between(pa: Point, pb: Point, pc: Point) -> bool {
    let pba = pb.sub(pa);
    let pca = pc.sub(pa);
    if ccw(pa, pb, pc) != ISON {
        return false;
    }
    dot(pca, pba) >= 0.0 && dot(pca, pca) <= dot(pba, pba)
}

/// `intersects`: line to line intersection.
fn intersects(pa: Point, pb: Point, pc: Point, pd: Point) -> bool {
    if ccw(pa, pb, pc) == ISON || ccw(pa, pb, pd) == ISON
        || ccw(pc, pd, pa) == ISON || ccw(pc, pd, pb) == ISON
    {
        return between(pa, pb, pc)
            || between(pa, pb, pd)
            || between(pc, pd, pa)
            || between(pc, pd, pb);
    }
    let c1 = (ccw(pa, pb, pc) == ISCCW) as i32;
    let c2 = (ccw(pa, pb, pd) == ISCCW) as i32;
    let c3 = (ccw(pc, pd, pa) == ISCCW) as i32;
    let c4 = (ccw(pc, pd, pb) == ISCCW) as i32;
    (c1 ^ c2) != 0 && (c3 ^ c4) != 0
}

/// `isdiagonal`, indexed over a working vertex list.
fn is_diagonal(poly: &[Point], i: usize, ip2: usize) -> bool {
    let n = poly.len();
    let ip1 = (i + 1) % n;
    let im1 = (i + n - 1) % n;
    let res = if ccw(poly[im1], poly[i], poly[ip1]) == ISCCW {
        ccw(poly[i], poly[ip2], poly[im1]) == ISCCW
            && ccw(poly[ip2], poly[i], poly[ip1]) == ISCCW
    } else {
        ccw(poly[i], poly[ip2], poly[ip1]) == ISCW
    };
    if !res {
        return false;
    }
    for j in 0..n {
        let jp1 = (j + 1) % n;
        if j != i && jp1 != i && j != ip2 && jp1 != ip2
            && intersects(poly[i], poly[ip2], poly[j], poly[jp1])
        {
            return false;
        }
    }
    true
}

// ── Triangle storage (shortest.c) ─────────────────────────────────────────

/// Triangle edge referencing vertex indices of a shared point table.
#[derive(Clone, Copy)]
struct TEdge {
    v0: usize,
    v1: usize,
    right: usize,
}

#[derive(Clone)]
struct Tri {
    e: [TEdge; 3],
    mark: i32,
}

const NO_IDX: usize = usize::MAX;

/// Ear-clip the working polygon (a mutable list of point-table indices),
/// appending triangles. Ported from shortest.c `triangulate`/`loadtriangle`.
fn clip_triangles(cur: &mut Vec<usize>, tris: &mut Vec<Tri>) {
    if cur.len() > 3 {
        let mut found = false;
        for pnli in 0..cur.len() {
            let ip1 = (pnli + 1) % cur.len();
            let ip2 = (pnli + 2) % cur.len();
            // isdiagonal on the current vertex sequence; build a temp polygon.
            let poly: Vec<Point> = cur.iter().map(|&v| unsafe_point(v)).collect();
            if is_diagonal(&poly, pnli, ip2) {
                load_triangle(cur[pnli], cur[ip1], cur[ip2], tris);
                cur.remove(ip1);
                found = true;
                break;
            }
        }
        assert!(found, "triangulation failed");
        clip_triangles(cur, tris);
    } else {
        load_triangle(cur[0], cur[1], cur[2], tris);
    }
}

fn load_triangle(a: usize, b: usize, c: usize, tris: &mut Vec<Tri>) {
    tris.push(Tri {
        e: [
            TEdge { v0: a, v1: b, right: NO_IDX },
            TEdge { v0: b, v1: c, right: NO_IDX },
            TEdge { v0: c, v1: a, right: NO_IDX },
        ],
        mark: 0,
    });
}

// Point table is threaded through the API; the triangulator accesses vertices
// via this thread-local to keep the port structurally identical to the C
// indexer callback.
thread_local! {
    static POINTS: std::cell::RefCell<Vec<Point>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn unsafe_point(v: usize) -> Point {
    POINTS.with(|p| p.borrow()[v])
}

/// Connect triangle pairs sharing an edge. Ported from `connecttris`.
fn connect_tris(tris: &mut [Tri]) {
    let n = tris.len();
    for t1 in 0..n {
        for t2 in t1 + 1..n {
            for ei in 0..3 {
                for ej in 0..3 {
                    let a = tris[t1].e[ei];
                    let b = tris[t2].e[ej];
                    if (a.v0 == b.v0 && a.v1 == b.v1) || (a.v0 == b.v1 && a.v1 == b.v0) {
                        tris[t1].e[ei].right = t2;
                        tris[t2].e[ej].right = t1;
                    }
                }
            }
        }
    }
}

/// Recursive triangle-strip marker. Ported from `marktripath`.
fn mark_tripath(tris: &mut [Tri], from: usize, to: usize) -> bool {
    if tris[from].mark != 0 {
        return false;
    }
    tris[from].mark = 1;
    if from == to {
        return true;
    }
    for ei in 0..3 {
        let right = tris[from].e[ei].right;
        if right != NO_IDX && mark_tripath(tris, right, to) {
            return true;
        }
    }
    tris[from].mark = 0;
    false
}

/// Point-in-triangle test. Ported from `pointintri`; vertices come from the
/// shared point table.
fn point_in_tri(tris: &[Tri], trii: usize, p: Point) -> bool {
    let mut sum = 0;
    for e in &tris[trii].e {
        if ccw(unsafe_point(e.v0), unsafe_point(e.v1), p) != ISCW {
            sum += 1;
        }
    }
    sum == 3 || sum == 0
}

// ── Funnel deque (shortest.c) ─────────────────────────────────────────────

/// Linkable funnel node. The first `n` handles are polygon vertices; handles
/// `n` and `n+1` are the two endpoints.
struct Funnel {
    /// Next node in the shortest-path list.
    link: Vec<Option<usize>>,
    /// Deque slots (handles), preallocated.
    dq: Vec<usize>,
    f: usize,
    l: usize,
    apex: usize,
}

impl Funnel {
    fn new(n: usize) -> Self {
        let total = n + 2;
        Self {
            link: vec![None; total],
            dq: vec![0; n * 2],
            f: n,
            l: n - 1,
            apex: 0,
        }
    }

    /// Ported from `add2dq`.
    fn add(&mut self, side: i32, h: usize) {
        if side == DQ_FRONT {
            if self.l >= self.f {
                self.link[h] = Some(self.dq[self.f]);
            }
            self.f -= 1;
            self.dq[self.f] = h;
        } else {
            if self.l >= self.f {
                self.link[h] = Some(self.dq[self.l]);
            }
            self.l += 1;
            self.dq[self.l] = h;
        }
    }

    /// Ported from `finddqsplit`.
    fn find_split(&self, h: usize) -> usize {
        let p = unsafe_point(h);
        for idx in self.f..self.apex {
            if ccw(unsafe_point(self.dq[idx + 1]), unsafe_point(self.dq[idx]), p) == ISCCW {
                return idx;
            }
        }
        let mut idx = self.l;
        while idx > self.apex {
            if ccw(unsafe_point(self.dq[idx - 1]), unsafe_point(self.dq[idx]), p) == ISCW {
                return idx;
            }
            idx -= 1;
        }
        self.apex
    }
}

const DQ_FRONT: i32 = 1;
const DQ_BACK: i32 = 2;

/// Find the shortest path inside `polygon` between `eps`.
/// Ported from: lib/pathplan/shortest.c (`Pshortestpath`).
///
/// The polygon may be in either winding order. Endpoints are appended to the
/// shared point table and referenced by handles `n` / `n+1`.
pub fn shortest_path(polygon: &[Point], start: Point, end: Point) -> Vec<Point> {

    // Install point table: working points + endpoints.
    let (working, start_h, end_h) = prepare_points(polygon, start, end);
    let n = working.len();
    debug_assert_eq!(start_h, n);
    debug_assert_eq!(end_h, n + 1);

    // Triangulate.
    let mut cur: Vec<usize> = (0..n).collect();
    let mut tris: Vec<Tri> = Vec::new();
    clip_triangles(&mut cur, &mut tris);
    connect_tris(&mut tris);

    let find_tri = |p: Point| -> Option<usize> {
        (0..tris.len()).find(|&t| point_in_tri(&tris, t, p))
    };
    let Some(first) = find_tri(start) else {
        return vec![start, end];
    };
    let Some(last) = find_tri(end) else {
        return vec![start, end];
    };

    if !mark_tripath(&mut tris, first, last) {
        return vec![start, end];
    }
    if first == last {
        return vec![start, end];
    }

    // Funnel.
    let mut fq = Funnel::new(n);
    fq.add(DQ_FRONT, start_h);
    fq.apex = fq.f;

    let mut trii = first;
    loop {
        let mark = tris[trii].mark;
        tris[trii].mark = 2;

        // Find the exiting edge toward an unvisited triangle.
        let mut ei = 3usize;
        for k in 0..3 {
            let right = tris[trii].e[k].right;
            if right != NO_IDX && tris[right].mark == mark {
                ei = k;
                break;
            }
        }

        let (lptr, rptr) = if ei == 3 {
            // Last triangle: orient against the endpoint.
            let lh = fq.dq[fq.l];
            if ccw(end, unsafe_point(fq.dq[fq.f]), unsafe_point(lh)) == ISCCW {
                (lh, end_h)
            } else {
                (end_h, lh)
            }
        } else {
            let a = tris[trii].e[ei];
            let other = tris[trii].e[(ei + 1) % 3].v1;
            if ccw(unsafe_point(a.v0), unsafe_point(other), unsafe_point(a.v1)) == ISCCW {
                (a.v1, a.v0)
            } else {
                (a.v0, a.v1)
            }
        };

        if trii == first {
            fq.add(DQ_BACK, lptr);
            fq.add(DQ_FRONT, rptr);
        } else if fq.dq[fq.f] != rptr && fq.dq[fq.l] != rptr {
            let split = fq.find_split(rptr);
            fq.l = split;
            fq.add(DQ_FRONT, rptr);
            if split > fq.apex {
                fq.apex = split;
            }
        } else {
            let split = fq.find_split(lptr);
            fq.f = split;
            fq.add(DQ_BACK, lptr);
            if split < fq.apex {
                fq.apex = split;
            }
        }

        // Advance to next triangle in strip.
        let mut next = NO_IDX;
        for k in 0..3 {
            let right = tris[trii].e[k].right;
            if right != NO_IDX && tris[right].mark == mark {
                next = right;
                break;
            }
        }
        if next == NO_IDX {
            break;
        }
        trii = next;
    }

    // Walk link chain from endpoint.
    let mut out = Vec::new();
    let mut h = end_h;
    loop {
        out.push(unsafe_point(h));
        match fq.link[h] {
            Some(nxt) => h = nxt,
            None => break,
        }
    }
    out.reverse();
    out
}

/// Fill the shared point table with a CCW working polygon plus endpoints,
/// dropping consecutive duplicate vertices. Ported from the orientation /
/// point-loading prologue of `Pshortestpath`. Returns (working point count
/// is implicit; start handle, end handle).
fn prepare_points(polygon: &[Point], start: Point, end: Point) -> (Vec<Point>, usize, usize) {
    let raw_n = polygon.len();

    // Leftmost vertex for orientation test.
    let mut minx = f64::INFINITY;
    let mut minpi = 0usize;
    for (i, p) in polygon.iter().enumerate() {
        if p.x < minx {
            minx = p.x;
            minpi = i;
        }
    }
    let p2 = polygon[minpi];
    let p1 = polygon[if minpi == 0 { raw_n - 1 } else { minpi - 1 }];
    let p3 = polygon[(minpi + 1) % raw_n];
    let reverse = (feq(p1.x, p2.x) && feq(p2.x, p3.x) && p3.y > p2.y)
        || ccw(p1, p2, p3) != ISCCW;

    let mut working: Vec<Point> = Vec::with_capacity(raw_n);
    if reverse {
        for pi in (0..raw_n).rev() {
            if pi < raw_n - 1
                && feq(polygon[pi].x, polygon[pi + 1].x)
                && feq(polygon[pi].y, polygon[pi + 1].y)
            {
                continue;
            }
            working.push(polygon[pi]);
        }
    } else {
        for (pi, p) in polygon.iter().enumerate() {
            if pi > 0 && feq(p.x, polygon[pi - 1].x) && feq(p.y, polygon[pi - 1].y) {
                continue;
            }
            working.push(*p);
        }
    }

    let n = working.len();
    let mut table = working.clone();
    table.push(start);
    table.push(end);
    POINTS.with(|p| *p.borrow_mut() = table);
    (working, n, n + 1)
}

// ── Spline fitting (route.c) ──────────────────────────────────────────────

/// Fitted-spline output accumulator (C: static `ops`/`opl`).
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
/// Ported from: lib/pathplan/route.c (`Proutespline`).
pub fn route_spline(
    barriers: &[Edge],
    input: &[Point],
    ev0: Point,
    ev1: Point,
) -> Vec<Point> {
    let mut out = SplineOut { ops: Vec::new() };
    out.grow(4);
    out.ops[0] = input[0];
    let mut opl = 1usize;
    really_route(barriers, input, ev0, ev1, &mut out, &mut opl);
    out.ops.truncate(opl);
    out.ops
}

/// Ported from `reallyroutespline`.
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
    // fit < 0 is an allocation failure in C; with Vec it cannot occur.

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

/// Fitted spline endpoints: start point/tangent and end point/tangent.
struct SplineEnds {
    p1: Point,
    v1: Point,
    p2: Point,
    v2: Point,
}

/// Ported from `mkspline`.
fn mk_spline(inps: &[Point], tnas: &[Tna], ev0: Point, ev1: Point) -> SplineEnds {
    let n = inps.len();
    let mut c = [[0.0f64; 2]; 2];
    let mut x = [0.0f64; 2];
    for i in 0..n {
        c[0][0] += dot(tnas[i].a[0], tnas[i].a[0]);
        c[0][1] += dot(tnas[i].a[0], tnas[i].a[1]);
        c[1][1] += dot(tnas[i].a[1], tnas[i].a[1]);
        let tmp = inps[i].sub(
            inps[0].scale(b01(tnas[i].t)).add(inps[n - 1].scale(b23(tnas[i].t))),
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

/// Ported from `splinefits`. Returns 1 on success, 0 when the spline must be
/// split; the forced-straight-line case also appends and returns 1.
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
        let sps = [
            pa,
            pa.add(va.scale(a / 3.0)),
            pb.sub(vb.scale(a / 3.0)),
            pb,
        ];

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

/// Ported from `splineisinside`.
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

/// Ported from `splineintersectsline`.
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

/// Ported from `points2coeff`.
fn points2coeff(v: [f64; 4], coeff: &mut [f64; 4]) {
    coeff[3] = v[3] + 3.0 * v[1] - (v[0] + 3.0 * v[2]);
    coeff[2] = 3.0 * v[0] + 3.0 * v[2] - 6.0 * v[1];
    coeff[1] = 3.0 * (v[1] - v[0]);
    coeff[0] = v[0];
}

/// Wrapper adapting the 4-slot coefficient/root arrays; handles the identity
/// sentinel from `solve1`.
fn solve3_coeff(coeff: &[f64; 4], roots: &mut [f64; 3]) -> usize {
    let n = solve3(*coeff, roots);
    if n == 4 {
        return 4;
    }
    n
}

// ── Box to polygon (routespl.c) ───────────────────────────────────────────

/// Axis-aligned rectangle. Ported from: lib/common/geom.h (`boxf`).
#[derive(Clone, Copy, Debug)]
pub struct Box {
    pub ll: Point,
    pub ur: Point,
}


/// Convert an ordered, vertically contiguous box list into the boundary
/// polygon used by the shortest-path router.
/// Ported from: lib/common/routespl.c (`routesplines_`, polygon assembly).
pub fn boxes_to_polygon(boxes_in: &[Box]) -> Vec<Point> {
    let mut boxes: Vec<Box> = boxes_in.to_vec();
    let boxn = boxes.len();

    // Flip so box LL.y is non-increasing? C flips when box0 is below box1.
    let flip = boxn > 1 && boxes[0].ll.y > boxes[1].ll.y;
    if flip {
        for b in &mut boxes {
            let v = b.ur.y;
            b.ur.y = -b.ll.y;
            b.ll.y = -v;
        }
    }
    let mut pp: Vec<Point> = Vec::new();

    // Forward pass over boxes.
    for bi in 0..boxn {
        let prev = if bi > 0 {
            if boxes[bi].ll.y > boxes[bi - 1].ll.y { -1 } else { 1 }
        } else {
            0
        };
        let next = if bi + 1 < boxn {
            if boxes[bi + 1].ll.y > boxes[bi].ll.y { 1 } else { -1 }
        } else {
            0
        };
        if prev != next {
            if next == -1 || prev == 1 {
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
            } else {
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
            }
        } else if prev == 0 {
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
        } else if !(prev == -1 && next == -1) {
            // Illegal geometry in C; cannot occur for our monotonic boxes.
        }
    }

    // Reverse pass over boxes.
    let mut bi = boxn;
    while bi > 0 {
        bi -= 1;
        let prev = if bi + 1 < boxn {
            if boxes[bi].ll.y > boxes[bi + 1].ll.y { -1 } else { 1 }
        } else {
            0
        };
        let next = if bi > 0 {
            if boxes[bi - 1].ll.y > boxes[bi].ll.y { 1 } else { -1 }
        } else {
            0
        };
        if prev != next {
            if next == -1 || prev == 1 {
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
                pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
            } else {
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
                pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
            }
        } else if prev == 0 {
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
        } else if prev == -1 && next == -1 {
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ll.y));
            pp.push(Point::new(boxes[bi].ur.x, boxes[bi].ur.y));
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ur.y));
            pp.push(Point::new(boxes[bi].ll.x, boxes[bi].ll.y));
        }
    }

    if flip {
        for p in &mut pp {
            p.y *= -1.0;
        }
    }
    pp
}

// ── de Casteljau split and shape clipping ──────────────────────────────────

/// Split a cubic Bezier at parameter `t`.
/// Ported from: lib/common/utils.c (`Bezier`).
fn bezier_split(v: &[Point; 4], t: f64) -> ([Point; 4], [Point; 4]) {
    let mut vt = [[Point::new(0.0, 0.0); 4]; 4];
    vt[0] = *v;
    for i in 1..=3 {
        for j in 0..=3 - i {
            vt[i][j] = Point::new(
                (1.0 - t) * vt[i - 1][j].x + t * vt[i - 1][j + 1].x,
                (1.0 - t) * vt[i - 1][j].y + t * vt[i - 1][j + 1].y,
            );
        }
    }
    let left = [vt[0][0], vt[1][0], vt[2][0], vt[3][0]];
    let right = [vt[3][0], vt[2][1], vt[1][2], vt[0][3]];
    (left, right)
}

/// Node shape for endpoint clipping, with half extents about the center.
#[derive(Clone, Copy, Debug)]
pub enum NodeShape {
    Rect { half_w: f64, half_h: f64 },
    Ellipse { half_w: f64, half_h: f64 },
}

impl NodeShape {
    /// Ported from the normalized inside test of lib/common/shapes.c
    /// (`poly_inside`): ellipse uses the normalized-radius test, rect the
    /// half-extent test. Coordinates are relative to the node center.
    fn inside(&self, p: Point) -> bool {
        match *self {
            Self::Rect { half_w, half_h } => p.x.abs() <= half_w && p.y.abs() <= half_h,
            Self::Ellipse { half_w, half_h } => {
                (p.x / half_w).hypot(p.y / half_h) < 1.0
            }
        }
    }
}

/// Endpoint of a routed edge: its center and shape.
#[derive(Clone, Copy, Debug)]
pub struct Endpoint {
    pub center: Point,
    pub shape: NodeShape,
}

/// Clip a cubic in place to the node boundary at one end.
/// Ported from: lib/common/splines.c (`bezier_clip`); `left_inside` means
/// curve[0] is inside the node, else curve[3] is.
pub fn bezier_clip(curve: &mut [Point; 4], end: Endpoint, left_inside: bool) {
    let rel = |p: Point| Point::new(p.x - end.center.x, p.y - end.center.y);
    let is_inside = |p: Point| end.shape.inside(rel(p));

    let mut low = 0.0f64;
    let mut high = 1.0f64;
    let mut pt = if left_inside { curve[0] } else { curve[3] };
    let mut best = *curve;
    let mut found = false;
    loop {
        let opt = pt;
        let t = (high + low) / 2.0;
        let (left, right) = bezier_split(curve, t);
        let seg = if left_inside { &right } else { &left };
        pt = if left_inside { right[0] } else { left[3] };
        if is_inside(pt) {
            if left_inside {
                low = t;
            } else {
                high = t;
            }
            best = *seg;
            found = true;
        } else if left_inside {
            high = t;
        } else {
            low = t;
        }
        if (opt.x - pt.x).abs() <= 0.5 && (opt.y - pt.y).abs() <= 0.5 {
            break;
        }
    }
    *curve = if found { best } else { bezier_split(curve, (high + low) / 2.0).1 };
}


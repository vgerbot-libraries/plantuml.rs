//! Ear-clipping triangulation and shortest path inside a simple polygon.
//!
//! Ported from graphviz 14.1.2:
//! - `lib/pathplan/triang.c` (`ccw`, `between`, `intersects`, `isdiagonal`)
//! - `lib/pathplan/shortest.c` (`triangulate`, `connecttris`, `marktripath`,
//!   `Pshortestpath`)
//!
//! Unlike the cuca port, the shared point table is threaded explicitly
//! instead of through a thread-local, and a failed triangulation degrades to
//! a straight segment rather than panicking.

#![allow(clippy::similar_names)]

use super::geom::{ccw, dist, dot, feq, Point, ISCCW, ISCW, ISON};

// ── Ear-clipping geometry ─────────────────────────────────────────────────

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

/// `isdiagonal` over a working vertex slice resolved against `table`.
fn is_diagonal(poly: &[usize], table: &[Point], i: usize, ip2: usize) -> bool {
    let n = poly.len();
    let ip1 = (i + 1) % n;
    let im1 = (i + n - 1) % n;
    let at = |k: usize| table[poly[k]];
    let res = if ccw(at(im1), at(i), at(ip1)) == ISCCW {
        ccw(at(i), at(ip2), at(im1)) == ISCCW && ccw(at(ip2), at(i), at(ip1)) == ISCCW
    } else {
        ccw(at(i), at(ip2), at(ip1)) == ISCW
    };
    if !res {
        return false;
    }
    for j in 0..n {
        let jp1 = (j + 1) % n;
        if j != i && jp1 != i && j != ip2 && jp1 != ip2
            && intersects(at(i), at(ip2), at(j), at(jp1))
        {
            return false;
        }
    }
    true
}

// ── Triangle storage ──────────────────────────────────────────────────────

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
/// appending triangles. Returns false if no ear could be clipped.
fn clip_triangles(cur: &mut Vec<usize>, table: &[Point], tris: &mut Vec<Tri>) -> bool {
    if cur.len() > 3 {
        let mut clipped = false;
        for pnli in 0..cur.len() {
            let ip1 = (pnli + 1) % cur.len();
            let ip2 = (pnli + 2) % cur.len();
            if is_diagonal(cur, table, pnli, ip2) {
                load_triangle(cur[pnli], cur[ip1], cur[ip2], tris);
                cur.remove(ip1);
                clipped = true;
                break;
            }
        }
        if !clipped {
            return false;
        }
        clip_triangles(cur, table, tris)
    } else {
        load_triangle(cur[0], cur[1], cur[2], tris);
        true
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

/// Connect triangle pairs sharing an edge.
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

/// Recursive triangle-strip marker.
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

/// Point-in-triangle test against the shared point table.
fn point_in_tri(tris: &[Tri], table: &[Point], trii: usize, p: Point) -> bool {
    let mut sum = 0;
    for e in &tris[trii].e {
        if ccw(table[e.v0], table[e.v1], p) != ISCW {
            sum += 1;
        }
    }
    sum == 3 || sum == 0
}

// ── Funnel deque ─────────────────────────────────────────────────────────

struct Funnel {
    link: Vec<Option<usize>>,
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

    fn find_split(&self, table: &[Point], h: usize) -> usize {
        let p = table[h];
        for idx in self.f..self.apex {
            if ccw(table[self.dq[idx + 1]], table[self.dq[idx]], p) == ISCCW {
                return idx;
            }
        }
        let mut idx = self.l;
        while idx > self.apex {
            if ccw(table[self.dq[idx - 1]], table[self.dq[idx]], p) == ISCW {
                return idx;
            }
            idx -= 1;
        }
        self.apex
    }
}

const DQ_FRONT: i32 = 1;
const DQ_BACK: i32 = 2;

/// Build a CCW working polygon plus endpoints, dropping consecutive
/// duplicates. Returns (working points, start handle, end handle).
fn prepare_points(polygon: &[Point], start: Point, end: Point) -> (Vec<Point>, usize, usize, usize) {
    let raw_n = polygon.len();

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
    (table, n, n, n + 1)
}

/// Find the shortest path inside `polygon` between `start` and `end`.
///
/// The polygon may be in either winding. On any failure to construct the
/// triangulated path, a straight `start -> end` segment is returned.
pub fn shortest_path(polygon: &[Point], start: Point, end: Point) -> Vec<Point> {
    if polygon.len() < 3 {
        return vec![start, end];
    }
    let (table, n, start_h, end_h) = prepare_points(polygon, start, end);

    let mut cur: Vec<usize> = (0..n).collect();
    let mut tris: Vec<Tri> = Vec::new();
    if !clip_triangles(&mut cur, &table, &mut tris) || tris.len() < 2 {
        return vec![start, end];
    }
    connect_tris(&mut tris);

    let find_tri = |p: Point| (0..tris.len()).find(|&t| point_in_tri(&tris, &table, t, p));
    let Some(first) = find_tri(start) else {
        return vec![start, end];
    };
    let Some(last) = find_tri(end) else {
        return vec![start, end];
    };

    if !mark_tripath(&mut tris, first, last) || first == last {
        return vec![start, end];
    }

    let mut fq = Funnel::new(n);
    fq.add(DQ_FRONT, start_h);
    fq.apex = fq.f;

    let mut trii = first;
    loop {
        let mark = tris[trii].mark;
        tris[trii].mark = 2;

        let mut ei = 3usize;
        for k in 0..3 {
            let right = tris[trii].e[k].right;
            if right != NO_IDX && tris[right].mark == mark {
                ei = k;
                break;
            }
        }

        let (lptr, rptr) = if ei == 3 {
            let lh = fq.dq[fq.l];
            if ccw(end, table[fq.dq[fq.f]], table[lh]) == ISCCW {
                (lh, end_h)
            } else {
                (end_h, lh)
            }
        } else {
            let a = tris[trii].e[ei];
            let other = tris[trii].e[(ei + 1) % 3].v1;
            if ccw(table[a.v0], table[other], table[a.v1]) == ISCCW {
                (a.v1, a.v0)
            } else {
                (a.v0, a.v1)
            }
        };

        if trii == first {
            fq.add(DQ_BACK, lptr);
            fq.add(DQ_FRONT, rptr);
        } else if fq.dq[fq.f] != rptr && fq.dq[fq.l] != rptr {
            let split = fq.find_split(&table, rptr);
            fq.l = split;
            fq.add(DQ_FRONT, rptr);
            if split > fq.apex {
                fq.apex = split;
            }
        } else {
            let split = fq.find_split(&table, lptr);
            fq.f = split;
            fq.add(DQ_BACK, lptr);
            if split < fq.apex {
                fq.apex = split;
            }
        }

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

    let mut out = Vec::new();
    let mut h = end_h;
    loop {
        out.push(table[h]);
        match fq.link[h] {
            Some(nxt) => h = nxt,
            None => break,
        }
    }
    out.reverse();
    if out.len() < 2 || dist(out[0], start) > 1e-6 {
        return vec![start, end];
    }
    out
}

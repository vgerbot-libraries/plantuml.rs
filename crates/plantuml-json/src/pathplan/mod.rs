//! Pure-Rust port of graphviz's self-contained `libpathplan` plus the
//! box-to-polygon machinery used by dot's edge routing.
//!
//! Sources (graphviz 14.1.2):
//! - `lib/pathplan/solvers.c` — cubic root solver ([`geom`])
//! - `lib/pathplan/triang.c` — ear-clipping triangulation ([`shortest`])
//! - `lib/pathplan/shortest.c` — shortest path inside a simple polygon
//!   ([`shortest`])
//! - `lib/pathplan/route.c` — cubic Bezier spline fitting ([`route`])
//! - `lib/common/routespl.c` — box list to polygon ([`boxes`])
//!
//! Coordinates are dot internal-frame pixels (y grows upward).

pub mod boxes;
pub mod geom;
pub mod route;
pub mod shortest;

pub use boxes::{bezier_clip, boxes_to_polygon, Box, Endpoint, NodeShape};
pub use geom::{Edge, Point};
pub use route::route_spline;
pub use shortest::shortest_path;

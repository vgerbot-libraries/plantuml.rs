//! Mindmap contour layout.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/mindmap/MindMap.java`
//! - `net/sourceforge/plantuml/mindmap/Branch.java`
//! - `net/sourceforge/plantuml/mindmap/FingerImpl.java`
//! - `net/sourceforge/plantuml/mindmap/Tetris.java`
//! - `net/sourceforge/plantuml/mindmap/Stripe.java`
//! - `net/sourceforge/plantuml/mindmap/StripeFrontier.java`
//! - `net/sourceforge/plantuml/mindmap/SymetricalTee.java`
//! - `net/sourceforge/plantuml/mindmap/SymetricalTeePositioned.java`
//!
//! The regular (right) and reverse (left) sides are laid out independently by
//! `FingerImpl`; a parent packs its child subtrees against an upper-envelope
//! `StripeFrontier` and recenters the row. `MindMap` finally aligns both sides
//! on the shared root, translating by `reverse.getX12` horizontally and the
//! larger half-thickness vertically.

use crate::idea::{Idea, MindMapDirection};
use crate::text_metrics::box_size;

/// Point size of every node label.
const FONT_SIZE: i32 = 14;
/// Style margin around a node, every side (`PName.Margin` = 10).
const MARGIN: f64 = 10.0;
/// Extra horizontal run on the far side (`getX2`: margin right + 30).
const FAR_PAD: f64 = 30.0;
/// `FingerImpl.getX12` = getX1 + getX2 = 10 + 40.
const X12: f64 = MARGIN + (MARGIN + FAR_PAD);
/// Page origin of the drawn mindmap (exporter margin).
const PAGE_X: f64 = 10.0;
const PAGE_Y: f64 = 10.0;
/// Canvas enlargement over the `MindMap` dimension (measured against the jar).
const CANVAS_PAD_W: f64 = 30.0;
const CANVAS_PAD_H: f64 = 20.0;

// ── SymetricalTee ─────────────────────────────────────────────────────────

/// The two stacked elbows of a laid-out subtree (`SymetricalTee`).
#[derive(Clone, Copy, Debug)]
struct Tee {
    /// Thickness of band 1.
    t1: f64,
    /// Elongation of band 1 (includes the getX1 lead for an internal node).
    e1: f64,
    /// Thickness of band 2.
    t2: f64,
    /// Elongation of band 2 (includes the getX2 lead for an internal node).
    e2: f64,
}

impl Tee {
    fn max_x(self) -> f64 {
        self.e1 + self.e2
    }
}

// ── Stripe / StripeFrontier ───────────────────────────────────────────────

/// One horizontal band of the packing frontier (`Stripe`).
#[derive(Clone, Copy, Debug)]
struct Stripe {
    x1: f64,
    x2: f64,
    value: f64,
}

/// Upper-envelope step function of already-placed subtrees (`StripeFrontier`).
#[derive(Debug)]
struct Frontier {
    stripes: Vec<Stripe>,
}

impl Frontier {
    fn new() -> Self {
        Self {
            stripes: vec![Stripe {
                x1: f64::NEG_INFINITY,
                x2: f64::INFINITY,
                value: f64::NEG_INFINITY,
            }],
        }
    }

    fn is_empty(&self) -> bool {
        self.stripes.len() == 1
    }

    /// Stripes overlapping `[x1, x2]` (`collisionning`).
    fn collisionning(&self, x1: f64, x2: f64) -> Vec<Stripe> {
        let mut result = Vec::new();
        for stripe in &self.stripes {
            if x1 >= stripe.x2 {
                continue;
            }
            result.push(*stripe);
            if x2 <= stripe.x2 {
                return result;
            }
        }
        result
    }

    /// Highest frontier value across `[x1, x2]` (`getContact`).
    fn get_contact(&self, x1: f64, x2: f64) -> f64 {
        self.collisionning(x1, x2)
            .iter()
            .fold(f64::NEG_INFINITY, |acc, s| acc.max(s.value))
    }

    /// Raise the envelope to `value` over `[x1, x2]` (`addSegment`).
    fn add_segment(&mut self, x1: f64, x2: f64, value: f64) {
        if x2 <= x1 {
            return;
        }
        let collisions = self.collisionning(x1, x2);
        if collisions.len() > 1 {
            let mut x = x1;
            for stripe in collisions.iter().skip(1) {
                self.add_segment(x, stripe.x1, value);
                x = stripe.x1;
            }
            self.add_segment(x, x2, value);
        } else if let Some(touch) = collisions.first().copied() {
            self.add_single_internal(x1, x2, value, touch);
        }
    }

    /// `addSingleInternal`.
    fn add_single_internal(&mut self, x1: f64, x2: f64, value: f64, touch: Stripe) {
        if value <= touch.value {
            return;
        }
        if let Some(pos) = self
            .stripes
            .iter()
            .position(|s| s.x1 == touch.x1 && s.x2 == touch.x2 && s.value == touch.value)
        {
            self.stripes.remove(pos);
        }
        if touch.x1 != x1 {
            self.stripes.push(Stripe { x1: touch.x1, x2: x1, value: touch.value });
        }
        self.stripes.push(Stripe { x1, x2, value });
        if x2 != touch.x2 {
            self.stripes.push(Stripe { x1: x2, x2: touch.x2, value: touch.value });
        }
        self.stripes
            .sort_by(|a, b| a.x1.partial_cmp(&b.x1).unwrap_or(std::cmp::Ordering::Equal));
    }
}

// ── Positioned tee ────────────────────────────────────────────────────────

/// A `Tee` plus its placed centre offset (`SymetricalTeePositioned`).
#[derive(Clone, Copy, Debug)]
struct Positioned {
    tee: Tee,
    y: f64,
}

impl Positioned {
    const fn new(tee: Tee) -> Self {
        Self { tee, y: 0.0 }
    }

    /// Move so the top of band 1 sits on `new_y` (`moveSoThatSegmentA1isOn`).
    fn move_a1_on(&mut self, new_y: f64) {
        let current = self.y - self.tee.t1 / 2.0;
        self.y += new_y - current;
    }

    /// Move so the top of band 2 sits on `new_y` (`moveSoThatSegmentA2isOn`).
    fn move_a2_on(&mut self, new_y: f64) {
        let current = self.y - self.tee.t2 / 2.0;
        self.y += new_y - current;
    }

    fn min_y(self) -> f64 {
        self.y - self.tee.t1.max(self.tee.t2) / 2.0
    }

    fn max_y(self) -> f64 {
        self.y + self.tee.t1.max(self.tee.t2) / 2.0
    }
}

// ── Tetris ────────────────────────────────────────────────────────────────

/// Packs a row of child subtrees on one shared frontier (`Tetris`).
#[derive(Debug)]
struct Tetris {
    frontier: Frontier,
    elements: Vec<Positioned>,
    min_y: f64,
    max_y: f64,
}

impl Tetris {
    fn new() -> Self {
        Self {
            frontier: Frontier::new(),
            elements: Vec::new(),
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    /// Pack one child tee (`add`).
    fn add(&mut self, tee: Tee) {
        let result = if self.frontier.is_empty() {
            Positioned::new(tee)
        } else {
            let c1 = self.frontier.get_contact(0.0, tee.e1);
            let c2 = self.frontier.get_contact(tee.e1, tee.e1 + tee.e2);

            let mut p1 = Positioned::new(tee);
            p1.move_a1_on(c1);
            let mut p2 = Positioned::new(tee);
            p2.move_a2_on(c2);
            if p2.y > p1.y { p2 } else { p1 }
        };
        self.add_internal(result);
    }

    /// `addInternal`.
    fn add_internal(&mut self, result: Positioned) {
        let tee = result.tee;
        self.elements.push(result);
        self.frontier.add_segment(0.0, tee.e1, result.y + tee.t1 / 2.0);
        if tee.e2 > 0.0 {
            self.frontier
                .add_segment(tee.e1, tee.e1 + tee.e2, result.y + tee.t2 / 2.0);
        }
    }

    /// Recenter the packed row on y = 0 (`balance`).
    fn balance(&mut self) {
        if self.elements.is_empty() {
            return;
        }
        for element in &self.elements {
            self.min_y = self.min_y.min(element.min_y());
            self.max_y = self.max_y.max(element.max_y());
        }
        let mean = (self.min_y + self.max_y) / 2.0;
        for stp in &mut self.elements {
            stp.y -= mean;
        }
    }

    fn height(&self) -> f64 {
        if self.elements.is_empty() {
            0.0
        } else {
            self.max_y - self.min_y
        }
    }

    fn width(&self) -> f64 {
        self.elements.iter().fold(0.0, |acc, e| acc.max(e.tee.max_x()))
    }
}

// ── Laid-out node tree ────────────────────────────────────────────────────

/// A node placed in its side's finger coordinate system.
pub(crate) struct LaidNode {
    /// Label text.
    pub label: String,
    /// Box rectangle width.
    pub width: f64,
    /// Box rectangle height.
    pub height: f64,
    /// Centre offset relative to the parent finger origin (root = 0).
    pub cy: f64,
    /// Children with offsets resolved.
    pub children: Vec<Self>,
    /// Phalanx thickness (0 if this root phalanx is hidden).
    phalanx_t: f64,
    /// Phalanx box width (0 if hidden).
    pub phalanx_e: f64,
    /// Whether the node's own box is drawn.
    pub draw_phalanx: bool,
    /// Internal tee used when this node is packed by its parent.
    tee: Tee,
}

impl LaidNode {
    /// Phalanx thickness: box height plus the top/bottom style margins.
    fn phalanx_thickness(height: f64) -> f64 {
        height + 2.0 * MARGIN
    }

    /// Build this node and its descendants for direction `dir`.
    ///
    /// `hide_phalanx` (the reverse root of a two-sided map) suppresses the
    /// node's own box and zeroes its phalanx contribution, since the regular
    /// side draws the shared root.
    fn build(idea: &Idea, dir: MindMapDirection, hide_phalanx: bool) -> Self {
        let (width, height) = box_size(&idea.label, FONT_SIZE);

        // Children belonging to this side.
        let kids: Vec<&Idea> = idea
            .children
            .iter()
            .filter(|c| c.direction == dir)
            .collect();

        let phalanx_t_full = Self::phalanx_thickness(height);
        let (phalanx_t, phalanx_e) = if hide_phalanx {
            (0.0, 0.0)
        } else {
            (phalanx_t_full, width)
        };

        if kids.is_empty() {
            // Leaf: SymetricalTee(thickness1, elongation1, 0, 0).
            let tee = Tee { t1: phalanx_t, e1: phalanx_e, t2: 0.0, e2: 0.0 };
            return Self {
                label: idea.label.clone(),
                width,
                height,
                cy: 0.0,
                children: Vec::new(),
                phalanx_t,
                phalanx_e,
                draw_phalanx: !hide_phalanx,
                tee,
            };
        }

        // Recursively build children, then pack with a Tetris.
        let child_nodes: Vec<Self> = kids
            .iter()
            .map(|c| Self::build(c, dir, false))
            .collect();

        let mut tetris = Tetris::new();
        for child in &child_nodes {
            tetris.add(child.tee);
        }
        tetris.balance();

        let nail_h = tetris.height();
        let nail_w = tetris.width();

        // Internal tee (`asSymetricalTee`): elongation1 adds getX1; band 2
        // leads with getX2 before the nail elongation.
        let tee = Tee {
            t1: phalanx_t,
            e1: phalanx_e + MARGIN,
            t2: nail_h,
            e2: MARGIN + FAR_PAD + nail_w,
        };

        let mut children = child_nodes;
        for (child, placed) in children.iter_mut().zip(tetris.elements.iter()) {
            child.cy = placed.y;
        }

        Self {
            label: idea.label.clone(),
            width,
            height,
            cy: 0.0,
            children,
            phalanx_t,
            phalanx_e,
            draw_phalanx: !hide_phalanx,
            tee,
        }
    }

    /// Nail (packed children) width: band-2 elongation minus its getX2 lead.
    fn nail_width(&self) -> f64 {
        (self.tee.e2 - (MARGIN + FAR_PAD)).max(0.0)
    }

    /// Nail thickness.
    fn nail_thickness(&self) -> f64 {
        self.tee.t2
    }

    /// `FingerImpl.getFullThickness` = max(phalanx, nail).
    fn full_thickness(&self) -> f64 {
        self.phalanx_t.max(self.nail_thickness())
    }

    /// `FingerImpl.getFullElongation` = phalanx elongation + nail elongation.
    fn full_elongation(&self) -> f64 {
        self.phalanx_e + self.nail_width()
    }

    /// `Branch.getX12` = full elongation + FingerImpl.getX12.
    fn side_extent(&self) -> f64 {
        self.full_elongation() + X12
    }
}

// ── Side / whole mindmap ──────────────────────────────────────────────────

/// One laid-out side; `root` is `None` for an empty side.
pub(crate) struct Side {
    pub root: Option<LaidNode>,
}

impl Side {
    /// `Branch.getHalfThickness`.
    fn half_thickness(&self) -> f64 {
        self.root.as_ref().map_or(0.0, |r| r.full_thickness() / 2.0)
    }

    /// `Branch.getX12`.
    fn extent(&self) -> f64 {
        self.root.as_ref().map_or(0.0, LaidNode::side_extent)
    }
}

/// Whole-mindmap layout: sides, shared root centre, and canvas size.
pub(crate) struct Layout {
    pub right: Side,
    pub left: Side,
    /// Root finger origin in page coordinates (regular side draws the box).
    pub root_ox: f64,
    pub root_oy: f64,
    pub canvas_w: f64,
    pub canvas_h: f64,
}

/// Lays out an idea tree into the two sides.
#[must_use]
pub(crate) fn layout(root: &Idea) -> Layout {
    let has_left = root
        .children
        .iter()
        .any(|c| c.direction == MindMapDirection::Left);

    let right = Side {
        root: Some(LaidNode::build(root, MindMapDirection::Right, false)),
    };
    // On a two-sided map the reverse root phalanx is hidden (regular draws it).
    let left = Side {
        root: has_left.then(|| LaidNode::build(root, MindMapDirection::Left, true)),
    };

    // MindMap local translate.
    let translate_x = left.extent();
    let translate_y = right.half_thickness().max(left.half_thickness());

    let dim_w = left.extent() + right.extent();
    let dim_h = translate_y + right.half_thickness().max(left.half_thickness());

    Layout {
        right,
        left,
        root_ox: PAGE_X + translate_x,
        root_oy: PAGE_Y + translate_y,
        canvas_w: dim_w.ceil() + CANVAS_PAD_W,
        canvas_h: dim_h.ceil() + CANVAS_PAD_H,
    }
}

use spacemath::two::boundary::{Boundary, Edge};
use spacemath::two::dist::Dist;
use spacemath::two::Point;

use crate::reader::PartModel;

#[derive(Debug, Clone, Copy)]
pub struct Mark {
    pub annot: Annotation,
    pub inter: Interaction,
}

impl Mark {
    pub fn click(&mut self) {
        self.inter = Interaction::Clicked;
    }

    pub fn annotate(&mut self, annot: Annotation) {
        self.annot = annot;
    }

    pub fn is_clicked(&self) -> bool {
        self.inter == Interaction::Clicked
    }

    pub fn is_force(&self) -> bool {
        match self.annot {
            Annotation::Force(_, _) => true,
            _ => false,
        }
    }

    pub fn is_constraint(&self) -> bool {
        match self.annot {
            Annotation::Constraint(_, _) => true,
            _ => false,
        }
    }
}

impl Default for Mark {
    fn default() -> Mark {
        Mark {
            annot: Annotation::Empty,
            inter: Interaction::Ignored,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Annotation {
    Empty,
    Constraint(bool, bool),
    Force(f64, f64),
}

impl Annotation {
    pub fn parse_constraint(x: &str) -> Option<Self> {
        // expect x", "y", or "xy"
        match x {
            "x" => Some(Self::Constraint(true, false)),
            "y" => Some(Self::Constraint(false, true)),
            "xy" => Some(Self::Constraint(true, true)),
            _ => None,
        }
    }

    pub fn parse_force(x: &str) -> Option<Self> {
        // expect (0.05, 1.205)
        // do this with nom when I have internet
        if !x.starts_with("(") {
            return None;
        }

        if !x.ends_with(")") {
            return None;
        }

        let x = x.trim_matches(|c| c == '(' || c == ')');

        let mut vals = x.split(", ");
        let x = vals.next()?.parse().ok()?;
        let y = vals.next()?.parse().ok()?;

        Some(Self::Force(x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interaction {
    Ignored,
    Clicked,
}

// a boundary with
#[derive(Debug, Clone)]
pub struct MarkedBound {
    bound: Boundary,
    marks: Vec<Mark>,
}

impl MarkedBound {
    pub fn edges_and_marks<'a>(&'a self) -> impl Iterator<Item = (&'a Edge, &'a Mark)> + 'a {
        self.bound.edges().zip(self.marks.iter())
    }

    pub fn from_edges_and_marks(edges: Vec<Edge>, marks: Vec<Mark>) -> Self {
        Self {
            bound: Boundary::new(edges),
            marks,
        }
    }

    pub fn pos_edge_index(&self, pos: Point) -> usize {
        self.bound
            .edges()
            .enumerate()
            .min_by(|(_, x), (_, y)| x.dist(pos).partial_cmp(&y.dist(pos)).unwrap())
            .unwrap()
            .0
    }

    pub fn click_at_pos(&mut self, pos: Point) -> (Edge, Mark) {
        // apply mark to the edge closest to the given point, return the edge and resulting mark
        let i = self.pos_edge_index(pos);
        self.marks[i].click();

        return (*self.bound.edges().nth(i).unwrap(), self.marks[i]);
    }

    pub fn annotate_clicked(&mut self, annot: Annotation) -> Vec<(Edge, Mark)> {
        let mut res = Vec::new();

        for (e, m) in self.bound.edges().zip(self.marks.iter_mut()) {
            if m.is_clicked() {
                m.annotate(annot);
                res.push((*e, *m));
            }
        }

        res
    }

    pub fn clear_interactions(&mut self) {
        for m in self.marks.iter_mut() {
            m.inter = Interaction::Ignored
        }
    }

    pub fn into_parts(self) -> (Boundary, Vec<Mark>) {
        (self.bound, self.marks)
    }

    pub fn segmentify(self, len: f64) -> Self {
        // convert boundary to segments, cloning marks as appropriate
        // len is maximum segment length of subdivided curve

        let mut res_edges = Vec::new();
        let mut res_marks = Vec::new();

        for (e, m) in self.edges_and_marks() {
            let edge_segments = e.into_segments(len);

            for _ in 0..edge_segments.len() {
                res_marks.push(m.clone());
            }

            res_edges.extend(edge_segments);
        }

        Self::from_edges_and_marks(res_edges, res_marks)
    }
}

impl From<Boundary> for MarkedBound {
    fn from(bound: Boundary) -> Self {
        let n = bound.num_edges();

        MarkedBound {
            bound,
            marks: vec![Mark::default(); n],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarkedModel(Vec<MarkedBound>);

impl MarkedModel {
    pub fn bounds<'a>(&'a self) -> impl Iterator<Item = &'a MarkedBound> + 'a {
        self.0.iter()
    }

    pub fn click_at_pos(&mut self, pos: Point) -> (Edge, Mark) {
        // apply the mark to the closest boundary to the given point
        let b = self
            .0
            .iter_mut()
            .min_by(|x, y| x.bound.dist(pos).partial_cmp(&y.bound.dist(pos)).unwrap())
            .unwrap();

        b.click_at_pos(pos)
    }

    pub fn annotate_clicked(&mut self, annot: Annotation) -> Vec<(Edge, Mark)> {
        let mut res = Vec::new();

        for b in self.0.iter_mut() {
            res.extend(b.annotate_clicked(annot));
        }

        res
    }

    pub fn clear_interactions(&mut self) {
        for b in self.0.iter_mut() {
            b.clear_interactions()
        }
    }

    pub fn bounding_box(&self) -> (Point, Point) {
        // currently relies on the outer bound coming first (see below From<PartModel>)
        self.0[0].bound.bounding_box()
    }

    pub fn segmentify(&mut self, len: f64) {
        // convert boundaries to segments, cloning marks as appropriate
        // len is maximum segment length of subdivided curve

        for b in self.0.iter_mut() {
            *b = b.clone().segmentify(len); // boo clone
        }
    }
}

impl From<PartModel> for MarkedModel {
    fn from(model: PartModel) -> Self {
        let mut res = Vec::new();

        res.push(model.outer_bound.into());
        for b in model.inner_bounds {
            res.push(b.into())
        }

        Self(res)
    }
}

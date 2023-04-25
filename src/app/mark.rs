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
    pub fn clicked() -> Self {
        let mut res = Self::default();
        res.inter = Interaction::Clicked;
        res
    }

    pub fn is_clicked(&self) -> bool {
        self.inter == Interaction::Clicked
    }

    pub fn is_empty(&self) -> bool {
        self.annot == Annotation::Empty && self.inter == Interaction::Ignored
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
    Constraint,
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interaction {
    Ignored,
    Hovered,
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

    pub fn mark_at_pos(&mut self, pos: Point, mark: Mark) {
        // find index of edge closest to pos
        let i = self
            .bound
            .edges()
            .enumerate()
            .min_by(|(_, x), (_, y)| x.dist(pos).partial_cmp(&y.dist(pos)).unwrap())
            .unwrap()
            .0;

        self.marks[i] = mark;
    }

    pub fn clear_interactions(&mut self) {
        for m in self.marks.iter_mut() {
            m.inter = Interaction::Ignored
        }
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

    pub fn mark_at_pos(&mut self, pos: Point, mark: Mark) {
        // apply the mark to the closest boundary to the given point
        let b = self
            .0
            .iter_mut()
            .min_by(|x, y| x.bound.dist(pos).partial_cmp(&y.bound.dist(pos)).unwrap())
            .unwrap();

        b.mark_at_pos(pos, mark);
    }

    pub fn clear_interactions(&mut self) {
        for b in self.0.iter_mut() {
            b.clear_interactions()
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

mod handle;

use handle::{EdgeHandle, PointStore};

use spacemath::two::boundary::Boundary;
use spacemath::two::Point;

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct PartModel {
    pub(crate) outer_bound: Boundary,
    pub(crate) inner_bounds: Vec<Boundary>,
}

impl PartModel {
    pub fn load_dxf<T: AsRef<Path>>(source: T) -> Self {
        let input = dxf::Drawing::load_file(source).unwrap();

        let mut store = PointStore::new();
        let mut edges = Vec::new();

        // construct edge list
        for ent in input.entities().cloned() {
            edges.extend(EdgeHandle::from_entity(&mut store, ent))
        }

        // group segments into boundaries
        let mut bounds: Vec<Boundary> = Vec::new();

        while !edges.is_empty() {
            let mut bound_edges = vec![edges.pop().unwrap()];
            let first_id = bound_edges.first().unwrap().p_id();

            // find a closed loop of segments
            loop {
                let cur_e = bound_edges.last().unwrap();

                if bound_edges.last().unwrap().q_id() == first_id {
                    break;
                }

                let next_pos = edges
                    .iter()
                    .position(|e| e.as_starting_with(cur_e.q_id()).is_some())
                    .expect("dangling edge");

                let new_edge = edges
                    .swap_remove(next_pos)
                    .as_starting_with(cur_e.q_id())
                    .unwrap();
                bound_edges.push(new_edge);
            }

            // loses point id information
            bounds.push(Boundary::new(bound_edges));
        }

        // one of the bounds will enclose all bounds
        // all others will be enclosed by that one and no others

        let mut enclosing = None;

        for i in 0..bounds.len() {
            let cur_bound = bounds.get(i).unwrap();

            // check if all but the current boundary are enclosed
            let mut all_but_cur = bounds
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, v)| v);
            if all_but_cur.all(|b| cur_bound.contains_boundary(b)) {
                enclosing = Some(i);
            }
        }

        let mut enclosing = bounds.swap_remove(enclosing.expect("no enclosing boundary"));

        // store outer bound in positive orientation and inner bounds in negative orientation
        enclosing.orient_positive();
        for b in bounds.iter_mut() {
            b.orient_negative();
        }

        // TODO: handle improper models where a boundary is enclosed by more than one (nested)
        // TODO: check that no boundaries intersect

        Self {
            outer_bound: enclosing,
            inner_bounds: bounds,
        }
    }
}

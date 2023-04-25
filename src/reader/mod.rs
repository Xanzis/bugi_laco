mod handle;

use handle::{EdgeHandle, PointStore};

use std::collections::{HashMap, BTreeSet};
use spacemath::two::boundary::Boundary;
use spacemath::two::Point;

#[derive(Clone, Debug)]
pub struct PartModel {
	pub (crate) outer_bound: Boundary,
	pub (crate) inner_bounds: Vec<Boundary>,
}

impl PartModel {
	pub fn load_dxf() -> Self {
		let input = dxf::Drawing::load_file("example_files/test.DXF").unwrap();

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
					break
				}

				let next_pos = edges
					.iter()
					.position(|e| e.as_starting_with(cur_e.q_id()).is_some())
					.expect("dangling edge");

				bound_edges.push(edges.swap_remove(next_pos));
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
			let mut all_but_cur = bounds.iter().enumerate().filter(|&(j, _)| j != i).map(|(_, v)| v);
			if all_but_cur.all(|b| cur_bound.contains_boundary(b)) {
				enclosing = Some(i);
			}
		}

		let enclosing = bounds.swap_remove(enclosing.expect("no enclosing bounday"));

		// TODO: handle improper models where a boundary is enclosed by more than one (nested)
		// TODO: check that no boundaries intersect

		Self { outer_bound: enclosing, inner_bounds: bounds }
	}
}
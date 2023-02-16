mod bounds;

use std::collections::{HashMap, HashSet};
use crate::spatial::Point;
use bounds::{Segment, ClosedBound};

pub struct PartModel {
	points: PointStore,
}

impl PartModel {
	pub fn load_dxf() -> Self {
		let input = dxf::Drawing::load_file("example_files/test.DXF").unwrap();

		let mut store = PointStore::new();
		let mut segments = Vec::new();

		// construct segment list
		for ent in input.entities() {
			println!("{:#?}\n", ent);

			if let Some(seg) = Segment::maybe_from_entity(ent, &mut store) {
				segments.push(seg)
			}
		}

		println!("{:#?}", segments);

		// group segments into boundaries
		let mut bounds: Vec<ClosedBound> = Vec::new()
		let mut remaining_points: HashSet<_> = store.all_ids().cloned().collect();

		while !remaining_points.is_empty() {
			// select an unclaimed starting point
			let first = remaining_points.iter().next().unwrap().clone();

			let mut bound_points = vec![first];
			let mut bound_segments = vec![];

			// find a closed loop of segments
			for i in 0.. {
				let cur_a = bound_points[i];
				
				// find a segment, avoiding picking the previous segment if i > 0
				let mut new_seg = if i == 0 {
					segments.iter()
						.filter_map(|s| s.as_starting_with(cur_a))
						.next().expect("dangling point")
				} else {
					segments.iter()
						.filter_map(|s| s.as_starting_with(cur_a))
						.filter(|s| s.end_point() != bound_points[i-1])
						.next().expect("dangling point")
				}

				bound_points.push(new_seg.end_id());
				bound_segments.push(new_seg);

				if bound_points.last().unwrap() == bound_points[0] {
					break;
				}
			}

			bound_points.iter().for_each(|p| remaining_points.remove(p));
			bounds.push(ClosedBound::from_segments(bound_segments));
		}

		// one of the bounds will enclose all bounds
		// all others will be enclosed by that one and no others

		unimplemented!("Not done lol")
	}
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PointId(usize);

pub struct PointStore(HashMap<PointId, Point>);

impl PointStore {
	fn new() -> Self {
		Self(HashMap::new())
	}

	fn next_id(&self) -> PointId {
		PointId(self.0.len())
	}

	fn id_or_insert(&mut self, p: Point) -> PointId {
		// hot garbage O(n) approach
		// necessary because arcs are defined by center / angle but
		//   arc end points need to alias with line end points
		// quadtrees are probably the right way to go about this

		for (id, stored_p) in self.0.iter() {
			// ad hoc tolerance
			if stored_p.dist(p) < 1e-6 {
				return *id
			}
		}

		let new_id = self.next_id();
		self.0.insert(new_id, p);

		new_id
	}

	fn all_ids(&self) -> impl Iterator<Item=&PointId> {
		self.0.iter_keys()
	}
}
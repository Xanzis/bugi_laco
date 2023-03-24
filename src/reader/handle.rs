use std::collections::HashMap;

use spacemath::two::boundary::Edge;
use spacemath::two::point::Point;
use spacemath::two::line::{Segment, Arc};

use dxf::entities::Entity;

fn dxf_point(p: &dxf::Point) -> Point {
	Point::new(p.x, p.y)
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PointId(usize);

#[derive(Debug)]
pub struct PointStore(HashMap<PointId, Point>);

impl PointStore {
	pub fn new() -> Self {
		Self(HashMap::new())
	}

	fn next_id(&self) -> PointId {
		PointId(self.0.len())
	}

	pub fn id_or_insert(&mut self, p: Point) -> PointId {
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

	pub fn all_ids(&self) -> impl Iterator<Item=&PointId> {
		self.0.keys()
	}
}

#[derive(Clone, Debug)]
pub struct EdgeHandle {
	edge: Edge,
	p_id: PointId,
	q_id: PointId,
}

impl EdgeHandle {
	pub fn from_entity(store: &mut PointStore, ent: Entity) -> Option<Self> {
		match ent.specific.to_owned() {
			dxf::entities::EntityType::Line(l) => {
				let p_id = store.id_or_insert(dxf_point(&l.p1));
				let q_id = store.id_or_insert(dxf_point(&l.p2));

				let edge = Segment::new(dxf_point(&l.p1), dxf_point(&l.p2)).into();
				
				Some(Self {edge, p_id, q_id})
			},
			dxf::entities::EntityType::Arc(a) => {
				assert_eq!(a.normal, dxf::Vector::z_axis());

				let center: Point = dxf_point(&a.center);
				let r = a.radius;

				let start = a.start_angle.to_radians();
				let end = a.end_angle.to_radians();

				let p = (Point::unit(start) * r) + center;
				let q = (Point::unit(end) * r) + center;

				let p_id = store.id_or_insert(p);
				let q_id = store.id_or_insert(q);

				// currently assuming dxfs always list arcs in ccw direction
				let edge = Arc::from_center_ang(center, r, start, end, true).into();

				Some(Self {edge, p_id, q_id})
			},
			x => {
				println!("no implementation for:\n{:?}", x);
				None
			},
		}
	}

	pub fn p_id(&self) -> PointId {
		self.p_id
	}

	pub fn q_id(&self) -> PointId {
		self.q_id
	}

	pub fn reversed(&self) -> Self {
		Self {
			edge: self.edge.reverse(),
			p_id: self.q_id,
			q_id: self.p_id,
		}
	}

	pub fn as_starting_with(&self, id: PointId) -> Option<Self> {
		if self.p_id == id {
			Some(self.clone())
		} else if self.q_id == id {
			Some(self.reversed())
		} else {
			None
		}
	}
}

impl From<EdgeHandle> for Edge {
	fn from(x: EdgeHandle) -> Edge {
		x.edge
	}
}
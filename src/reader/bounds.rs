use super::{PointStore, PointId};
use crate::spatial::Point;

#[derive(Clone, Debug)]
struct ClosedBound(Vec<Segment>);

impl ClosedBound {
	fn from_segments(s: Vec<Segment>) {
		Self(s)
	}
}

#[derive(Clone, Debug)]
enum Segment {
	Line { a: PointId, b: PointId },

	// arc gets redundant info
	Arc { a: PointId, b: PointId, center: PointId, start: f64, end: f64},
}

impl Segment {
	pub fn maybe_from_entity(ent: &dxf::entities::Entity, store: &mut PointStore) -> Option<Self> {
		match ent.specific.to_owned() {
			dxf::entities::EntityType::Line(l) => {
				let a = store.id_or_insert(l.p1.into());
				let b = store.id_or_insert(l.p2.into());

				Some(Segment::Line{a, b})
			},
			dxf::entities::EntityType::Arc(a) => {
				assert_eq!(a.normal, dxf::Vector::z_axis());

				let center_p: Point = a.center.into();
				let rad = a.radius;

				let start = a.start_angle.to_radians();
				let end = a.end_angle.to_radians();

				let start_p = (Point::unit_vec(start) * rad) + center_p;
				let end_p = (Point::unit_vec(end) * rad) + center_p;

				let a = store.id_or_insert(start_p);
				let b = store.id_or_insert(end_p);
				let center = store.id_or_insert(center_p);

				Some(Segment::Arc{a, b, center, start, end})
			},
			_ => None,
		}
	}

	pub fn start_id(&self) -> PointId {
		match *self {
			Self::Line{a, ..} => a,
			Self::Arc{a, ..} => a,
		}
	}

	pub fn end_id(&self) -> PointId {
		match *self {
			Self::Line{b, ..} => b,
			Self::Arc{b, ..} => b,
		}
	}

	pub fn start_point(&self, store: &PointStore) -> Point {

	}

	pub fn reversed(&self) -> Self {
		match self.clone() {
			Self::Line{a, b} => {
				Self::Line{b: a, a: b}
			}
			Self::Arc{a, b, center, start, end} => {
				Self::Arc{b: a, a: b, center, end: start, start: end}
			},
		}
	}

	pub fn as_starting_with(&self, pid: PointId) -> Option<Self> {
		if self.start_id() == pid {
			return Some(self.clone())
		}

		if self.end_id() == pid {
			Some(self.reversed())
		} else {
			None
		}
	}

	pub fn intersects_hray(&self, store: &PointStore, origin: Point) -> bool {
		// checks for intersection with horizontal ray
		// to prevent double counting, the check is for whether 
	}
}
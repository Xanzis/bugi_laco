use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Point(f64, f64);

impl Point {
	pub fn dist(&self, other: Self) -> f64 {
		((self.0 - other.0).powi(2) + (self.1 - other.1).powi(2)).sqrt()
	}

	pub fn as_tuple(&self) -> (f64, f64) {
		(self.0, self.1)
	}

	pub fn unit_vec(angle: f64) -> Self {
		Self(angle.cos(), angle.sin())
	}
}

impl From<dxf::Point> for Point {
	fn from(p: dxf::Point) -> Self {
		// just drop z values (maybe non-planar inputs should error?)
		Self(p.x, p.y)
	}
}

impl From<(f64, f64)> for Point {
	fn from(p: (f64, f64)) -> Self {
		Self(p.0, p.1)
	}
}

impl Add for Point {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self(self.0 + other.0, self.1 + other.1)
	}
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

impl Mul<f64> for Point {
	type Output = Self;

	fn mul(self, other: f64) -> Self {
		Self(self.0 * other, self.1 * other)
	}
}
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use spacemath::two::Point;

use crate::app::mark::{Annotation, MarkedBound};

pub struct Writer {
    points: Vec<Vec<(String, Option<String>)>>,
    constraints: Vec<((String, String), (bool, bool))>,
    forces: Vec<((String, String), (f64, f64))>,

    // for unit conversions (bbnd is meters)
    scale: f64,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            constraints: Vec::new(),
            forces: Vec::new(),

            scale: 1.0,
        }
    }

    pub fn scale(self, scale: f64) -> Self {
        Self {
            scale,
            ..self
        }
    }

    pub fn add_boundary(&mut self, marked_bound: MarkedBound) {
        let (bound, marks) = marked_bound.into_parts();

        let mut points: Vec<(String, Option<String>)> = bound
            .points()
            .into_iter()
            .map(|p| (format_point(p * self.scale), None))
            .collect();

        // gross
        let edge_vertices: Vec<(usize, usize)> = (0..points.len())
            .zip((1..points.len()).chain(0..1))
            .collect();

        // treats all edges as segments. Break arcs down before this point
        for (vs, mark) in edge_vertices.iter().zip(marks) {
            if mark.is_force() {
                // both the edge's vertices get used
                let p_label = point_label(&points[vs.0].0);
                let q_label = point_label(&points[vs.1].0);

                // some duplicate computation here but oh well
                points[vs.0].1 = Some(p_label.clone());
                points[vs.1].1 = Some(q_label.clone());

                let f = match mark.annot {
                    Annotation::Force(x, y) => (x, y),
                    _ => unreachable!(),
                };

                self.forces.push(((p_label, q_label), f));
            } else if mark.is_constraint() {
                let p_label = point_label(&points[vs.0].0);
                let q_label = point_label(&points[vs.1].0);

                points[vs.0].1 = Some(p_label.clone());
                points[vs.1].1 = Some(q_label.clone());

                let c = match mark.annot {
                    Annotation::Constraint(x, y) => (x, y),
                    _ => unreachable!(),
                };

                self.constraints.push(((p_label, q_label), c));
            }
        }

        self.points.push(points);
    }

    pub fn write<T: AsRef<Path>>(self, path: T, material: &str, thickness: f64) {
        let mut to_write = String::new();

        // write all the polygons
        for polygon in self.points {
            to_write.push_str("polygon\n");

            for (point, label) in polygon {
                to_write.push_str(&point);

                if let Some(l) = label {
                    to_write.push(' ');
                    to_write.push_str(&l);
                }

                to_write.push('\n');
            }

            to_write.push_str("end\n");
        }

        // fill in proper material selection logic (with user input somewhere)
        to_write.push_str(&format!("thickness {}\n", thickness));
        to_write.push_str(&format!("material {}\n", material));

        // write all the constraints
        for ((p_label, q_label), c) in self.constraints {
            let c = format!(
                "distributed_constraint {} {} {}\n",
                p_label,
                q_label,
                format_constraint(c)
            );
            to_write.push_str(&c);
        }

        // write all the forces
        for ((p_label, q_label), f) in self.forces {
            let f = format!(
                "distributed_force {} {} {} {}\n",
                p_label, q_label, f.0, f.1
            );
            to_write.push_str(&f);
        }

        to_write.pop(); // pull off trailing whitespace

        std::fs::write(path, to_write).unwrap();
    }
}

fn point_label(p_str: &str) -> String {
    let mut hasher = DefaultHasher::new();
    p_str.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn format_point(p: Point) -> String {
    format!("{:.5} {:.5}", p.x, p.y)
}

fn format_constraint(c: (bool, bool)) -> String {
    // allocating but whatever

    let mut res = String::new();

    if c.0 {
        res.push('x');
    }

    if c.1 {
        res.push('y');
    }

    return res;
}

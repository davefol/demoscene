use std::f32::consts::PI;

use nalgebra::{Point3, Vector3};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct Cone {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Cone {
    pub fn new(height: f32, base_radius: f32, base_samples: usize) -> Self {
        if base_samples < 3 {
            panic!("base_samples must be greater than 3");
        }
        let mut points: Vec<Point3<f32>> = Vec::new();
        points.reserve_exact(base_samples + 1);

        let mut normals: Vec<Vector3<f32>> = Vec::new();
        normals.reserve_exact(base_samples + 1);

        // The cone points toward the screen
        points.push(Point3::new(0.0, 0.0, height));
        normals.push(Vector3::new(0.0, 0.0, 1.0));

        // the base of the cone is a circle on the xy plane.
        for i in 0..base_samples {
            // sample along a full circle (2pi)
            let theta: f32 = i as f32 / base_samples as f32 * 2f32 * PI;
            let x = base_radius * theta.cos();
            let y = base_radius * theta.sin();
            let point = Point3::new(x, y, 0f32);

            // the normal of the vertex points from the tip to the vertex
            let normal = Vector3::new(x, y, -height).normalize();

            points.push(point);
            normals.push(normal);
        }

        // cone side wall faces are triangles formed by pairs of base points and the tip
        let mut triangles: Vec<[usize; 3]> = Vec::new();
        for i in 1..base_samples {
            let a = i;
            let b = if i + 1 == base_samples { 1 } else { i + 1 };
            triangles.push([0, a, b]);
        }

        // bottom is formed by taking the the first base point as the tip of triangles
        // formed of adjacent pairs of other base points.
        for i in 2..(base_samples - 1) {
            let a = i;
            let b = i + 1;
            triangles.push([1, b, a]);
        }

        Self {
            vertices: points
                .iter()
                .zip(normals)
                .map(|(p, n)| Vertex {
                    position: [p.x, p.y, p.z],
                    normal: [n.x, n.y, n.z],
                })
                .collect(),
            indices: triangles.iter().flatten().map(|i| *i as u32).collect(),
        }
    }
}

impl Default for Cone {
    fn default() -> Self {
        Cone::new(1.0, 0.5, 32)
    }
}
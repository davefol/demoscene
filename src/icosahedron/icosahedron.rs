use std::{
    collections::HashSet,
    f32::consts::{PHI, PI},
};

use nalgebra::{Point3, Rotation3, distance};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

pub(super) struct Icosahedron {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Icosahedron {
    pub fn new() -> Icosahedron {
        // vertices should land on unit circle (sphere but z is 0)
        // sqrt(x**2 + y**2) = 1
        // ratio of x to y should be golden ratio
        // x = y * g
        // implies...
        // sqrt((y * g)**2 + y**2) = 1
        // (y * g)**2 + y**2 = 1
        // y**2 * g**2 + y**2 = 1
        // g**2 + 1 = 1 / y**2
        // y**2 * (g**2 + 1) = 1
        // y**2 = 1 / (g**2 + 1)
        // y = sqrt(1 / (g**2 + 1))
        let y = (1.0 / (PHI * PHI + 1.0)).sqrt();
        let x = y * PHI;

        // single rectangle in R3 whose corners
        // lie on unit sphere
        let seed_positions = vec![
            Point3::new(-x, y, 0.0),
            Point3::new(-x, -y, 0.0),
            Point3::new(x, y, 0.0),
            Point3::new(x, -y, 0.0),
        ];

        // rotate the above rectangle to get the other corners
        // of the icosahedron
        let mut positions = vec![];
        positions.extend_from_slice(&seed_positions);
        let rot1 = Rotation3::from_euler_angles(PI / 2.0, PI / 2.0, 0.0);
        positions.extend(seed_positions.iter().map(|pt| rot1.transform_point(pt)));
        let rot2 = Rotation3::from_euler_angles(0.0, 0.0, PI / 2.0)
            * Rotation3::from_euler_angles(PI / 2.0, 0.0, 0.0);
        positions.extend(seed_positions.iter().map(|pt| rot2.transform_point(pt)));

        // recover faces according to a few rules
        // 1. Since all the triangles that make up the icosahedron are equilateral
        //    two vertices share an edge if they have the minium distance

        let mut edge_length = distance(&positions[0], &positions[1]);
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                edge_length = edge_length.min(distance(&positions[i], &positions[j]));
            }
        }

        let mut neighbors = vec![Vec::<usize>::new(); positions.len()];
        const EPS: f32 = 1e-4;
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let d = distance(&positions[i], &positions[j]);
                if (edge_length - d).abs() < EPS {
                    neighbors[i].push(j);
                    neighbors[j].push(i);
                }
            }
        }

        // 2. if a has neighbors b and c and b and c are also neighbors then abc
        //    forms a face.
        let mut faces: Vec<[usize; 3]> = Vec::new();
        let mut seen: HashSet<[usize; 3]> = HashSet::new();
        for a in 0..positions.len() {
            for bi in 0..neighbors[a].len() {
                for ci in (bi + 1)..neighbors[a].len() {
                    let b = neighbors[a][bi];
                    let c = neighbors[a][ci];
                    if neighbors[b].contains(&c) {
                        let mut face = [a, b, c];
                        face.sort();
                        if !seen.contains(&face) {
                            seen.insert(face);
                            let n =
                                (positions[b] - positions[a]).cross(&(positions[c] - positions[a]));
                            if n.dot(&positions[a].coords) < 0.0 {
                                faces.push([a, c, b]);
                            } else {
                                faces.push([a, b, c]);
                            }
                        }
                    }
                }
            }
        }

        let vertices = positions
            .iter()
            .map(|p| {
                let n = p.coords.normalize();
                Vertex {
                    position: [p.x, p.y, p.z],
                    normal: [n.x, n.y, n.z],
                }
            })
            .collect();

        let indices = faces.iter().flatten().map(|i| *i as u32).collect();

        Icosahedron { vertices, indices }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{Point3, distance};
    use super::Icosahedron;

    #[test]
    fn test_icosahedron() {
        let icosahedron = Icosahedron::new();
        assert_eq!(icosahedron.vertices.len(), 12);
        assert_eq!(icosahedron.indices.len(), 20 * 3);

        // test all faces are equilateral and same area
        let side_lens = icosahedron
            .indices
            .iter()
            .array_chunks::<3>()
            .map(|f| {
                let a = Point3::from_slice(
                    &icosahedron.vertices[*f[0] as usize].position,
                );
                let b = Point3::from_slice(
                    &icosahedron.vertices[*f[1] as usize].position,
                );
                let c = Point3::from_slice(
                    &icosahedron.vertices[*f[2] as usize].position,
                );
                [distance(&a, &b), distance(&a, &c), distance(&b, &c)]
            })
            .flatten()
            .collect::<Vec<f32>>();
        for s in side_lens.iter() {
            assert!((s - side_lens[0]).abs() < 1e-4);
        }
    }
}

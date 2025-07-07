use std::{
    collections::{HashMap, HashSet},
    f32::consts::{PHI, PI},
};

use nalgebra::{Point3, Rotation3, distance};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

pub(super) struct Icosphere {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// sperhically interpolate from a to b assuming a and b are on unit sphere.
fn slerp(a: &Point3<f32>, b: &Point3<f32>, t: f32) -> Point3<f32> {
    let theta = a.coords.dot(&b.coords).acos();
    ((a * ((1.0 - t) * theta).sin() / theta.sin()).coords
        + (b * (t * theta).sin() / theta.sin()).coords)
        .into()
}

impl Icosphere {
    pub fn new(resolution: u8) -> Icosphere {
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

        let mut edge_was_split = HashMap::<[usize; 2], usize>::new();
        // Subdivide the icosahedron resolution times to get the sphere
        for _ in 0..resolution {
            faces = faces.iter().map(|face| {
                // split face half way along its three edges to form
                // 4 triangles out of the original one.
                let [i0, i1, i2] = *face;
                let p0 = positions[i0];
                let p1 = positions[i1];
                let p2 = positions[i2];

                // Instead of simply linearly interpolating to go halfay
                // between two points, we'll spherically interpolate
                // to stay on the unit sphere
                let p01 = slerp(&p0, &p1, 0.5);
                let p02 = slerp(&p0, &p2, 0.5);
                let p12 = slerp(&p1, &p2, 0.5);

                // add new points to our positions structure while de-duplicating
                // and keeping track of their index.
                let i01 = {
                    let mut edge = [i0, i1];
                    edge.sort();
                    if edge_was_split.contains_key(&edge) {
                        edge_was_split[&edge]
                    } else {
                        let new_idx = positions.len();
                        positions.push(p01);
                        edge_was_split.insert(edge, new_idx);
                        new_idx
                    }
                };
                let i02 = {
                    let mut edge = [i0, i2];
                    edge.sort();
                    if edge_was_split.contains_key(&edge) {
                        edge_was_split[&edge]
                    } else {
                        let new_idx = positions.len();
                        positions.push(p02);
                        edge_was_split.insert(edge, new_idx);
                        new_idx
                    }
                };
                let i12 = {
                    let mut edge = [i1, i2];
                    edge.sort();
                    if edge_was_split.contains_key(&edge) {
                        edge_was_split[&edge]
                    } else {
                        let new_idx = positions.len();
                        positions.push(p12);
                        edge_was_split.insert(edge, new_idx);
                        new_idx
                    }
                };

                // now we need to emit 4 new faces, paying attention to orientation
                // (front is Ccw)
                [
                    [i0, i01, i02],
                    [i01, i1, i12],
                    [i02, i12, i2],
                    [i01, i12, i02],
                ]
            }).flatten().collect();
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

        Icosphere { vertices, indices }
    }
}

#[cfg(test)]
mod tests {
    use super::Icosphere;
    use nalgebra::{Point3, distance};

    #[test]
    fn test_icosphere_2() {
        let resolution = 2;
        let icosphere = Icosphere::new(resolution);
        assert_eq!(icosphere.vertices.len(), 10 * 2usize.pow(resolution as u32).pow(2) + 2);
        assert_eq!(icosphere.indices.len(), 20 * 2usize.pow(resolution as u32).pow(2) * 3);

        // test all vertices are on unit sphere
        for v in icosphere.vertices.iter() {
            let mag_sq = v.position[0].powi(2) + v.position[1].powi(2) + v.position[2].powi(2);
            assert!((1.0 - mag_sq).abs() < 1e-4);
        }
    }
}

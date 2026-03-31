use glam::Vec3;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SphereVertex {
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct Icosphere {
    pub vertices: Vec<SphereVertex>,
    pub indices: Vec<u32>,
}

impl Icosphere {
    pub fn new(subdivisions: u32) -> Self {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;

        let mut vertices = vec![
            Vec3::new(-1.0, phi, 0.0).normalize(),
            Vec3::new(1.0, phi, 0.0).normalize(),
            Vec3::new(-1.0, -phi, 0.0).normalize(),
            Vec3::new(1.0, -phi, 0.0).normalize(),
            Vec3::new(0.0, -1.0, phi).normalize(),
            Vec3::new(0.0, 1.0, phi).normalize(),
            Vec3::new(0.0, -1.0, -phi).normalize(),
            Vec3::new(0.0, 1.0, -phi).normalize(),
            Vec3::new(phi, 0.0, -1.0).normalize(),
            Vec3::new(phi, 0.0, 1.0).normalize(),
            Vec3::new(-phi, 0.0, -1.0).normalize(),
            Vec3::new(-phi, 0.0, 1.0).normalize(),
        ];

        let mut indices = vec![
            0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11,
            1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7, 1, 8,
            3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9,
            4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9, 8, 1,
        ];

        for _ in 0..subdivisions {
            let mut new_indices = Vec::new();
            let mut edge_cache: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();

            for i in (0..indices.len()).step_by(3) {
                let i0 = indices[i];
                let i1 = indices[i + 1];
                let i2 = indices[i + 2];

                let i01 = subdivide_edge(i0, i1, &mut vertices, &mut edge_cache);
                let i12 = subdivide_edge(i1, i2, &mut vertices, &mut edge_cache);
                let i20 = subdivide_edge(i2, i0, &mut vertices, &mut edge_cache);

                new_indices.extend_from_slice(&[i0, i01, i20]);
                new_indices.extend_from_slice(&[i01, i1, i12]);
                new_indices.extend_from_slice(&[i20, i12, i2]);
                new_indices.extend_from_slice(&[i01, i12, i20]);
            }

            indices = new_indices;
        }

        let sphere_vertices: Vec<SphereVertex> = vertices
            .iter()
            .map(|pos| SphereVertex {
                position: *pos,
                normal: *pos,
            })
            .collect();

        Icosphere {
            vertices: sphere_vertices,
            indices,
        }
    }
}

fn subdivide_edge(
    i0: u32,
    i1: u32,
    vertices: &mut Vec<Vec3>,
    cache: &mut std::collections::HashMap<(u32, u32), u32>,
) -> u32 {
    let key = if i0 < i1 { (i0, i1) } else { (i1, i0) };

    if let Some(&idx) = cache.get(&key) {
        return idx;
    }

    let v0 = vertices[i0 as usize];
    let v1 = vertices[i1 as usize];
    let mid = ((v0 + v1) / 2.0).normalize();

    let idx = vertices.len() as u32;
    vertices.push(mid);
    cache.insert(key, idx);

    idx
}

use std::collections::HashMap;

// render/capsule_renderer.rs
use glow::HasContext;
use parry3d::na;

use crate::render::world::program::BasicProgram;

pub struct CapsuleMesh {
    pub vao: glow::VertexArray,
    pub vbo: glow::Buffer,
    pub ebo: glow::Buffer,
    pub index_count: i32,
}

impl CapsuleMesh {
    pub fn new(
        gl: &glow::Context,
        rings: usize,
        segments: usize,
        radius: f32,
        half_height: f32,
    ) -> Self {
        // Build positions + normals for a capsule aligned along Y:
        // top hemisphere (center +half_height), cylinder, bottom hemisphere (center -half_height).
        let mut verts: Vec<[f32; 6]> = Vec::new(); // [px,py,pz, nx,ny,nz]
        let mut idx: Vec<u32> = Vec::new();

        let push = |verts: &mut Vec<[f32; 6]>, p: na::Vector3<f32>, n: na::Vector3<f32>| {
            verts.push([p.x, p.y, p.z, n.x, n.y, n.z]);
        };

        // hemispheres
        let hemi = |center_y: f32, sign: f32, verts: &mut Vec<[f32; 6]>, idx: &mut Vec<u32>| {
            let base = verts.len() as u32;
            for y in 0..=rings {
                let v = y as f32 / rings as f32; // 0..1
                // polar angle from 0..pi/2
                let theta = (v * 0.5 * std::f32::consts::PI);
                let sy = theta.sin();
                let cy = theta.cos();
                for x in 0..=segments {
                    let u = x as f32 / segments as f32;
                    let phi = u * 2.0 * std::f32::consts::PI;
                    let nx = cy * phi.cos();
                    let nz = cy * phi.sin();
                    let ny = sign * sy;
                    let n = na::Vector3::new(nx, ny, nz);
                    let p = na::Vector3::new(nx * radius, center_y + ny * radius, nz * radius);
                    push(verts, p, n);
                }
            }
            // indices
            let stride = (segments + 1) as u32;
            for y in 0..rings {
                for x in 0..segments {
                    let i0 = base + y as u32 * stride + x as u32;
                    let i1 = i0 + 1;
                    let i2 = i0 + stride;
                    let i3 = i2 + 1;
                    // triangle order CCW
                    idx.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
                }
            }
        };

        // top hemi (center +half_height), bottom hemi (center -half_height)
        hemi(half_height, 1.0, &mut verts, &mut idx);
        // cylinder
        let base_cyl = verts.len() as u32;
        for y in 0..=1 {
            let yy = -half_height + (y as f32) * (2.0 * half_height);
            for x in 0..=segments {
                let u = x as f32 / segments as f32;
                let phi = u * 2.0 * std::f32::consts::PI;
                let n = na::Vector3::new(phi.cos(), 0.0, phi.sin());
                let p = na::Vector3::new(n.x * radius, yy, n.z * radius);
                push(&mut verts, p, n);
            }
        }
        let stride = (segments + 1) as u32;
        for x in 0..segments {
            let i0 = base_cyl + x as u32;
            let i1 = i0 + 1;
            let i2 = i0 + stride;
            let i3 = i2 + 1;
            idx.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
        // bottom hemi
        hemi(-half_height, -1.0, &mut verts, &mut idx);

        // upload
        let (vao, vbo, ebo);
        unsafe {
            vao = gl.create_vertex_array().unwrap();
            vbo = gl.create_buffer().unwrap();
            ebo = gl.create_buffer().unwrap();
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&verts),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&idx),
                glow::STATIC_DRAW,
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 6 * 4, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 6 * 4, 3 * 4);
            gl.bind_vertex_array(None);
        }

        Self {
            vao,
            vbo,
            ebo,
            index_count: idx.len() as i32,
        }
    }

    pub fn draw(&self, gl: &glow::Context, prog: &BasicProgram, model: na::Matrix4<f32>) {
        unsafe {
            gl.use_program(Some(prog.program));
            if let Some(loc) = &prog.loc_model {
                gl.uniform_matrix_4_f32_slice(Some(loc), false, model.as_slice());
            }
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    r_mm: u32,
    h_mm: u32,
} // quantize to avoid f32 hashmap keys

fn key(radius: f32, half_height: f32) -> Key {
    Key {
        r_mm: (radius * 1000.0) as u32,
        h_mm: (half_height * 1000.0) as u32,
    }
}

pub struct CapsuleCache {
    meshes: HashMap<Key, CapsuleMesh>,
    rings: usize,
    segments: usize,
}

impl CapsuleCache {
    pub fn new(rings: usize, segments: usize) -> Self {
        Self {
            meshes: HashMap::new(),
            rings,
            segments,
        }
    }

    pub fn get_or_create(
        &mut self,
        gl: &glow::Context,
        radius: f32,
        half_height: f32,
    ) -> &CapsuleMesh {
        let k = key(radius, half_height);
        self.meshes
            .entry(k)
            .or_insert_with(|| CapsuleMesh::new(gl, self.rings, self.segments, radius, half_height))
    }

    pub fn destroy(&self, gl: &glow::Context) {
        for m in self.meshes.values() {
            m.destroy(gl);
        }
    }
}

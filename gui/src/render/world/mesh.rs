use glam::Vec3;
use glow::HasContext;
use parry3d::{na, shape::TriMesh};
use rerecast::PolygonNavmesh;

use crate::render::world::program::BasicProgram;

pub struct Mesh {
    pub vao: glow::VertexArray,
    pub vbo: glow::Buffer,
    pub ebo: glow::Buffer,
    pub index_count: i32,
}

// TODO: Lots of weird back-and-forth conversions here, clean up later.

impl Mesh {
    /// Upload interleaved [px,py,pz, nx,ny,nz] + u32 indices
    pub fn from_interleaved(gl: &glow::Context, interleaved: &[[f32; 6]], indices: &[u32]) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(interleaved),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices),
                glow::STATIC_DRAW,
            );

            // a_pos
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 6 * 4, 0);
            // a_nrm
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 6 * 4, 3 * 4);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Mesh {
                vao,
                vbo,
                ebo,
                index_count: indices.len() as i32,
            }
        }
    }

    fn smooth_normals(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Vec<na::Vector3<f32>> {
        let mut normals = vec![na::Vector3::<f32>::zeros(); positions.len()];
        for tri in triangles {
            let (ia, ib, ic) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            let a = na::Vector3::new(positions[ia][0], positions[ia][1], positions[ia][2]);
            let b = na::Vector3::new(positions[ib][0], positions[ib][1], positions[ib][2]);
            let c = na::Vector3::new(positions[ic][0], positions[ic][1], positions[ic][2]);
            let n = (b - a).cross(&(c - a));
            normals[ia] += n;
            normals[ib] += n;
            normals[ic] += n;
        }
        for n in &mut normals {
            let len = n.norm();
            if len > 1e-6 {
                *n /= len;
            } else {
                *n = na::Vector3::y();
            }
        }
        normals
    }

    /// Build from a Parry TriMesh (computes smooth normals if none are present)
    pub fn from_parry_trimesh(gl: &glow::Context, mesh: &TriMesh) -> Self {
        let positions = mesh.vertices();
        let triangles = mesh.indices();

        let normals = Self::smooth_normals(
            positions
                .iter()
                .map(|p| [p.x, p.y, p.z])
                .collect::<Vec<_>>()
                .as_slice(),
            triangles,
        );

        let mut interleaved = Vec::with_capacity(positions.len());
        for (p, n) in positions.iter().zip(normals.iter()) {
            interleaved.push([p.x, p.y, p.z, n.x, n.y, n.z]);
        }

        let mut idx = Vec::with_capacity(triangles.len() * 3);
        for tri in triangles {
            idx.extend_from_slice(tri);
        }

        Mesh::from_interleaved(gl, &interleaved, &idx)
    }

    pub fn from_obj(gl: &glow::Context, obj: &obj::Obj) -> Self {
        let positions = &obj.vertices;
        let triangles = &obj.indices;

        let normals = Self::smooth_normals(
            positions
                .iter()
                .map(|v| [v.position[0], v.position[1], v.position[2]])
                .collect::<Vec<_>>()
                .as_slice(),
            &triangles
                .chunks(3)
                .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
                .collect::<Vec<_>>(),
        );

        let mut interleaved = Vec::with_capacity(positions.len());
        for (p, n) in positions.iter().zip(normals.iter()) {
            interleaved.push([p.position[0], p.position[1], p.position[2], n.x, n.y, n.z]);
        }

        Mesh::from_interleaved(
            gl,
            &interleaved,
            triangles
                .iter()
                .map(|i| *i as u32)
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }

    pub fn from_poly_navmesh(gl: &glow::Context, poly_navmesh: &PolygonNavmesh) -> Self {
        let positions = poly_navmesh
            .vertices
            .iter()
            .map(|v| Vec3 {
                x: poly_navmesh.aabb.min.x + v.x as f32 * poly_navmesh.cell_size,
                y: poly_navmesh.aabb.min.y + v.y as f32 * poly_navmesh.cell_height,
                z: poly_navmesh.aabb.min.z + v.z as f32 * poly_navmesh.cell_size,
            })
            .collect::<Vec<_>>();
        let triangles = poly_navmesh
            .polygons
            .chunks(poly_navmesh.max_vertices_per_polygon.into())
            .map(|poly| {
                if poly.len() != 3 {
                    todo!("Handle non-triangular polygons");
                }

                let mut tris = Vec::new();
                for i in 1..(poly.len() - 1) {
                    tris.push([poly[0] as u32, poly[i] as u32, poly[i + 1] as u32]);
                }
                tris
            })
            .flatten()
            .collect::<Vec<_>>();

        let normals = Self::smooth_normals(
            positions
                .iter()
                .map(|v| [v.x, v.y, v.z])
                .collect::<Vec<_>>()
                .as_slice(),
            triangles
                .iter()
                .map(|tri| [tri[0], tri[1], tri[2]])
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let mut interleaved = Vec::with_capacity(positions.len());
        for (p, n) in positions.iter().zip(normals.iter()) {
            interleaved.push([p.x, p.y, p.z, n.x, n.y, n.z]);
        }

        let mut idx = Vec::with_capacity(triangles.len() * 3);
        for tri in triangles {
            idx.extend_from_slice(&[tri[0], tri[1], tri[2]]);
        }

        Mesh::from_interleaved(gl, &interleaved, &idx)
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }

    pub fn draw(
        &self,
        gl: &glow::Context,
        prog: &BasicProgram,
        model: &na::Matrix4<f32>,
        color: [f32; 4],
        wireframe: &Wireframe,
    ) {
        unsafe {
            gl.use_program(Some(prog.program));
            if let Some(loc) = &prog.loc_model {
                gl.uniform_matrix_4_f32_slice(Some(loc), false, model.as_slice());
            }
            if let Some(loc) = &prog.loc_color {
                gl.uniform_4_f32(Some(loc), color[0], color[1], color[2], color[3]);
            }
            gl.bind_vertex_array(Some(self.vao));

            match wireframe {
                Wireframe::None => {
                    if let Some(loc) = &prog.loc_mode {
                        gl.uniform_1_i32(Some(loc), 0); // lit
                    }
                    gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
                }

                Wireframe::Only { color, width } => {
                    // flat, polygon lines
                    if let Some(loc) = &prog.loc_mode {
                        gl.uniform_1_i32(Some(loc), 1);
                    }
                    if let Some(loc) = &prog.loc_color {
                        gl.uniform_4_f32(Some(loc), color[0], color[1], color[2], color[3]);
                    }
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                    gl.line_width(*width);
                    gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                }

                Wireframe::Overlay { color, width } => {
                    // pass 1: filled (lit)
                    if let Some(loc) = &prog.loc_mode {
                        gl.uniform_1_i32(Some(loc), 0);
                    }
                    gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);

                    // pass 2: lines (flat color) with depth offset so edges sit on top
                    if let Some(loc) = &prog.loc_mode {
                        gl.uniform_1_i32(Some(loc), 1);
                    }
                    if let Some(loc) = &prog.loc_color {
                        gl.uniform_4_f32(Some(loc), color[0], color[1], color[2], color[3]);
                    }

                    gl.enable(glow::POLYGON_OFFSET_LINE);
                    // negative offset pulls lines slightly toward the camera
                    gl.polygon_offset(-1.0, -1.0);

                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                    gl.line_width(*width);
                    gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);

                    // restore state
                    gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                    gl.disable(glow::POLYGON_OFFSET_LINE);
                }
            }

            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

pub enum Wireframe {
    None,
    /// Draw only edges
    Only {
        color: [f32; 4],
        width: f32,
    },
    /// Draw filled, then edge overlay with depth offset (avoids z-fighting)
    Overlay {
        color: [f32; 4],
        width: f32,
    },
}

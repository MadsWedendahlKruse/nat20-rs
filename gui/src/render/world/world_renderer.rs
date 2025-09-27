// gui/src/render/world_renderer.rs
use glow::HasContext;
use parry3d::{na, shape::TriMesh};

use crate::render::world::program::BasicProgram;

pub struct WorldRenderer {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
}

impl WorldRenderer {
    pub fn new(gl: &glow::Context, mesh: &TriMesh) -> Self {
        // --- Compile shader program ---
        let vert_src = r#"#version 330 core
        layout (location = 0) in vec3 a_pos;
        layout (location = 1) in vec3 a_nrm;
        uniform mat4 u_mvp;
        uniform mat4 u_model;
        out vec3 v_nrm;
        void main() {
            gl_Position = u_mvp * vec4(a_pos, 1.0);
            // normal in world space (assuming u_model has no non-uniform scale)
            v_nrm = mat3(u_model) * a_nrm;
        }"#;

        let frag_src = r#"#version 330 core
        in vec3 v_nrm;
        uniform vec3 u_light_dir;
        out vec4 FragColor;
        void main() {
            float NdotL = max(dot(normalize(v_nrm), normalize(-u_light_dir)), 0.0);
            vec3 base = vec3(0.6, 0.7, 0.8);
            vec3 color = base * (0.2 + 0.8 * NdotL);
            FragColor = vec4(color, 1.0);
        }"#;

        unsafe fn compile(gl: &glow::Context, ty: u32, src: &str) -> glow::Shader {
            let sh = gl.create_shader(ty).unwrap();
            gl.shader_source(sh, src);
            gl.compile_shader(sh);
            if !gl.get_shader_compile_status(sh) {
                panic!("shader error: {}", gl.get_shader_info_log(sh));
            }
            sh
        }

        let program = unsafe {
            let vs = compile(gl, glow::VERTEX_SHADER, vert_src);
            let fs = compile(gl, glow::FRAGMENT_SHADER, frag_src);
            let prog = gl.create_program().unwrap();
            gl.attach_shader(prog, vs);
            gl.attach_shader(prog, fs);
            gl.link_program(prog);
            if !gl.get_program_link_status(prog) {
                panic!("link error: {}", gl.get_program_info_log(prog));
            }
            gl.delete_shader(vs);
            gl.delete_shader(fs);
            prog
        };

        // --- Build CPU buffers (positions + normals interleaved) ---
        // Parry TriMesh exposes vertices() -> &[Point<Real>] and indices() -> &[[u32;3]]
        let positions = mesh.vertices();
        let triangles = mesh.indices();

        // Compute smooth vertex normals
        let mut normals = vec![na::Vector3::<f32>::zeros(); positions.len()];
        for tri in triangles.iter() {
            let ia = tri[0] as usize;
            let ib = tri[1] as usize;
            let ic = tri[2] as usize;
            let a = positions[ia].coords; // Vector3
            let b = positions[ib].coords;
            let c = positions[ic].coords;
            let n = (b - a).cross(&(c - a));
            // area-weighted accumulation
            normals[ia] += n;
            normals[ib] += n;
            normals[ic] += n;
        }
        for n in &mut normals {
            let len = n.norm();
            if len > 1e-6 {
                *n /= len;
            } else {
                *n = na::Vector3::y(); // fallback normal
            }
        }

        // Interleave: [px,py,pz, nx,ny,nz] per vertex
        let mut interleaved: Vec<f32> = Vec::with_capacity(positions.len() * 6);
        for (p, n) in positions.iter().zip(normals.iter()) {
            interleaved.extend_from_slice(&[p.x, p.y, p.z, n.x, n.y, n.z]);
        }

        // Flatten indices
        let mut flat_indices: Vec<u32> = Vec::with_capacity(triangles.len() * 3);
        for tri in triangles.iter() {
            flat_indices.extend_from_slice(tri);
        }

        // --- Upload to GPU ---
        let (vao, vbo, ebo);
        unsafe {
            vao = gl.create_vertex_array().unwrap();
            vbo = gl.create_buffer().unwrap();
            ebo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&interleaved),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&flat_indices),
                glow::STATIC_DRAW,
            );

            // layout(location=0) position
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, (6 * 4) as i32, 0);

            // layout(location=1) normal
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, (6 * 4) as i32, (3 * 4) as i32);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            // keep EBO bound to VAO; no need to unbind ELEMENT_ARRAY_BUFFER
        }

        Self {
            program,
            vao,
            vbo,
            ebo,
            index_count: flat_indices.len() as i32,
        }
    }

    pub fn draw(&self, gl: &glow::Context, prog: &BasicProgram) {
        unsafe {
            gl.use_program(Some(prog.program));
            if let Some(loc) = &prog.loc_model {
                let model = parry3d::na::Matrix4::<f32>::identity();
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
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }
}

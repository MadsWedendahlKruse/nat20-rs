use glam::{Mat4, Vec3};
use glow::HasContext;
use std::fs;

pub struct Cube {
    pub program: glow::Program,
    pub vao: glow::VertexArray,
    pub rotation: Vec3,
}

static SHADER_PATH: &str = "D:\\nat20-rs\\gui\\shaders\\";

impl Cube {
    pub fn new(gl: &glow::Context) -> Self {
        let vert = fs::read_to_string(format!("{}cube.vert", SHADER_PATH)).unwrap();
        let frag = fs::read_to_string(format!("{}cube.frag", SHADER_PATH)).unwrap();
        let program = unsafe {
            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, &vert);
            gl.compile_shader(vs);
            assert!(
                gl.get_shader_compile_status(vs),
                "Vertex shader error: {}",
                gl.get_shader_info_log(vs)
            );

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, &frag);
            gl.compile_shader(fs);
            assert!(
                gl.get_shader_compile_status(fs),
                "Fragment shader error: {}",
                gl.get_shader_info_log(fs)
            );

            let prog = gl.create_program().unwrap();
            gl.attach_shader(prog, vs);
            gl.attach_shader(prog, fs);
            gl.link_program(prog);
            assert!(
                gl.get_program_link_status(prog),
                "Link error: {}",
                gl.get_program_info_log(prog)
            );

            gl.delete_shader(vs);
            gl.delete_shader(fs);
            prog
        };

        // 24 vertices: [position.x, y, z, normal.x, y, z]
        let vertices: [f32; 6 * 24] = [
            // -Z face
            -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, 0.5, -0.5, 0.0,
            0.0, -1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, // +Z face
            -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0,
            1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, // -X face
            -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, 0.5, -0.5,
            -1.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, // +X face
            0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
            0.0, 0.5, 0.5, -0.5, 1.0, 0.0, 0.0, // -Y face
            -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.5, -0.5, -0.5, 0.0,
            -1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, // +Y face
            -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0,
            0.0, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0,
        ];

        let indices: [u32; 36] = [
            0, 1, 2, 2, 3, 0, // -Z
            4, 5, 6, 6, 7, 4, // +Z
            8, 9, 10, 10, 11, 8, // -X
            12, 13, 14, 14, 15, 12, // +X
            16, 17, 18, 18, 19, 16, // -Y
            20, 21, 22, 22, 23, 20, // +Y
        ];

        let vao;
        unsafe {
            vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices),
                glow::STATIC_DRAW,
            );

            // Attribute 0: position (3 floats)
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 6 * 4, 0);

            // Attribute 1: normal (3 floats)
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 6 * 4, 3 * 4);
        }

        Self {
            program,
            vao,
            rotation: Vec3::ZERO,
        }
    }

    pub fn draw(&self, gl: &glow::Context, aspect_ratio: f32) {
        // Clear color and depth buffer
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let model = Mat4::from_rotation_y(self.rotation.y) * Mat4::from_rotation_x(self.rotation.x);
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.5), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);
        let mvp = proj * view * model;
        let light_dir = Vec3::new(-0.5, -1.0, -1.0);

        unsafe {
            gl.use_program(Some(self.program));

            let loc_mvp = gl.get_uniform_location(self.program, "u_mvp");
            let loc_model = gl.get_uniform_location(self.program, "u_model");
            let loc_light = gl.get_uniform_location(self.program, "u_light_dir");

            if let Some(loc) = loc_mvp {
                gl.uniform_matrix_4_f32_slice(Some(&loc), false, mvp.to_cols_array().as_slice());
            }
            if let Some(loc) = loc_model {
                gl.uniform_matrix_4_f32_slice(Some(&loc), false, model.to_cols_array().as_slice());
            }
            if let Some(loc) = loc_light {
                gl.uniform_3_f32_slice(Some(&loc), &light_dir.to_array());
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, 36, glow::UNSIGNED_INT, 0);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
        }
    }
}

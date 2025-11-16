// render/line_renderer.rs
use glow::HasContext;
use parry3d::na;

#[repr(C)]
#[derive(Clone, Copy)]
struct LineVertex {
    pos: [f32; 3],
    col: [f32; 3],
}
unsafe impl bytemuck::Zeroable for LineVertex {}
unsafe impl bytemuck::Pod for LineVertex {}

#[derive(Clone, Copy)]
pub enum LineMode {
    Lines,     // GL_LINES (pairs)
    LineStrip, // GL_LINE_STRIP (polyline)
    LineLoop,  // GL_LINE_LOOP (closed poly)
}

struct DrawRange {
    mode: LineMode,
    first: i32,
    count: i32,
}

pub struct LineProgram {
    pub program: glow::Program,
    pub loc_model: Option<glow::UniformLocation>,
}

impl LineProgram {
    pub fn new(gl: &glow::Context, vert_src: &str, frag_src: &str) -> Self {
        unsafe {
            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, vert_src);
            gl.compile_shader(vs);
            assert!(
                gl.get_shader_compile_status(vs),
                "VS: {}",
                gl.get_shader_info_log(vs)
            );

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, frag_src);
            gl.compile_shader(fs);
            assert!(
                gl.get_shader_compile_status(fs),
                "FS: {}",
                gl.get_shader_info_log(fs)
            );

            let program = gl.create_program().unwrap();
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "Link: {}",
                gl.get_program_info_log(program)
            );
            gl.delete_shader(vs);
            gl.delete_shader(fs);

            // Bind the Frame block to binding=0 to match your UBO
            let block = gl
                .get_uniform_block_index(program, "Frame")
                .expect("no 'Frame' uniform block");
            gl.uniform_block_binding(program, block, 0);

            let loc_model = gl.get_uniform_location(program, "u_model");
            Self { program, loc_model }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe { gl.delete_program(self.program) }
    }
}

pub struct LineRenderer {
    program: LineProgram,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    capacity: i32, // vertex capacity
    verts: Vec<LineVertex>,
    draws: Vec<DrawRange>,
}

impl LineRenderer {
    pub fn new(gl: &glow::Context, vert_src: &str, frag_src: &str) -> Self {
        let program = LineProgram::new(gl, vert_src, frag_src);
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            // start with a small buffer; will grow as needed
            gl.buffer_data_size(glow::ARRAY_BUFFER, 1024, glow::DYNAMIC_DRAW);

            // a_pos @ loc 0
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<LineVertex>() as i32,
                0,
            );
            // a_col @ loc 1
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<LineVertex>() as i32,
                (3 * 4) as i32,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Self {
                program,
                vao,
                vbo,
                capacity: 1024 / std::mem::size_of::<LineVertex>() as i32,
                verts: Vec::new(),
                draws: Vec::new(),
            }
        }
    }

    pub fn clear(&mut self) {
        self.verts.clear();
        self.draws.clear();
    }

    #[inline]
    fn push_range(&mut self, mode: LineMode, first: i32, count: i32) {
        if count > 0 {
            self.draws.push(DrawRange { mode, first, count });
        }
    }

    pub fn add_line(&mut self, a: [f32; 3], b: [f32; 3], col: [f32; 3]) {
        let first = self.verts.len() as i32;
        self.verts.push(LineVertex { pos: a, col });
        self.verts.push(LineVertex { pos: b, col });
        self.push_range(LineMode::Lines, first, 2);
    }

    pub fn add_polyline(&mut self, points: &[[f32; 3]], col: [f32; 3]) {
        if points.len() < 2 {
            return;
        }
        let first = self.verts.len() as i32;
        for &p in points {
            self.verts.push(LineVertex { pos: p, col });
        }
        self.push_range(LineMode::LineStrip, first, points.len() as i32);
    }

    pub fn add_polyline_color(&mut self, points: &[[f32; 3]], colors: &[[f32; 3]]) {
        if points.len() < 2 || points.len() != colors.len() {
            return;
        }
        let first = self.verts.len() as i32;
        for (p, c) in points.iter().zip(colors.iter()) {
            self.verts.push(LineVertex { pos: *p, col: *c });
        }
        self.push_range(LineMode::LineStrip, first, points.len() as i32);
    }

    pub fn add_loop(&mut self, points: &[[f32; 3]], col: [f32; 3]) {
        if points.len() < 2 {
            return;
        }
        let first = self.verts.len() as i32;
        for &p in points {
            self.verts.push(LineVertex { pos: p, col });
        }
        self.push_range(LineMode::LineLoop, first, points.len() as i32);
    }

    pub fn add_circle(&mut self, center: [f32; 3], radius: f32, col: [f32; 3]) {
        let segments = 32;
        let mut points = Vec::with_capacity(segments);
        for i in 0..segments {
            let theta = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let x = center[0] + radius * theta.cos();
            let z = center[2] + radius * theta.sin();
            points.push([x, center[1], z]);
        }
        self.add_loop(&points, col);
    }

    pub fn add_ray(&mut self, origin: [f32; 3], dir: [f32; 3], t: f32, col: [f32; 3]) {
        let b = [
            origin[0] + dir[0] * t,
            origin[1] + dir[1] * t,
            origin[2] + dir[2] * t,
        ];
        self.add_line(origin, b, col);
    }

    pub fn add_parabola(
        &mut self,
        start: [f32; 3],
        velocity: [f32; 3],
        steps: usize,
        col: [f32; 3],
    ) {
        if steps < 2 {
            return;
        }
        let mut points = Vec::with_capacity(steps);
        for i in 0..steps {
            let t = i as f32 / (steps - 1) as f32;
            let x = start[0] + velocity[0] * t;
            // Gravity could be a parameter, but seems unnecessary for now
            let y = start[1] + velocity[1] * t - 0.5 * 9.81 * t * t;
            let z = start[2] + velocity[2] * t;
            points.push([x, y, z]);
        }
        self.add_polyline(&points, col);
    }

    /// Upload & draw everything in the batch.
    /// `model` lets you draw in a local space (pass identity for world-space lines).
    pub fn draw(&mut self, gl: &glow::Context, model: &na::Matrix4<f32>, line_width: f32) {
        // grow buffer if needed
        let required = self.verts.len() as i32;
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            if required > self.capacity {
                let new_capacity = (required * 2).max(1024);
                gl.buffer_data_size(
                    glow::ARRAY_BUFFER,
                    new_capacity as i32 * std::mem::size_of::<LineVertex>() as i32,
                    glow::DYNAMIC_DRAW,
                );
                self.capacity = new_capacity;
            }
            gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, bytemuck::cast_slice(&self.verts));
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            gl.use_program(Some(self.program.program));
            if let Some(loc) = &self.program.loc_model {
                gl.uniform_matrix_4_f32_slice(Some(loc), false, model.as_slice());
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.line_width(line_width);

            for r in &self.draws {
                let mode = match r.mode {
                    LineMode::Lines => glow::LINES,
                    LineMode::LineStrip => glow::LINE_STRIP,
                    LineMode::LineLoop => glow::LINE_LOOP,
                };
                gl.draw_arrays(mode, r.first, r.count);
            }

            gl.bind_vertex_array(None);
            gl.use_program(None);
        }

        // usually you clear after drawing (one-shot debug batch)
        self.clear();
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
        self.program.destroy(gl);
    }
}

// render/basic_program.rs
use glow::HasContext;

pub struct BasicProgram {
    pub program: glow::Program,
    pub loc_model: Option<glow::UniformLocation>,
}
impl BasicProgram {
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
            let prog = gl.create_program().unwrap();
            gl.attach_shader(prog, vs);
            gl.attach_shader(prog, fs);
            gl.link_program(prog);
            assert!(
                gl.get_program_link_status(prog),
                "Link: {}",
                gl.get_program_info_log(prog)
            );
            gl.delete_shader(vs);
            gl.delete_shader(fs);

            // (Optional) explicitly bind the "Frame" block to 0 if not using layout(binding=0)
            // let block = gl.get_uniform_block_index(prog, "Frame");
            // gl.uniform_block_binding(prog, block, 0);

            let loc_model = gl.get_uniform_location(prog, "u_model");
            Self {
                program: prog,
                loc_model,
            }
        }
    }
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe { gl.delete_program(self.program) }
    }
}

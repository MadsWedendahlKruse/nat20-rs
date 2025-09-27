// render/frame.rs
use glow::HasContext;
use parry3d::na;

#[repr(C)] // std140-friendly packing
#[derive(Clone, Copy)]
pub struct FrameStd140 {
    pub view_proj: [[f32; 4]; 4],
    pub light_dir: [f32; 4], // pad to vec4 for std140
}
unsafe impl bytemuck::Zeroable for FrameStd140 {}
unsafe impl bytemuck::Pod for FrameStd140 {}

pub struct FrameUniforms {
    ubo: glow::Buffer,
    binding_index: u32, // e.g. 0
}

impl FrameUniforms {
    pub fn new(gl: &glow::Context, binding_index: u32) -> Self {
        let ubo = unsafe { gl.create_buffer().unwrap() };
        unsafe {
            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(ubo));
            gl.buffer_data_size(glow::UNIFORM_BUFFER, 128, glow::DYNAMIC_DRAW); // enough for our block
            gl.bind_buffer(glow::UNIFORM_BUFFER, None);
            gl.bind_buffer_base(glow::UNIFORM_BUFFER, binding_index, Some(ubo));
        }
        Self { ubo, binding_index }
    }

    pub fn update(
        &self,
        gl: &glow::Context,
        view: na::Isometry3<f32>,
        proj: na::Perspective3<f32>,
        light_dir: na::Vector3<f32>,
    ) {
        let vp = proj.to_homogeneous() * view.to_homogeneous();
        let data = FrameStd140 {
            view_proj: vp.into(),
            light_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
        };
        unsafe {
            gl.bind_buffer(glow::UNIFORM_BUFFER, Some(self.ubo));
            gl.buffer_sub_data_u8_slice(glow::UNIFORM_BUFFER, 0, bytemuck::bytes_of(&data));
            gl.bind_buffer(glow::UNIFORM_BUFFER, None);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe { gl.delete_buffer(self.ubo) }
    }
}

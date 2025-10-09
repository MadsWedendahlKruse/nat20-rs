use std::collections::BTreeMap;

use glow::HasContext;
use imgui_glow_renderer::AutoRenderer;
use nat20_rs::systems::geometry::RaycastResult;
use parry3d::{na::Vector3, query::Ray};
use winit::window::Window;

use crate::{
    render::{
        ui::utils::ImguiRenderableMut,
        world::{
            camera::OrbitCamera, frame_uniforms::FrameUniforms, mesh::Mesh, program::BasicProgram,
        },
    },
    state::settings::GuiSettings,
};

pub struct GuiState {
    pub ig_renderer: AutoRenderer,
    pub frame_uniforms: FrameUniforms,
    pub program: BasicProgram,
    pub camera: OrbitCamera,
    pub settings: GuiSettings,

    // TODO: Everything involving the mesh cache seems like a mess right now.
    pub mesh_cache: BTreeMap<String, Mesh>,

    /// The result of a raycast from the cursor into the 3D world.
    /// This is updated every frame, and can be used by various UI components
    /// to determine what the cursor is pointing at.
    ///
    /// To avoid multiple UI components using the same raycast result, this is
    /// an `Option`. UI components that use it should `.take()` it, so that other
    /// components know it has already been used.
    ///
    /// Note that this will also be `None` if the cursor didn't hit anything in
    /// the 3D world.
    pub cursor_ray_result: Option<RaycastResult>,
}

impl GuiState {
    pub fn new(gl: glow::Context, imgui_context: &mut imgui::Context) -> Self {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.enable(glow::BLEND);
        }

        let ig_renderer =
            AutoRenderer::new(gl, imgui_context).expect("failed to create imgui OpenGL renderer");
        let frame_uniforms = FrameUniforms::new(ig_renderer.gl_context(), 0);
        let program = BasicProgram::new(
            ig_renderer.gl_context(),
            include_str!("../render/world/shaders/basic.vert"),
            include_str!("../render/world/shaders/basic.frag"),
        );

        Self {
            ig_renderer,
            frame_uniforms,
            program,
            camera: OrbitCamera::new(),
            settings: GuiSettings::default(),
            mesh_cache: BTreeMap::new(),
            cursor_ray_result: None,
        }
    }

    pub fn gl_context(&self) -> &glow::Context {
        self.ig_renderer.gl_context()
    }

    pub fn new_frame(&mut self, window: &Window) {
        let gl = self.ig_renderer.gl_context();
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let size = window.inner_size();
        unsafe {
            gl.viewport(0, 0, size.width as i32, size.height as i32);
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let view = self.camera.view();
        let proj = self.camera.proj(size.width, size.height);
        let light_dir = Vector3::new(-0.5, -1.0, -0.8);
        self.frame_uniforms.update(gl, view, proj, light_dir);
    }
}

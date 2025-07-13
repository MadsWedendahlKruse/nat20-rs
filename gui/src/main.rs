use std::{num::NonZeroU32, time::Instant};

mod render;
mod state;
mod utils;

use glow::HasContext;
use glutin::surface::GlSurface;
use imgui::{Condition, TreeNodeFlags};
use nat20_rs::{
    engine::world::World,
    stats::{ability::Ability, d20_check::RollMode, proficiency::Proficiency, skill::Skill},
    test_utils::fixtures,
};
use strum::IntoEnumIterator;

use crate::state::GuiState;

fn main() {
    let (event_loop, window, surface, context) = utils::create_window("Hello, triangle!", None);
    let (mut winit_platform, mut imgui_context) = utils::imgui_init(&window);
    let gl = utils::glow_context(&context);

    unsafe {
        gl.enable(glow::DEPTH_TEST);
    }

    let mut ig_renderer = imgui_glow_renderer::AutoRenderer::new(gl, &mut imgui_context)
        .expect("failed to create renderer");

    let mut last_frame = Instant::now();

    let mut gui_state = GuiState::new(World::new());

    #[allow(deprecated)]
    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui_context
                        .io_mut()
                        .update_delta_time(now.duration_since(last_frame));
                    last_frame = now;
                }

                winit::event::Event::AboutToWait => {
                    winit_platform
                        .prepare_frame(imgui_context.io_mut(), &window)
                        .unwrap();

                    window.request_redraw();
                }

                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::RedrawRequested,
                    ..
                } => {
                    let gl = ig_renderer.gl_context();
                    unsafe {
                        gl.clear_color(0.05, 0.05, 0.1, 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);
                    }

                    let ui = imgui_context.frame();
                    ui.show_demo_window(&mut true);

                    gui_state.render(&ui);

                    winit_platform.prepare_render(ui, &window);
                    let draw_data = imgui_context.render();

                    // Render imgui on top of it
                    ig_renderer
                        .render(draw_data)
                        .expect("error rendering imgui");

                    surface
                        .swap_buffers(&context)
                        .expect("Failed to swap buffers");
                }

                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    window_target.exit();
                }

                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(new_size),
                    ..
                } => {
                    if new_size.width > 0 && new_size.height > 0 {
                        surface.resize(
                            &context,
                            NonZeroU32::new(new_size.width).unwrap(),
                            NonZeroU32::new(new_size.height).unwrap(),
                        );
                    }
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }

                winit::event::Event::LoopExiting => {}

                event => {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }
            }
        })
        .expect("EventLoop error");
}

use std::{num::NonZeroU32, time::Instant};

mod render;
mod utils;
mod windows;

use glow::HasContext;
use glutin::surface::GlSurface;
use parry3d::na::Vector3;

use crate::{
    render::world::{camera::OrbitCamera, frame_uniforms::FrameUniforms, program::BasicProgram},
    windows::main_menu::MainMenuWindow,
};

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

    let frame_uniforms = FrameUniforms::new(ig_renderer.gl_context(), 0);
    let program = BasicProgram::new(
        ig_renderer.gl_context(),
        include_str!("render/world/shaders/basic.vert"),
        include_str!("render/world/shaders/basic.frag"),
    );
    // TODO: Where should the camera live?
    let mut camera = OrbitCamera::new();

    let mut main_menu = MainMenuWindow::new(&ig_renderer.gl_context());

    #[allow(deprecated)]
    event_loop
        .run(move |event, window_target| match event {
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
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }

                let size = window.inner_size();
                unsafe {
                    gl.viewport(0, 0, size.width as i32, size.height as i32);
                    gl.clear_color(0.05, 0.05, 0.1, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                }

                let view = camera.view();
                let proj = camera.proj(size.width, size.height);
                let light_dir = Vector3::new(-0.5, -1.0, -0.8);
                frame_uniforms.update(gl, view, proj, light_dir);

                let ui = imgui_context.frame();

                main_menu.render(ui, gl, &program, &mut camera);

                // ui.show_demo_window(&mut true);

                winit_platform.prepare_render(ui, &window);
                let draw_data = imgui_context.render();

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
                    unsafe {
                        ig_renderer.gl_context().viewport(
                            0,
                            0,
                            new_size.width as i32,
                            new_size.height as i32,
                        );
                    }
                }
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }

            // winit::event::Event::WindowEvent {
            //     event: winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. },
            //     ..
            // } => {
            //     // new_inner_size is &mut; read and resize the surface + viewport
            //     let (w, h) = (new_inner_size.width, new_inner_size.height);
            //     if w > 0 && h > 0 {
            //         surface.resize(
            //             &context,
            //             NonZeroU32::new(w).unwrap(),
            //             NonZeroU32::new(h).unwrap(),
            //         );
            //         unsafe {
            //             gl.viewport(0, 0, w as i32, h as i32);
            //         }
            //     }
            //     winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            // }
            winit::event::Event::WindowEvent {
                event: ref window_event,
                ..
            } => {
                let wants_mouse = imgui_context.io().want_capture_mouse;
                camera.handle_event(&window_event, wants_mouse);
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }

            winit::event::Event::LoopExiting => {}

            event => {
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
        })
        .expect("EventLoop error");
}

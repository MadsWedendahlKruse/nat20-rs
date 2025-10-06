use std::{collections::BTreeMap, num::NonZeroU32, time::Instant};

mod render;
mod state;
mod utils;
mod windows;

use glow::HasContext;
use glutin::surface::GlSurface;
use parry3d::na::Vector3;

use crate::{
    render::{
        ui::utils::ImguiRenderableMut,
        world::{camera::OrbitCamera, frame_uniforms::FrameUniforms, program::BasicProgram},
    },
    state::{
        gui_state::GuiState,
        settings::{GuiSettings, Setting},
    },
    windows::main_menu::MainMenuWindow,
};

fn main() {
    let (event_loop, window, surface, context) = utils::create_window("Hello, triangle!", None);
    let (mut winit_platform, mut imgui_context) = utils::imgui_init(&window);
    let gl = utils::glow_context(&context);

    let mut gui_state = GuiState::new(gl, &mut imgui_context);
    let mut show_settings = false;

    let mut last_frame = Instant::now();

    let mut main_menu = MainMenuWindow::new(gui_state.gl_context());

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
                let ui = imgui_context.frame();

                gui_state.new_frame(&window);

                let prev_show_settings = show_settings;

                if ui.is_key_pressed(imgui::Key::Escape) {
                    show_settings = !show_settings;
                }

                if show_settings {
                    ui.window("Settings")
                        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
                        .size([0.0, 300.0], imgui::Condition::Always)
                        .opened(&mut show_settings)
                        .focus_on_appearing(true)
                        .build(|| {
                            if !prev_show_settings {
                                ui.set_keyboard_focus_here();
                            }
                            gui_state.settings.render_mut(&ui);
                        });
                }

                main_menu.render(ui, &mut gui_state);

                let show_demo = gui_state
                    .settings
                    .get_bool(state::parameters::RENDER_IMGUI_DEMO);
                if *show_demo {
                    ui.show_demo_window(show_demo);
                }

                winit_platform.prepare_render(ui, &window);
                let draw_data = imgui_context.render();

                gui_state
                    .ig_renderer
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
                        gui_state.gl_context().viewport(
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
                gui_state.camera.handle_event(&window_event, wants_mouse);
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }

            winit::event::Event::LoopExiting => {}

            event => {
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
        })
        .expect("EventLoop error");
}

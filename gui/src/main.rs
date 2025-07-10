use std::{num::NonZeroU32, time::Instant};

mod utils;

use glutin::surface::GlSurface;
use imgui::{Condition, TreeNodeFlags};
use nat20_rs::{
    stats::{ability::Ability, d20_check::RollMode, proficiency::Proficiency, skill::Skill},
    test_utils::fixtures,
};
use strum::IntoEnumIterator;
use utils::Triangler;

fn main() {
    let (event_loop, window, surface, context) = utils::create_window("Hello, triangle!", None);
    let (mut winit_platform, mut imgui_context) = utils::imgui_init(&window);
    let gl = utils::glow_context(&context);

    let mut ig_renderer = imgui_glow_renderer::AutoRenderer::new(gl, &mut imgui_context)
        .expect("failed to create renderer");
    let tri_renderer = Triangler::new(ig_renderer.gl_context(), "#version 330");

    let mut last_frame = Instant::now();

    let character = fixtures::creatures::heroes::fighter();

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
                    // Render your custom scene, note we need to borrow the OpenGL
                    // context from the `AutoRenderer`, which takes ownership of it.
                    tri_renderer.render(ig_renderer.gl_context());

                    let ui = imgui_context.frame();
                    ui.show_demo_window(&mut true);

                    ui.window(character.name())
                        .size([300.0, 1200.0], Condition::FirstUseEver)
                        .build(|| {
                            ui.text(format!("ID: {}", character.id()));
                            ui.text(format!("Level: {}", character.total_level()));
                            ui.text(format!("HP: {}/{}", character.hp(), character.max_hp()));

                            if ui.collapsing_header("Classes", TreeNodeFlags::DEFAULT_OPEN) {
                                for (class_name, level) in character.classes() {
                                    if let Some(subclass_name) = character.subclass(class_name) {
                                        ui.text(format!(
                                            "Level {} {} {}",
                                            level, subclass_name.name, class_name
                                        ));
                                    } else {
                                        ui.text(format!("Level {} {}", level, class_name));
                                    }
                                }
                            }

                            if ui.collapsing_header("Ability Scores", TreeNodeFlags::DEFAULT_OPEN) {
                                for (_, ability_score) in character.ability_scores().scores.iter() {
                                    ui.text(format!("{}", ability_score));
                                }
                            }

                            if ui.collapsing_header("Skills", TreeNodeFlags::DEFAULT_OPEN) {
                                for skill in Skill::iter() {
                                    let stats = character.skills().get(skill);
                                    if stats.modifiers().is_empty()
                                        && stats.advantage_tracker().roll_mode() == RollMode::Normal
                                        && *stats.proficiency() == Proficiency::None
                                    {
                                        continue; // Skip skills with no modifiers
                                    }
                                    ui.text(format!(
                                        "{}: {}",
                                        skill,
                                        stats.format_bonus(character.proficiency_bonus())
                                    ));
                                }
                            }

                            if ui.collapsing_header("Saving Throws", TreeNodeFlags::DEFAULT_OPEN) {
                                for ability in Ability::iter() {
                                    let stats = character.saving_throws().get(ability);
                                    if stats.modifiers().is_empty()
                                        && stats.advantage_tracker().roll_mode() == RollMode::Normal
                                        && *stats.proficiency() == Proficiency::None
                                    {
                                        continue; // Skip saving throws with no modifiers
                                    }
                                    ui.text(format!(
                                        "{}: {}",
                                        ability,
                                        stats.format_bonus(character.proficiency_bonus())
                                    ));
                                }
                            }

                            if ui.collapsing_header("Resources", TreeNodeFlags::DEFAULT_OPEN) {
                                for (resource_id, resource) in character.resources().iter() {
                                    ui.text(format!(
                                        "{}: {}/{}",
                                        resource_id,
                                        resource.current_uses(),
                                        resource.max_uses()
                                    ));
                                }
                            }

                            if ui.collapsing_header("Spell slots", TreeNodeFlags::DEFAULT_OPEN) {
                                if character.spellbook().spell_slots().is_empty() {
                                    ui.text("No spell slots available\n");
                                }
                                for (level, (current_slots, max_slots)) in
                                    character.spellbook().spell_slots().iter()
                                {
                                    ui.text(format!(
                                        "Level {}: ({}/{})\n",
                                        level, current_slots, max_slots
                                    ));
                                }
                            }

                            if ui.collapsing_header("Effects", TreeNodeFlags::DEFAULT_OPEN) {
                                for effect in character.effects() {
                                    ui.text(format!("{} ({})", effect.id(), effect.duration()));
                                }
                            }

                            // ui.separator();
                            // let mouse_pos = ui.io().mouse_pos;
                            // ui.text(format!(
                            //     "Mouse Position: ({:.1},{:.1})",
                            //     mouse_pos[0], mouse_pos[1]
                            // ));
                        });

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

                winit::event::Event::LoopExiting => {
                    let gl = ig_renderer.gl_context();
                    tri_renderer.destroy(gl);
                }
                event => {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }
            }
        })
        .expect("EventLoop error");
}

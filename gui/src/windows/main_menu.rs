use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use glam::{Vec3, Vec3Swizzles};
use imgui::{ChildFlags, MouseButton, WindowFlags, sys};
use nat20_rs::{
    components::{
        health::{hit_points::HitPoints, life_state::LifeState},
        id::Name,
        race::CreatureSize,
    },
    engine::{encounter::EncounterId, game_state::GameState, geometry::WorldGeometry},
    systems::{
        self,
        geometry::{CreaturePose, RaycastHitKind, RaycastResult},
    },
};
use parry3d::na::{self, Matrix4, Point3};
use strum::IntoEnumIterator;

use crate::{
    render::{
        ui::{
            engine::LogLevel,
            entities::render_if_present,
            utils::{
                ImguiRenderable, ImguiRenderableMutWithContext, ImguiRenderableWithContext,
                render_button_disabled_conditionally, render_uniform_buttons_with_padding,
                render_window_at_cursor,
            },
        },
        world::{
            camera::OrbitCamera,
            frame_uniforms::FrameUniforms,
            grid::GridRenderer,
            line::LineRenderer,
            mesh::{self, Mesh, Wireframe},
            program::BasicProgram,
            shapes::{self, CapsuleCache},
        },
    },
    state::{
        self,
        gui_state::{self, GuiState},
    },
    windows::{
        creature_debug::CreatureDebugWindow, creature_right_click::CreatureRightClickWindow,
        encounter::EncounterWindow, level_up::LevelUpWindow,
        navigation_debug::NavigationDebugWindow, spawn_predefined::SpawnPredefinedWindow,
    },
};

pub enum MainMenuState {
    World {
        game_state: GameState,
        grid_renderer: GridRenderer,
        auto_scroll_event_log: bool,
        log_level: LogLevel,
        log_source: usize,
        encounters: Vec<EncounterWindow>,
        level_up: Option<LevelUpWindow>,
        spawn_predefined: Option<SpawnPredefinedWindow>,
        creature_debug: Option<CreatureDebugWindow>,
        creature_right_click: Option<CreatureRightClickWindow>,
        navigation_debug: NavigationDebugWindow,
    },
}

pub struct MainMenuWindow {
    state: MainMenuState,
}

impl MainMenuWindow {
    pub fn new(gl_context: &glow::Context) -> Self {
        // TODO: I guess we should save/load this from/to a config file
        let mut initial_config = rerecast::ConfigBuilder::default();
        initial_config.cell_size_fraction = 8.0;
        initial_config.min_region_size = 4;
        // TODO: Add support for non-triangular polygons
        initial_config.max_vertices_per_polygon = 3;

        Self {
            state: MainMenuState::World {
                auto_scroll_event_log: true,
                log_level: LogLevel::Info,
                log_source: 0,
                game_state: GameState::new(
                    "engine/assets/test_terrain.obj",
                    &initial_config.clone().build(),
                ),
                grid_renderer: GridRenderer::new(
                    gl_context,
                    100, // extent: 20 → −20..+20
                    1.0, // step: 1 meter
                    10,  // major line every 10 units
                    include_str!("../render/world/shaders/grid.vert"),
                    include_str!("../render/world/shaders/grid.frag"),
                ),
                encounters: Vec::new(),
                level_up: None,
                spawn_predefined: None,
                creature_debug: None,
                creature_right_click: None,
                navigation_debug: NavigationDebugWindow::new(&initial_config),
            },
        }
    }

    pub fn render(&mut self, ui: &mut imgui::Ui, gui_state: &mut GuiState) {
        match &mut self.state {
            MainMenuState::World {
                game_state,
                grid_renderer,
                auto_scroll_event_log,
                log_level,
                log_source,
                encounters,
                level_up,
                spawn_predefined,
                creature_debug,
                creature_right_click,
                navigation_debug,
            } => {
                let gl_context = gui_state.ig_renderer.gl_context();
                let program = &gui_state.program;
                let camera = &mut gui_state.camera;
                let mesh_cache = &mut gui_state.mesh_cache;

                grid_renderer.draw(gl_context);

                camera.render_mut_with_context(
                    ui,
                    (
                        game_state,
                        gui_state
                            .settings
                            .get_bool(state::parameters::RENDER_CAMERA_DEBUG),
                    ),
                );

                navigation_debug.render_mut_with_context(
                    ui,
                    (
                        gui_state.ig_renderer.gl_context(),
                        game_state,
                        mesh_cache,
                        gui_state
                            .settings
                            .get_bool(state::parameters::RENDER_NAVIGATION_DEBUG),
                    ),
                );

                // TODO: Do something less "hardcoded" with the mesh cache
                if let Some(mesh) = mesh_cache.get("world") {
                    mesh.draw(
                        gl_context,
                        program,
                        &Matrix4::identity(),
                        [0.75, 0.75, 0.75, 1.0],
                        &Wireframe::None,
                    );
                } else {
                    let mesh = Mesh::from_parry_trimesh(gl_context, &game_state.geometry.trimesh);
                    mesh_cache.insert("world".to_string(), mesh);
                }

                if let Some(mesh) = mesh_cache.get("navmesh") {
                    if navigation_debug.render_navmesh {
                        mesh.draw(
                            gl_context,
                            program,
                            &Matrix4::identity(),
                            [0.2, 0.8, 0.2, 0.5],
                            &Wireframe::Overlay {
                                color: [0.0, 0.5, 0.0, 0.5],
                                width: 2.0,
                            },
                        );
                    }
                } else {
                    let mesh =
                        Mesh::from_poly_navmesh(gl_context, &game_state.geometry.poly_navmesh);
                    mesh_cache.insert("navmesh".to_string(), mesh);
                }

                for (entity, pose) in game_state.world.query::<&CreaturePose>().iter() {
                    systems::geometry::get_shape(&game_state.world, entity).map(|shape| {
                        let key = format!("{}-{}", shape.radius, shape.half_height());
                        if let Some(mesh) = mesh_cache.get(&key) {
                            mesh.draw(
                                gl_context,
                                program,
                                &pose.to_homogeneous(),
                                [0.8, 0.8, 0.8, 1.0],
                                &Wireframe::None,
                            );
                        } else {
                            let mesh = shapes::build_capsule_mesh(
                                gl_context,
                                8,
                                16,
                                shape.radius,
                                shape.half_height(),
                            );
                            mesh_cache.insert(key, mesh);
                        }
                    });
                }

                Self::render_creature_labels(ui, game_state, camera);

                // Make the raycast result available to the other parts of the UI
                // If anyone of them want to use a mouse click, e.g. spawning a
                // creature at the cursor, they should .take() it
                let mut raycast_result = if ui.io().want_capture_mouse {
                    None
                } else if let Some(ray_from_cursor) = camera.ray_from_cursor() {
                    systems::geometry::raycast(game_state, &ray_from_cursor)
                } else {
                    None
                };

                ui.window("World").always_auto_resize(true).build(|| {
                    Self::render_character_menu(
                        ui,
                        game_state,
                        level_up,
                        spawn_predefined,
                        encounters,
                        creature_debug,
                        &mut raycast_result,
                        log_source,
                    );

                    ui.same_line();

                    Self::render_event_log(
                        ui,
                        game_state,
                        encounters,
                        auto_scroll_event_log,
                        log_level,
                        log_source,
                    );
                });

                let mut encounter_finished = None;
                for encounter in &mut *encounters {
                    render_window_at_cursor(
                        ui,
                        &format!("Encounter: {}", encounter.id()),
                        true,
                        || {
                            encounter.render_mut_with_context(ui, (game_state, camera));
                        },
                    );
                    if encounter.finished() {
                        encounter_finished = Some(encounter.id().clone());
                    }
                }
                if let Some(id) = encounter_finished {
                    encounters.retain(|encounter| encounter.id() != &id);
                }

                // If the raycast result was not taken by anyone, we can fallback
                // to using it for inspecting entities
                if let Some(raycast) = &raycast_result {
                    if let Some(closest) = raycast.closest() {
                        match &closest.kind {
                            RaycastHitKind::Creature(entity) => {
                                if ui.is_mouse_clicked(MouseButton::Right) {
                                    ui.open_popup("RightClick");
                                    creature_right_click
                                        .replace(CreatureRightClickWindow::new(*entity));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(creature_right_click) = creature_right_click {
                    ui.popup("RightClick", || {
                        creature_right_click.render_mut_with_context(ui, game_state);
                    });
                }
            }
        }
    }

    fn render_character_menu(
        ui: &imgui::Ui,
        game_state: &mut GameState,
        level_up_window: &mut Option<LevelUpWindow>,
        spawn_predefined_window: &mut Option<SpawnPredefinedWindow>,
        encounters: &mut Vec<EncounterWindow>,
        debug_window: &mut Option<CreatureDebugWindow>,
        raycast_result: &mut Option<RaycastResult>,
        log_source: &mut usize,
    ) {
        ui.child_window("Characters")
            .child_flags(
                ChildFlags::ALWAYS_AUTO_RESIZE
                    | ChildFlags::AUTO_RESIZE_X
                    | ChildFlags::AUTO_RESIZE_Y,
            )
            .build(|| {
                ui.separator_with_text("Creatures");

                let mut entities = game_state
                    .world
                    .query::<&Name>()
                    .into_iter()
                    .map(|(entity, name)| (entity, name.clone()))
                    .collect::<Vec<_>>();

                let entitiy_count = entities.len();

                entities.iter_mut().for_each(|(entity, name)| {
                    if ui.collapsing_header(
                        format!("{}##{:?}", name.as_str(), entity),
                        imgui::TreeNodeFlags::FRAMED,
                    ) {
                        entity.render_mut_with_context(ui, &mut game_state.world);
                        ui.separator();

                        if ui.button(format!("Debug##{:?}", entity)) {
                            *debug_window = Some(CreatureDebugWindow::new(*entity));
                            ui.open_popup("Debug");
                        }
                    }
                });

                if let Some(debug_gui) = debug_window {
                    ui.popup("Debug", || {
                        debug_gui.render_mut_with_context(ui, game_state);
                    });
                }

                ui.separator();
                if ui.button("Spawn Creature") {
                    ui.open_popup("Spawn Creature");
                }
                Self::render_spawn_creature(
                    ui,
                    game_state,
                    level_up_window,
                    spawn_predefined_window,
                    raycast_result,
                );

                ui.separator();
                if render_button_disabled_conditionally(
                    ui,
                    "New Encounter",
                    [0.0, 0.0],
                    entitiy_count < 2,
                    "You must have at least two characters to create an encounter.",
                ) {
                    let window = EncounterWindow::new();
                    encounters.push(window);
                    *log_source = encounters.len(); // Select the new encounter as log source
                }
            });
    }

    fn render_spawn_creature(
        ui: &imgui::Ui,
        game_state: &mut GameState,
        level_up_window: &mut Option<LevelUpWindow>,
        spawn_predefined_window: &mut Option<SpawnPredefinedWindow>,
        raycast_result: &mut Option<RaycastResult>,
    ) {
        ui.popup("Spawn Creature", || {
            if let Some(index) = render_uniform_buttons_with_padding(
                ui,
                ["New Character", "Predefined Creature"],
                [20.0, 5.0],
            ) {
                match index {
                    0 => *level_up_window = Some(LevelUpWindow::new(&game_state.world, None)),
                    // TODO: Don't create the window from scratch every time
                    1 => *spawn_predefined_window = Some(SpawnPredefinedWindow::new()),
                    _ => unreachable!(),
                }
                ui.close_current_popup();
            }
        });

        if let Some(level_up) = level_up_window {
            level_up.render_mut_with_context(ui, &mut game_state.world);
            if level_up.is_level_up_complete() {
                level_up_window.take();
            }
        }

        if let Some(spawn_predefined) = spawn_predefined_window {
            spawn_predefined.render_mut_with_context(ui, (&mut game_state.world, raycast_result));
            if spawn_predefined.is_spawning_completed() {
                spawn_predefined_window.take();
            }
        }
    }

    fn render_event_log(
        ui: &imgui::Ui,
        game_state: &mut GameState,
        encounters: &mut Vec<EncounterWindow>,
        auto_scroll_event_log: &mut bool,
        log_level: &mut LogLevel,
        log_source: &mut usize,
    ) {
        ui.window("Event Log")
            .flags(WindowFlags::ALWAYS_AUTO_RESIZE)
            .build(|| {
                let mut log_sources = vec!["World".to_string()];
                log_sources.extend(
                    game_state
                        .encounters
                        .iter()
                        .map(|e| format!("Encounter {}", e.0)),
                );
                *log_source = log_sources.len().min(*log_source);

                let width_token = ui.push_item_width(150.0);
                ui.combo("Log source", log_source, &log_sources[..], |s| {
                    s.to_string().into()
                });
                width_token.end();

                let event_log = if *log_source == 0 || encounters.len() < *log_source {
                    &game_state.event_log
                } else {
                    let id = encounters.get(*log_source - 1).map(|e| e.id()).unwrap();
                    game_state
                        .encounters
                        .get(&id)
                        .map(|e| e.combat_log())
                        .unwrap_or(&game_state.event_log)
                };

                ui.child_window("Event Log Content")
                    .child_flags(
                        ChildFlags::ALWAYS_AUTO_RESIZE
                            | ChildFlags::AUTO_RESIZE_X
                            | ChildFlags::BORDERS,
                    )
                    .size([0.0, 400.0])
                    .build(|| {
                        event_log.render_with_context(ui, &(&game_state.world, log_level));

                        if *auto_scroll_event_log && ui.scroll_y() >= ui.scroll_max_y() - 5.0 {
                            ui.set_scroll_here_y_with_ratio(1.0);
                        }
                    });

                ui.checkbox("Auto-scroll", auto_scroll_event_log);

                let mut current_log_level = log_level.clone() as usize;
                let width_token = ui.push_item_width(60.0);
                if ui.combo(
                    "Log level",
                    &mut current_log_level,
                    &LogLevel::iter().collect::<Vec<_>>()[..],
                    |lvl| lvl.to_string().into(),
                ) {
                    *log_level = current_log_level.into();
                }
                width_token.end();
            });
    }

    fn render_creature_labels(ui: &imgui::Ui, game_state: &GameState, camera: &OrbitCamera) {
        for (entity, name) in game_state.world.query::<&Name>().iter() {
            if let Some(pose) = game_state.world.get::<&CreaturePose>(entity).ok() {
                let translation = pose.translation.vector;
                let pos = camera.world_to_screen(&Point3::new(
                    translation.x,
                    translation.y
                        + systems::geometry::get_height(&game_state.world, entity).unwrap(),
                    translation.z,
                ));

                if let Some((x, y)) = pos {
                    let height = ui.calc_text_size(name.as_str())[1] * 2.0;
                    let width = ui.calc_text_size("HP:")[0] + 150.0; // rough estimate for width
                    let window_pos = [x - width / 2.0, y - height];
                    ui.window(&format!("Label##{:?}", entity))
                        .always_auto_resize(true)
                        .position(window_pos, imgui::Condition::Always)
                        .bg_alpha(0.5)
                        .title_bar(false)
                        .resizable(false)
                        .movable(false)
                        .scrollable(false)
                        .focus_on_appearing(false)
                        .collapsed(false, imgui::Condition::Always)
                        .mouse_inputs(false)
                        .build(|| {
                            name.render(ui);
                            render_if_present::<HitPoints>(ui, &game_state.world, entity);
                            ui.same_line();
                            render_if_present::<LifeState>(ui, &game_state.world, entity);
                        });
                }
            }
        }
    }
}

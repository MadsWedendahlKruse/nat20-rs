use hecs::World;
use imgui::{ChildFlags, TreeNodeFlags};
use nat20_rs::{
    components::id::Name,
    engine::{
        encounter,
        game_state::{self, EventLog, GameEvent, GameState},
    },
    entities::character::CharacterTag,
    systems,
};

use crate::{
    render::{
        entities::CreatureRenderMode,
        text::{TextKind, TextSegments},
        utils::{
            ImguiRenderable, ImguiRenderableMut, ImguiRenderableMutWithContext,
            ImguiRenderableWithContext, render_button_disabled_conditionally,
            render_uniform_buttons, render_window_at_cursor,
        },
    },
    windows::{
        creature_debug::CreatureDebugWindow, encounter::EncounterWindow, level_up::LevelUpWindow,
        spawn_predefined::SpawnPredefinedWindow,
    },
};

pub enum MainMenuState {
    World {
        game_state: GameState,
        auto_scroll_event_log: bool,
        encounters: Vec<EncounterWindow>,
        level_up: Option<LevelUpWindow>,
        spawn_predefined: Option<SpawnPredefinedWindow>,
        character_debug: Option<CreatureDebugWindow>,
    },
}

pub struct MainMenuWindow {
    state: MainMenuState,
}

impl MainMenuWindow {
    pub fn new() -> Self {
        Self {
            state: MainMenuState::World {
                auto_scroll_event_log: true,
                game_state: GameState::new(),
                encounters: Vec::new(),
                level_up: None,
                spawn_predefined: None,
                character_debug: None,
            },
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        match &mut self.state {
            MainMenuState::World {
                game_state,
                auto_scroll_event_log,
                encounters,
                level_up,
                spawn_predefined,
                character_debug,
            } => {
                ui.window("World").always_auto_resize(true).build(|| {
                    Self::render_character_menu(
                        ui,
                        game_state,
                        level_up,
                        spawn_predefined,
                        encounters,
                        character_debug,
                    );

                    ui.same_line();

                    Self::render_event_log(ui, game_state, auto_scroll_event_log);
                });

                let mut encounter_finished = None;
                for encounter in &mut *encounters {
                    render_window_at_cursor(
                        ui,
                        &format!("Encounter: {}", encounter.id()),
                        true,
                        || {
                            encounter.render_mut_with_context(ui, game_state);
                        },
                    );
                    if encounter.finished() {
                        encounter_finished = Some(encounter.id().clone());
                    }
                }
                if let Some(id) = encounter_finished {
                    encounters.retain(|encounter| encounter.id() != &id);
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

                        if ui.button(format!("Remove Character##{:?}", entity)) {
                            let _ = game_state.world.despawn(*entity);
                        }

                        if ui.button(format!("Debug##{:?}", entity)) {
                            *debug_window = Some(CreatureDebugWindow::new(*entity));
                            ui.open_popup("Debug");
                        }

                        if let Some(debug_gui) = debug_window {
                            debug_gui.render_mut_with_context(ui, game_state);
                        }
                    }
                });

                ui.separator();
                if ui.button("Spawn Creature") {
                    ui.open_popup("Spawn Creature");
                }
                Self::render_spawn_creature(
                    ui,
                    game_state,
                    level_up_window,
                    spawn_predefined_window,
                );

                ui.separator();
                if render_button_disabled_conditionally(
                    ui,
                    "New Encounter",
                    [0.0, 0.0],
                    entitiy_count < 2,
                    "You must have at least two characters to create an encounter.",
                ) {
                    encounters.push(EncounterWindow::new());
                }
            });
    }

    fn render_spawn_creature(
        ui: &imgui::Ui,
        game_state: &mut GameState,
        level_up_window: &mut Option<LevelUpWindow>,
        spawn_predefined_window: &mut Option<SpawnPredefinedWindow>,
    ) {
        ui.popup("Spawn Creature", || {
            if let Some(index) =
                render_uniform_buttons(ui, ["New Character", "Predefined Creature"], [20.0, 5.0])
            {
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
            spawn_predefined.render_mut_with_context(ui, &mut game_state.world);
            if spawn_predefined.is_spawning_completed() {
                spawn_predefined_window.take();
            }
        }
    }

    fn render_event_log(
        ui: &imgui::Ui,
        game_state: &mut GameState,
        auto_scroll_event_log: &mut bool,
    ) {
        ui.child_window("Event Log")
            .child_flags(
                ChildFlags::ALWAYS_AUTO_RESIZE
                    | ChildFlags::AUTO_RESIZE_X
                    | ChildFlags::AUTO_RESIZE_Y,
            )
            .build(|| {
                ui.separator_with_text("Event Log");

                ui.child_window("Event Log Content")
                    .child_flags(
                        ChildFlags::ALWAYS_AUTO_RESIZE
                            | ChildFlags::AUTO_RESIZE_X
                            | ChildFlags::BORDERS,
                    )
                    .size([0.0, 500.0])
                    .build(|| {
                        game_state
                            .event_log
                            .render_with_context(ui, &game_state.world);

                        if *auto_scroll_event_log && ui.scroll_y() >= ui.scroll_max_y() - 5.0 {
                            ui.set_scroll_here_y_with_ratio(1.0);
                        }
                    });

                ui.checkbox("Auto-scroll", auto_scroll_event_log);
            });
    }
}

impl ImguiRenderableWithContext<&World> for EventLog {
    fn render_with_context(&self, ui: &imgui::Ui, world: &World) {
        for event in self {
            match event {
                GameEvent::EncounterStarted(encounter_id) => {
                    ui.separator_with_text(&format!("Encounter {}", encounter_id));
                }

                GameEvent::EncounterEnded(encounter_id, combat_log) => {
                    if ui.collapsing_header(
                        format!("Combat log##{}", encounter_id),
                        TreeNodeFlags::FRAMED,
                    ) {
                        combat_log.render_with_context(ui, world);
                    }
                    ui.separator();
                }

                GameEvent::SavingThrow(entity, result, dc) => {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *entity).to_string(),
                            TextKind::Actor,
                        ),
                        (
                            // TODO: a vs an
                            if result.success {
                                "succeeded a".to_string()
                            } else {
                                "failed a".to_string()
                            },
                            TextKind::Normal,
                        ),
                        (dc.key.to_string(), TextKind::Ability),
                        ("saving throw".to_string(), TextKind::Normal),
                    ])
                    .render(ui);

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text("DC:");
                            ui.same_line();
                            dc.render(ui);
                            ui.text("");
                            ui.text("Saving Throw:");
                            ui.same_line();
                            result.render(ui);
                        });
                    }
                }

                GameEvent::SkillCheck(entity, result, dc) => {
                    TextSegments::new(vec![
                        (
                            systems::helpers::get_component::<Name>(world, *entity).to_string(),
                            TextKind::Actor,
                        ),
                        (
                            if result.success {
                                "succeeded a".to_string()
                            } else {
                                "failed a".to_string()
                            },
                            TextKind::Normal,
                        ),
                        (dc.key.to_string(), TextKind::Skill),
                        ("skill check".to_string(), TextKind::Normal),
                    ])
                    .render(ui);

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text("DC:");
                            ui.same_line();
                            dc.render(ui);
                            ui.text("");
                            ui.text("Skill Check:");
                            ui.same_line();
                            result.render(ui);
                        });
                    }
                }
            }
        }
    }
}

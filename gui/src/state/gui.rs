use hecs::World;
use imgui::{ChildFlags, TreeNodeFlags};
use nat20_rs::{
    engine::game_state::{EventLog, GameEvent, GameState},
    entities::character::CharacterTag,
    systems,
};

use crate::{
    render::{
        text::{TextKind, TextSegments},
        utils::{
            ImguiRenderable, ImguiRenderableMut, ImguiRenderableMutWithContext,
            ImguiRenderableWithContext, render_button_disabled_conditionally,
            render_window_at_cursor,
        },
    },
    state::{
        character_creation::{CharacterCreation, CharacterCreationState},
        character_debug::CharacterDebugGui,
        encounter::EncounterGui,
    },
};

pub enum GuiState {
    MainMenu { auto_scroll_event_log: bool },
}

pub struct GameGui {
    gui_state: GuiState,
    game_state: GameState,
    encounters: Vec<EncounterGui>,
    character_creation: CharacterCreation,
    character_debug: Option<CharacterDebugGui>,
}

impl GameGui {
    pub fn new() -> Self {
        Self {
            gui_state: GuiState::MainMenu {
                auto_scroll_event_log: true,
            },
            game_state: GameState::new(),
            encounters: Vec::new(),
            character_creation: CharacterCreation::new(),
            character_debug: None,
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        ui.window("World")
            .always_auto_resize(true)
            .build(|| match self.gui_state {
                GuiState::MainMenu { .. } => {
                    self.render_character_menu(ui);

                    ui.same_line();

                    self.render_event_log(ui);
                }
            });

        let mut encounter_finished = None;
        for encounter in &mut self.encounters {
            render_window_at_cursor(ui, &format!("Encounter: {}", encounter.id()), true, || {
                encounter.render_mut_with_context(ui, &mut self.game_state);
            });
            if encounter.finished() {
                encounter_finished = Some(encounter.id().clone());
            }
        }
        if let Some(id) = encounter_finished {
            self.encounters.retain(|encounter| encounter.id() != &id);
        }
    }

    pub fn render_character_menu(&mut self, ui: &imgui::Ui) {
        ui.child_window("Characters")
            .child_flags(
                ChildFlags::ALWAYS_AUTO_RESIZE
                    | ChildFlags::AUTO_RESIZE_X
                    | ChildFlags::AUTO_RESIZE_Y,
            )
            .build(|| {
                ui.separator_with_text("Characters in the world");
                // Avoid double borrow
                let characters = self
                    .game_state
                    .world
                    .query_mut::<(&String, &CharacterTag)>()
                    .into_iter()
                    .map(|(entity, (name, tag))| (entity, name.clone(), tag.clone()))
                    .collect::<Vec<_>>();

                for (entity, name, tag) in &characters {
                    if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                        (*entity, tag.clone())
                            .render_mut_with_context(ui, &mut self.game_state.world);

                        ui.separator();
                        if ui.button(format!("Remove Character##{:?}", entity)) {
                            let _ = self.game_state.world.despawn(*entity);
                        }

                        if ui.button(format!("Debug##{:?}", entity)) {
                            self.character_debug = Some(CharacterDebugGui::new(*entity));
                            ui.open_popup("Debug");
                        }

                        if let Some(debug_gui) = &mut self.character_debug {
                            debug_gui.render_mut_with_context(ui, &mut self.game_state);
                        }
                    }
                }

                ui.separator();
                if ui.button("Add Character") {
                    self.character_creation
                        .set_state(CharacterCreationState::ChoosingMethod);
                }

                ui.separator();
                if render_button_disabled_conditionally(
                    ui,
                    "New Encounter",
                    [0.0, 0.0],
                    characters.len() < 2,
                    "You must have at least two characters to create an encounter.",
                ) {
                    self.encounters.push(EncounterGui::new());
                }

                self.character_creation.render_mut(ui);

                if self.character_creation.creation_complete() {
                    println!("Character creation complete!");
                    if let Some(character) = self.character_creation.get_character() {
                        println!("Character created: {:?}", character.name);
                        let entity = self.game_state.world.spawn(character);
                        // They spawn at zero health by default
                        systems::health::heal_full(&mut self.game_state.world, entity);
                    }
                }
            });
    }

    pub fn render_event_log(&mut self, ui: &imgui::Ui) {
        let auto_scroll_event_log = match &mut self.gui_state {
            GuiState::MainMenu {
                auto_scroll_event_log,
            } => auto_scroll_event_log,
        };

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
                        self.game_state
                            .event_log
                            .render_with_context(ui, &self.game_state.world);

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
                            systems::helpers::get_component::<String>(world, *entity).to_string(),
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
                            systems::helpers::get_component::<String>(world, *entity).to_string(),
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

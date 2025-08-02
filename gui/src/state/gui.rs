use std::collections::HashSet;

use hecs::Entity;
use imgui::TreeNodeFlags;
use nat20_rs::{
    components::id::EncounterId,
    engine::{encounter::Encounter, game_state::GameState},
    entities::character::CharacterTag,
    systems,
};

use crate::{
    render::utils::{
        ImguiRenderableMut, ImguiRenderableMutWithContext, render_button_disabled_conditionally,
        render_button_selectable,
    },
    state::{
        character_creation::{CharacterCreation, CharacterCreationState},
        encounter::EncounterGui,
    },
};

pub enum GuiState {
    MainMenu,
}

pub struct GameGui {
    gui_state: GuiState,
    game_state: GameState,
    character_creation: CharacterCreation,
    encounters: Vec<EncounterGui>,
}

impl GameGui {
    pub fn new() -> Self {
        Self {
            gui_state: GuiState::MainMenu,
            game_state: GameState::new(),
            character_creation: CharacterCreation::new(),
            encounters: Vec::new(),
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        ui.window("World").always_auto_resize(true).build(|| {
            match self.gui_state {
                GuiState::MainMenu => {
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
                        "Start Encounter",
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
                }
            }
        });

        for encounter in &mut self.encounters {
            ui.window(format!("Encounter: {}", encounter.id()))
                .always_auto_resize(true)
                .build(|| {
                    encounter.render_mut_with_context(ui, &mut self.game_state);
                });
        }
    }
}

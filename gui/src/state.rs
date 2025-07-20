use std::collections::HashMap;

use hecs::{Entity, World};
use imgui::{Condition, TreeNodeFlags};
use nat20_rs::{components::id::CharacterId, systems, test_utils::fixtures};

use crate::render::imgui_render::ImguiRenderableMut;

pub struct GuiState {
    world: World,
    creation_state: Option<CharacterCreationState>,
    characters: HashMap<CharacterId, Entity>,
}

#[derive(Debug, PartialEq)]
enum CharacterCreationState {
    ChoosingMethod,
    FromPredefined,
    FromScratch,
}

impl GuiState {
    pub fn new() -> Self {
        let mut world = World::new();

        let spawners = vec![
            fixtures::creatures::heroes::fighter,
            fixtures::creatures::heroes::wizard,
            fixtures::creatures::heroes::warlock,
            fixtures::creatures::monsters::goblin_warrior,
        ];

        let mut characters = HashMap::new();

        for spawner in spawners {
            let (entity, id) = spawner(&mut world);
            characters.insert(id, entity);
            let name = systems::helpers::get_component_clone::<String>(&world, entity);
            println!("Spawned character: {} ({}) ({})", name, id, entity.id());
        }

        Self {
            world,
            creation_state: None,
            characters,
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        self.render_world(ui);
        self.render_character_creation(ui);
    }

    fn render_world(&mut self, ui: &imgui::Ui) {
        ui.window("World")
            .size([400.0, 600.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Characters in the world:");
                for (id, character) in &mut self.characters {
                    let name =
                        systems::helpers::get_component_clone::<String>(&self.world, *character);
                    if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                        (&mut self.world, *character).render_mut(ui);
                    }
                }
                if ui.button("Add Character") {
                    self.creation_state = Some(CharacterCreationState::ChoosingMethod);
                }
            });
    }

    fn render_character_creation(&mut self, ui: &imgui::Ui) {
        if self.creation_state.is_none() {
            return;
        }
        ui.window("Character Creation")
            .size([600.0, 800.0], Condition::FirstUseEver)
            .build(|| {
                match self.creation_state {
                    Some(CharacterCreationState::ChoosingMethod) => {
                        if ui.button("From Predefined") {
                            self.creation_state = Some(CharacterCreationState::FromPredefined);
                        }
                        if ui.button("From Scratch") {
                            self.creation_state = Some(CharacterCreationState::FromScratch);
                        }
                        if ui.button("Cancel") {
                            self.creation_state = None;
                        }
                    }
                    Some(CharacterCreationState::FromPredefined) => {
                        for (id, character) in &mut self.characters {
                            let name = systems::helpers::get_component_clone::<String>(
                                &self.world,
                                *character,
                            );
                            if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                                if ui.button(format!("Add to World##{}", character.id())) {
                                    // self.world.add_character(character.clone());
                                    self.creation_state = None;
                                }
                                ui.separator();
                                (&mut self.world, *character).render_mut(ui);
                            }
                        }
                        ui.separator();
                        if ui.button("Back") {
                            self.creation_state = Some(CharacterCreationState::ChoosingMethod);
                        }
                    }
                    Some(CharacterCreationState::FromScratch) => {
                        // Logic for scratch character creation
                        ui.separator();
                        if ui.button("Back") {
                            self.creation_state = Some(CharacterCreationState::ChoosingMethod);
                        }
                    }
                    None => {}
                }
            });
    }
}

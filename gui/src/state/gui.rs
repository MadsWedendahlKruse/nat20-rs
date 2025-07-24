use std::collections::HashMap;

use hecs::{Entity, World};
use imgui::{Condition, TreeNodeFlags};
use nat20_rs::{components::id::CharacterId, entities::character::CharacterTag, systems};

use crate::{
    render::imgui_render::ImguiRenderableMut,
    state::character_creation::{CharacterCreation, CharacterCreationState},
};

pub struct GuiState {
    world: World,
    character_creation: CharacterCreation,
    characters: HashMap<CharacterId, Entity>,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            character_creation: CharacterCreation::new(),
            characters: HashMap::new(),
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui) {
        self.render_world(ui);
        self.character_creation.render_mut(ui);

        if self.character_creation.creation_complete() {
            println!("Character creation complete!");
            if let Some(character) = self.character_creation.get_character() {
                println!("Character created: {:?}", character.name);
                let character_id = character.id;
                let entity = self.world.spawn(character);
                self.characters.insert(character_id, entity);
                // They spawn at zero health by default
                systems::health::heal_full(&mut self.world, entity);
            }
        }
    }

    fn render_world(&mut self, ui: &imgui::Ui) {
        ui.window("World")
            .size([400.0, 600.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Characters in the world:");

                for (id, entity) in &mut self.characters {
                    let name =
                        systems::helpers::get_component_clone::<String>(&self.world, *entity);
                    let tag =
                        systems::helpers::get_component_clone::<CharacterTag>(&self.world, *entity);
                    if ui.collapsing_header(&name, TreeNodeFlags::FRAMED) {
                        (&mut self.world, *entity, tag).render_mut(ui);
                    }
                }
                if ui.button("Add Character") {
                    self.character_creation.state = Some(CharacterCreationState::ChoosingMethod);
                }
            });
    }
}

use imgui::{Condition, Id, TableColumnFlags, TableColumnSetup, TableFlags, TreeNodeFlags};
use nat20_rs::{
    creature::character::Character,
    engine::world::World,
    stats::{ability::Ability, d20_check::RollMode, proficiency::Proficiency, skill::Skill},
    test_utils::fixtures,
};
use strum::IntoEnumIterator;

use crate::render::imgui_render::{
    ImguiRenderable, ImguiRenderableMut, ImguiRenderableWithContext,
};

pub struct GuiState<'c> {
    world: World<'c>,
    creation_state: Option<CharacterCreationState>,
    predefined_characters: Vec<Character>,
}

#[derive(Debug, PartialEq)]
enum CharacterCreationState {
    ChoosingMethod,
    FromPredefined,
    FromScratch,
}

impl<'c> GuiState<'c> {
    pub fn new(world: World<'c>) -> Self {
        Self {
            world,
            creation_state: None,
            predefined_characters: vec![
                fixtures::creatures::heroes::fighter(),
                fixtures::creatures::heroes::wizard(),
                fixtures::creatures::heroes::warlock(),
                fixtures::creatures::monsters::goblin_warrior(),
            ],
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
                for character in self.world.characters_mut() {
                    if ui.collapsing_header(&character.name(), TreeNodeFlags::FRAMED) {
                        character.render_mut(ui);
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
            .size([600.0, 1200.0], Condition::FirstUseEver)
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
                        for character in &mut self.predefined_characters {
                            if ui.collapsing_header(&character.name(), TreeNodeFlags::FRAMED) {
                                if ui.button(format!("Add to World##{}", character.id())) {
                                    self.world.add_character(character.clone());
                                    self.creation_state = None;
                                }
                                ui.separator();
                                character.render_mut(ui);
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

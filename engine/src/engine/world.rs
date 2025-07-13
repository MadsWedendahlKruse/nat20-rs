use std::collections::HashMap;

use crate::{
    creature::character::Character,
    engine::encounter::Encounter,
    utils::id::{CharacterId, EncounterId},
};

// TODO: Alternative name, GameState?
/// The World represents the current state of the game.
pub struct World<'c> {
    pub characters: HashMap<CharacterId, Character>,
    // TODO: Acutally implement multiple encounters
    pub encounters: HashMap<EncounterId, Encounter<'c>>,
}

impl<'c> World<'c> {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            encounters: HashMap::new(),
        }
    }

    pub fn add_character(&mut self, character: Character) {
        self.characters.insert(character.id(), character);
    }

    pub fn character(&self, id: &CharacterId) -> Option<&Character> {
        self.characters.get(id)
    }

    pub fn character_mut(&mut self, id: &CharacterId) -> Option<&mut Character> {
        self.characters.get_mut(id)
    }

    pub fn characters(&self) -> Vec<&Character> {
        self.characters.values().collect()
    }

    pub fn characters_mut(&mut self) -> Vec<&mut Character> {
        self.characters.values_mut().collect()
    }

    pub fn start_encounter(&mut self, participants: Vec<&'c mut Character>) -> &Encounter<'c> {
        let encounter = Encounter::new(participants);
        let encounter_id = EncounterId::new_v4();
        self.encounters.insert(encounter_id, encounter);
        self.encounters.get(&encounter_id).unwrap()
    }
}

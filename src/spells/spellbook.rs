use std::collections::HashMap;

use crate::{
    combat::action::{CombatAction, CombatActionProvider},
    stats::ability::Ability,
    utils::id::SpellId,
};

use super::spell::Spell;

#[derive(Debug, Clone)]
pub struct Spellbook {
    spells: HashMap<SpellId, Spell>,
    /// Spell slots available for each spell level.
    /// The key is the spell level (1-9), and the value is the number of slots available.
    spell_slots: HashMap<u8, u8>,
    current_spell_slots: HashMap<u8, u8>,
}

impl Spellbook {
    pub fn new() -> Self {
        Self {
            spells: HashMap::new(),
            spell_slots: HashMap::new(),
            current_spell_slots: HashMap::new(),
        }
    }

    pub fn add_spell(&mut self, mut spell: Spell, spellcasting_ability: Ability) {
        spell.set_spellcasting_ability(spellcasting_ability);
        self.spells.insert(spell.id().clone(), spell);
    }

    pub fn remove_spell(&mut self, spell_id: &str) -> Option<Spell> {
        self.spells.remove(spell_id)
    }

    pub fn get_spell(&self, spell_id: &str) -> Option<&Spell> {
        self.spells.get(spell_id)
    }

    pub fn has_spell(&self, spell_id: &str) -> bool {
        self.spells.contains_key(spell_id)
    }

    pub fn all_spells(&self) -> Vec<SpellId> {
        self.spells.keys().cloned().collect()
    }

    pub fn update_spell_slots(&mut self, caster_level: u8) {
        // Clear current slots
        self.current_spell_slots.clear();
        // Calculate new spell slots based on caster level
        let spell_slots = Self::spell_slots_per_level(caster_level);
        for spell_level in 1..=spell_slots.len() as u8 {
            let slots = spell_slots[spell_level as usize - 1];
            self.spell_slots.insert(spell_level, slots);
            self.current_spell_slots.insert(spell_level, slots);
        }
    }

    fn spell_slots_per_level(caster_level: u8) -> Vec<u8> {
        match caster_level {
            1 => vec![2],
            2 => vec![3],
            3 => vec![4, 2],
            4 => vec![4, 3],
            5 => vec![4, 3, 2],
            6 => vec![4, 3, 3],
            7 => vec![4, 3, 3, 1],
            8 => vec![4, 3, 3, 2],
            9 => vec![4, 3, 3, 3, 1],
            10 => vec![4, 3, 3, 3, 2],
            11..=12 => vec![4, 3, 3, 3, 2, 1],
            13..=14 => vec![4, 3, 3, 3, 2, 1, 1],
            15..=16 => vec![4, 3, 3, 3, 2, 1, 1, 1],
            17..=18 => vec![4, 3, 3, 3, 2, 1, 1, 1, 1],
            19 => vec![4, 3, 3, 3, 3, 2, 1, 1, 1],
            20 => vec![4, 3, 3, 3, 3, 2, 2, 1, 1],
            _ => vec![],
        }
    }

    pub fn use_spell_slot(&mut self, level: u8) -> bool {
        if let Some(current_slots) = self.current_spell_slots.get_mut(&level) {
            if *current_slots > 0 {
                *current_slots -= 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn restore_spell_slot(&mut self, level: u8) {
        if let Some(current_slots) = self.current_spell_slots.get_mut(&level) {
            *current_slots += 1;
        } else {
            // TODO: Is it allowed to restore a slot of a higher level than the current max?
            self.current_spell_slots.insert(level, 1);
        }
    }

    pub fn restore_all_spell_slots(&mut self) {
        for (level, slots) in &self.spell_slots {
            self.current_spell_slots.insert(*level, *slots);
        }
    }

    fn available_spell_slots_for_base_level(&self, base_level: u8) -> HashMap<u8, u8> {
        let mut spell_slots = HashMap::new();
        let max_level = self.spell_slots.keys().max().cloned().unwrap_or(0);
        if base_level > max_level {
            return spell_slots; // No slots available for levels higher than the max
        }
        for level in base_level..=max_level {
            if let Some(slots) = self.current_spell_slots.get(&level) {
                spell_slots.insert(level, *slots);
            }
        }
        spell_slots
    }
}

impl CombatActionProvider for Spellbook {
    fn available_actions(&self) -> Vec<CombatAction> {
        let mut actions = Vec::new();
        for spell in self.spells.values() {
            let available_slots = self.available_spell_slots_for_base_level(spell.base_level());
            for (level, slots) in available_slots {
                if slots > 0 {
                    actions.push(CombatAction::CastSpell {
                        id: spell.id().clone(),
                        level,
                    });
                }
            }
        }
        actions
    }
}

use std::collections::{HashMap, HashSet};

use crate::{
    actions::action::{ActionContext, ActionProvider},
    registry,
    stats::ability::Ability,
    utils::id::{ActionId, ResourceId, SpellId},
};

#[derive(Debug, Clone)]
pub struct Spellbook {
    /// Set of learned spells
    spells_by_spell_id: HashSet<SpellId>,
    /// Store the ID of the spell's action for quick access.
    /// This is primarily used when submitting actions in the combat engine,
    /// which is done using ActionId.
    spells_by_action_id: HashMap<ActionId, SpellId>,
    /// The ability to use when casting the spell. This depends on the class the
    /// spell was learned as
    spellcasting_ability: HashMap<SpellId, Ability>,
    /// Spell slots available for each spell level.
    /// The key is the spell level (1-9), and the value is:
    ///     (current_slots, maximum_slots).
    /// Spell slots could be treated as a resource, but that really overcomplicates things.
    spell_slots: HashMap<u8, (u8, u8)>,
}

impl Spellbook {
    pub fn new() -> Self {
        Self {
            spells_by_spell_id: HashSet::new(),
            spells_by_action_id: HashMap::new(),
            spellcasting_ability: HashMap::new(),
            spell_slots: HashMap::new(),
        }
    }

    pub fn add_spell(&mut self, spell_id: &SpellId, spellcasting_ability: Ability) {
        // TODO: Handle missing spells
        let spell = registry::spells::SPELL_REGISTRY
            .get(spell_id)
            .unwrap()
            .clone();
        let action_id = spell.action().id().clone();
        self.spells_by_spell_id.insert(spell_id.clone());
        self.spells_by_action_id.insert(action_id, spell_id.clone());
        self.spellcasting_ability
            .insert(spell_id.clone(), spellcasting_ability);
    }

    pub fn remove_spell(&mut self, spell_id: &SpellId) {
        // TODO: Handle missing spells
        let spell = registry::spells::SPELL_REGISTRY
            .get(spell_id)
            .unwrap()
            .clone();
        self.spells_by_spell_id.remove(spell_id);
        self.spells_by_action_id.remove(spell.action().id());
        self.spellcasting_ability.remove(spell_id);
    }

    pub fn get_spell_id_by_action_id(&self, action_id: &ActionId) -> Option<&SpellId> {
        self.spells_by_action_id
            .get(action_id)
            .and_then(|spell_id| self.spells_by_spell_id.get(spell_id))
    }

    pub fn has_spell(&self, spell_id: &SpellId) -> bool {
        self.spells_by_spell_id.contains(spell_id)
    }

    pub fn all_spells(&self) -> &HashSet<SpellId> {
        &self.spells_by_spell_id
    }

    pub fn spellcasting_ability(&self, spell_id: &SpellId) -> Option<&Ability> {
        self.spellcasting_ability.get(spell_id)
    }

    pub fn spell_slots(&self) -> &HashMap<u8, (u8, u8)> {
        &self.spell_slots
    }

    pub fn spell_slots_for_level(&self, level: u8) -> u8 {
        self.spell_slots
            .get(&level)
            .map(|(current, _)| *current)
            .unwrap_or(0)
    }

    pub fn update_spell_slots(&mut self, caster_level: u8) {
        // Calculate new spell slots based on caster level
        let spell_slots = Self::spell_slots_per_level(caster_level);
        for spell_level in 1..=spell_slots.len() as u8 {
            let slots = spell_slots[spell_level as usize - 1];
            self.spell_slots.insert(spell_level, (slots, slots));
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
        // TODO: Error instead of bool?
        if let Some((current_slots, _)) = self.spell_slots.get_mut(&level) {
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
        if let Some((current_slots, _)) = self.spell_slots.get_mut(&level) {
            *current_slots += 1;
        } else {
            // TODO: Is it allowed to restore a slot of a higher level than the current max?
            self.spell_slots.insert(level, (1, 0));
        }
    }

    pub fn restore_all_spell_slots(&mut self) {
        let slots: Vec<(u8, u8)> = self
            .spell_slots
            .iter()
            .map(|(level, (_, max_slots))| (*level, *max_slots))
            .collect();
        for (level, max_slots) in slots {
            self.spell_slots.insert(level, (max_slots, max_slots));
        }
    }

    fn spell_slots_for_base_level(
        &self,
        base_level: u8,
        use_current_slots: bool,
    ) -> HashMap<u8, u8> {
        let mut spell_slots = HashMap::new();
        let max_level = self.spell_slots.keys().max().cloned().unwrap_or(0);
        if base_level > max_level {
            // No slots available for levels higher than the max
            return spell_slots;
        }
        for level in base_level..=max_level {
            if let Some((current_slots, max_slots)) = self.spell_slots.get(&level) {
                let slots = if use_current_slots {
                    *current_slots
                } else {
                    *max_slots
                };
                spell_slots.insert(level, slots);
            }
        }
        spell_slots
    }

    fn action_map_from_slots(
        &self,
        use_current_slots: bool,
    ) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
        let mut actions = HashMap::new();
        for spell_id in &self.spells_by_spell_id {
            let spell = registry::spells::SPELL_REGISTRY.get(spell_id).unwrap();
            let available_slots = if spell.base_level() == 0 {
                // Cantrips always have 1 slot available
                HashMap::from([(0, 1)])
            } else {
                self.spell_slots_for_base_level(spell.base_level(), use_current_slots)
            };
            let contexts = available_slots
                .iter()
                .map(|(level, _)| ActionContext::Spell { level: *level })
                .collect();
            actions.insert(
                spell.action().id().clone(),
                (contexts, spell.action().resource_cost().clone()),
            );
        }
        actions
    }
}

impl ActionProvider for Spellbook {
    fn available_actions(
        &self,
    ) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
        self.action_map_from_slots(true)
    }

    fn all_actions(&self) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
        self.action_map_from_slots(false)
    }
}

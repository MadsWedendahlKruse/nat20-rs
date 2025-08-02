use std::collections::{HashMap, HashSet};

use crate::{
    components::{
        ability::Ability,
        actions::action::{ActionContext, ActionMap, ActionProvider},
        id::{ActionId, ResourceId, SpellId},
        resource::ResourceCostMap,
    },
    registry,
};

#[derive(Debug, Clone)]
pub struct SpellSlots {
    current: u8,
    maximum: u8,
}

impl SpellSlots {
    pub fn new(current: u8, maximum: u8) -> Self {
        Self { current, maximum }
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn maximum(&self) -> u8 {
        self.maximum
    }
}

#[derive(Debug, Clone)]
pub struct Spellbook {
    /// Set of learned spells
    spells_by_spell_id: HashSet<SpellId>,
    /// Spells that are currently prepared for casting.
    // TODO: Default prepared spells?
    prepared_spells: HashSet<SpellId>,
    /// Maximum number of spells that can be prepared.
    max_prepared_spells: usize,
    /// The ability to use when casting the spell. This depends on the class the
    /// spell was learned as
    spellcasting_ability: HashMap<SpellId, Ability>,
    /// Spell slots available for each spell level.
    /// Spell slots could be treated as a resource, but that really overcomplicates things.
    spell_slots: HashMap<u8, SpellSlots>,
}

impl Spellbook {
    pub fn new() -> Self {
        Self {
            spells_by_spell_id: HashSet::new(),
            prepared_spells: HashSet::new(),
            max_prepared_spells: 0,
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
        self.spells_by_spell_id.insert(spell_id.clone());
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
        self.spellcasting_ability.remove(spell_id);
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

    pub fn spell_slots(&self) -> &HashMap<u8, SpellSlots> {
        &self.spell_slots
    }

    pub fn spell_slots_for_level(&self, level: u8) -> SpellSlots {
        if let Some(slots) = self.spell_slots.get(&level) {
            slots.clone()
        } else {
            SpellSlots::new(0, 0)
        }
    }

    pub fn update_spell_slots(&mut self, caster_level: u8) {
        // Calculate new spell slots based on caster level
        let spell_slots = Self::spell_slots_per_level(caster_level);
        for spell_level in 1..=spell_slots.len() as u8 {
            let slots = spell_slots[spell_level as usize - 1];
            self.spell_slots
                .insert(spell_level, SpellSlots::new(slots, slots));
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
        if let Some(slots) = self.spell_slots.get_mut(&level) {
            if slots.current() > 0 {
                slots.current -= 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn restore_spell_slot(&mut self, level: u8) {
        if let Some(slots) = self.spell_slots.get_mut(&level) {
            slots.current += 1;
        } else {
            // TODO: Is it allowed to restore a slot of a higher level than the current max?
            self.spell_slots.insert(level, SpellSlots::new(1, 0));
        }
    }

    pub fn restore_all_spell_slots(&mut self) {
        let slots: Vec<(u8, u8)> = self
            .spell_slots
            .iter()
            .map(|(level, slots)| (*level, slots.maximum()))
            .collect();
        for (level, max_slots) in slots {
            self.spell_slots
                .insert(level, SpellSlots::new(max_slots, max_slots));
        }
    }

    pub fn set_max_prepared_spells(&mut self, max: usize) {
        self.max_prepared_spells = max;
    }

    pub fn max_prepared_spells(&self) -> usize {
        self.max_prepared_spells
    }

    pub fn prepared_spells(&self) -> &HashSet<SpellId> {
        &self.prepared_spells
    }

    pub fn is_spell_prepared(&self, spell_id: &SpellId) -> bool {
        self.prepared_spells.contains(spell_id)
    }

    /// Prepare a spell for casting. Returns true if the spell was successfully prepared.
    /// If the spell is already prepared, it will not be added again.
    pub fn prepare_spell(&mut self, spell_id: &SpellId) -> bool {
        if self.prepared_spells.len() < self.max_prepared_spells {
            if self.spells_by_spell_id.contains(spell_id) {
                self.prepared_spells.insert(spell_id.clone());
                return true;
            }
        }
        false
    }

    /// Unprepare a spell, removing it from the prepared spells.
    /// Returns true if the spell was successfully unprepared.
    pub fn unprepare_spell(&mut self, spell_id: &SpellId) -> bool {
        if self.prepared_spells.remove(spell_id) {
            return true;
        }
        false
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
            if let Some(slots) = self.spell_slots.get(&level) {
                let slots = if use_current_slots {
                    slots.current()
                } else {
                    slots.maximum()
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
    fn available_actions(&self) -> ActionMap {
        self.action_map_from_slots(true)
    }

    fn all_actions(&self) -> ActionMap {
        self.action_map_from_slots(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_has_spell() {
        let spell_id = &registry::spells::MAGIC_MISSILE_ID;

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id, Ability::Intelligence);

        assert!(spellbook.has_spell(spell_id));
        assert_eq!(
            spellbook.spellcasting_ability(spell_id),
            Some(&Ability::Intelligence)
        );
    }

    #[test]
    fn test_remove_spell() {
        let spell_id = &registry::spells::MAGIC_MISSILE_ID;

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id, Ability::Wisdom);
        assert!(spellbook.has_spell(spell_id));

        spellbook.remove_spell(spell_id);
        assert!(!spellbook.has_spell(spell_id));
        assert!(spellbook.spellcasting_ability(spell_id).is_none());
    }

    #[test]
    fn test_prepare_and_unprepare_spell() {
        let spell_id = &registry::spells::MAGIC_MISSILE_ID;

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id, Ability::Charisma);
        spellbook.set_max_prepared_spells(1);

        assert!(spellbook.prepare_spell(spell_id));
        assert!(spellbook.is_spell_prepared(spell_id));
        assert!(!spellbook.prepare_spell(spell_id)); // Already prepared

        assert!(spellbook.unprepare_spell(spell_id));
        assert!(!spellbook.is_spell_prepared(spell_id));
    }

    #[test]
    fn test_spell_slots_update_and_use_restore() {
        let spell_id = &registry::spells::MAGIC_MISSILE_ID;

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id, Ability::Intelligence);
        spellbook.update_spell_slots(3); // Should give [4,2]

        let slots_lvl1 = spellbook.spell_slots_for_level(1);
        assert_eq!(slots_lvl1.current(), 4);
        assert_eq!(slots_lvl1.maximum(), 4);

        assert!(spellbook.use_spell_slot(1));
        assert_eq!(spellbook.spell_slots_for_level(1).current(), 3);

        spellbook.restore_spell_slot(1);
        assert_eq!(spellbook.spell_slots_for_level(1).current(), 4);

        spellbook.use_spell_slot(2);
        spellbook.use_spell_slot(2);
        assert!(!spellbook.use_spell_slot(2)); // Only 2 slots at level 2

        spellbook.restore_all_spell_slots();
        assert_eq!(spellbook.spell_slots_for_level(2).current(), 2);
    }

    #[test]
    fn test_all_spells_and_prepared_spells() {
        let spell_id1 = &registry::spells::MAGIC_MISSILE_ID;
        let spell_id2 = &registry::spells::FIREBALL_ID;

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id1, Ability::Intelligence);
        spellbook.add_spell(spell_id2, Ability::Wisdom);

        let all_spells: HashSet<_> = spellbook.all_spells().clone();
        assert!(all_spells.contains(spell_id1));
        assert!(all_spells.contains(spell_id2));

        spellbook.set_max_prepared_spells(2);
        spellbook.prepare_spell(spell_id1);
        spellbook.prepare_spell(spell_id2);

        let prepared: HashSet<_> = spellbook.prepared_spells().clone();
        assert!(prepared.contains(spell_id1));
        assert!(prepared.contains(spell_id2));
    }
}

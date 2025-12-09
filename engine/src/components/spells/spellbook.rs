use std::collections::{HashMap, HashSet};

use crate::{
    components::{
        ability::Ability,
        actions::action::{ActionContext, ActionMap, ActionProvider},
        id::{ResourceId, SpellId},
        resource::ResourceAmount,
    },
    registry::{self, registry::SpellsRegistry},
    systems,
};

#[derive(Debug, Clone)]
pub struct Spellbook {
    /// Set of learned spells
    spells: HashSet<SpellId>,
    /// Spells that are currently prepared for casting.
    // TODO: Default prepared spells?
    prepared_spells: HashSet<SpellId>,
    /// Maximum number of spells that can be prepared.
    max_prepared_spells: usize,
    /// The ability to use when casting the spell. This depends on the class the
    /// spell was learned as
    spellcasting_ability: HashMap<SpellId, Ability>,
}

impl Spellbook {
    pub fn new() -> Self {
        Self {
            spells: HashSet::new(),
            prepared_spells: HashSet::new(),
            max_prepared_spells: 0,
            spellcasting_ability: HashMap::new(),
        }
    }

    pub fn add_spell(&mut self, spell_id: &SpellId, spellcasting_ability: Ability) {
        // TODO: Handle missing spells
        let spell = SpellsRegistry::get(spell_id)
            .expect(format!("Missing spell in registry: {} ", spell_id).as_str());
        self.spells.insert(spell_id.clone());
        self.spellcasting_ability
            .insert(spell_id.clone(), spellcasting_ability);
        if !spell.is_cantrip() && self.prepared_spells.len() < self.max_prepared_spells {
            // Automatically prepare the spell if we have space
            self.prepared_spells.insert(spell_id.clone());
        }
    }

    pub fn remove_spell(&mut self, spell_id: &SpellId) {
        self.spells.remove(spell_id);
        self.spellcasting_ability.remove(spell_id);
        self.prepared_spells.remove(spell_id);
    }

    pub fn has_spell(&self, spell_id: &SpellId) -> bool {
        self.spells.contains(spell_id)
    }

    pub fn is_empty(&self) -> bool {
        self.spells.is_empty()
    }

    pub fn all_spells(&self) -> &HashSet<SpellId> {
        &self.spells
    }

    pub fn spellcasting_ability(&self, spell_id: &SpellId) -> Option<&Ability> {
        self.spellcasting_ability.get(spell_id)
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
            if self.spells.contains(spell_id) {
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
}

impl ActionProvider for Spellbook {
    fn actions(&self) -> ActionMap {
        let mut actions = ActionMap::new();

        for spell_id in &self.spells {
            let spell = SpellsRegistry::get(spell_id)
                .expect(format!("Missing spell in registry: {} ", spell_id).as_str());

            if spell.is_cantrip() {
                let context = ActionContext::Spell {
                    id: spell_id.clone(),
                    level: 0,
                };
                actions.insert(
                    spell.action().id().clone(),
                    vec![(context, spell.action().resource_cost().clone())],
                );
                continue;
            }

            if !self.prepared_spells.contains(spell_id) {
                // Skip spells that are not prepared
                continue;
            }

            for level in spell.base_level()..=systems::spells::MAX_SPELL_LEVEL {
                let context = ActionContext::Spell {
                    id: spell_id.clone(),
                    level,
                };

                let mut resource_cost = spell.action().resource_cost().clone();
                resource_cost.insert(
                    ResourceId::from_str("resource.spell_slot"),
                    ResourceAmount::Tiered {
                        tier: level,
                        amount: 1,
                    },
                );

                actions
                    .entry(spell.action().id().clone())
                    .or_insert_with(Vec::new)
                    .push((context, resource_cost));
            }
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_has_spell() {
        let spell_id = &SpellId::from_str("spell.magic_missile");

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id, Ability::Intelligence);

        assert!(spellbook.has_spell(spell_id));
        assert_eq!(
            spellbook.spellcasting_ability(spell_id),
            Some(&Ability::Intelligence)
        );
    }

    #[test]
    fn remove_spell() {
        let spell_id = &SpellId::from_str("spell.magic_missile");

        let mut spellbook = Spellbook::new();
        spellbook.add_spell(spell_id, Ability::Wisdom);
        assert!(spellbook.has_spell(spell_id));

        spellbook.remove_spell(spell_id);
        assert!(!spellbook.has_spell(spell_id));
        assert!(spellbook.spellcasting_ability(spell_id).is_none());
    }

    #[test]
    fn prepare_and_unprepare_spell() {
        let spell_id = &SpellId::from_str("spell.magic_missile");

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
    fn all_spells_and_prepared_spells() {
        let spell_id1 = &SpellId::from_str("spell.magic_missile");
        let spell_id2 = &SpellId::from_str("spell.fireball");

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

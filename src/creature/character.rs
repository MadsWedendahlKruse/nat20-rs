use std::collections::HashMap;

use crate::{
    combat::damage::{DamageMitigationResult, DamageResistances, DamageRollResult},
    effects::effects::Effect,
    item::equipment::{
        armor::Armor,
        equipment::{EquipmentItem, GeneralEquipmentSlot, HandSlot},
        loadout::{Loadout, TryEquipError},
        weapon::{Weapon, WeaponCategory, WeaponType},
    },
    stats::{
        ability::AbilityScoreSet,
        d20_check::{execute_d20_check, D20CheckResult},
        proficiency::Proficiency,
        saving_throw::{create_saving_throw_set, SavingThrowSet},
        skill::{create_skill_set, SkillSet},
    },
};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum CharacterClass {
    Fighter,
    Rogue,
    Wizard,
    Cleric,
    // Add more as needed
}

#[derive(Debug)]
pub struct Character {
    pub name: String,
    pub class_levels: HashMap<CharacterClass, u8>,
    pub max_hp: i32,
    pub current_hp: i32,
    ability_scores: AbilityScoreSet,
    skills: SkillSet,
    saving_throws: SavingThrowSet,
    resistances: DamageResistances,
    // TODO: Might have to make this more granular later (not just martial/simple)
    // TODO: Should it just be a bool? Not sure if you can have expertise in a weapon
    pub weapon_proficiencies: HashMap<WeaponCategory, Proficiency>,
    // Equipped items
    loadout: Loadout,
    effects: Vec<Effect>,
}

impl Character {
    pub fn new(name: &str, class_levels: HashMap<CharacterClass, u8>, max_hp: i32) -> Self {
        Self {
            name: name.to_string(),
            class_levels,
            max_hp,
            current_hp: max_hp,
            ability_scores: AbilityScoreSet::new(),
            skills: create_skill_set(),
            saving_throws: create_saving_throw_set(),
            resistances: DamageResistances::new(),
            weapon_proficiencies: HashMap::new(),
            loadout: Loadout::new(),
            effects: Vec::new(),
        }
    }

    pub fn add_class_level(&mut self, class: CharacterClass, levels: u8) {
        *self.class_levels.entry(class).or_insert(0) += levels;
    }

    pub fn total_level(&self) -> u8 {
        self.class_levels.values().copied().sum()
    }

    pub fn proficiency_bonus(&self) -> i32 {
        match self.total_level() {
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            17..=20 => 6,
            _ => 2, // fallback default
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn take_damage(&mut self, damage_roll_result: &DamageRollResult) -> DamageMitigationResult {
        let mitigation_result = self.resistances.apply(damage_roll_result);
        self.current_hp = (self.current_hp - mitigation_result.total).max(0);
        mitigation_result
    }

    pub fn heal(&mut self, amount: i32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn ability_scores(&self) -> &AbilityScoreSet {
        &self.ability_scores
    }

    pub fn ability_scores_mut(&mut self) -> &mut AbilityScoreSet {
        &mut self.ability_scores
    }

    pub fn skills(&self) -> &SkillSet {
        &self.skills
    }

    pub fn skills_mut(&mut self) -> &mut SkillSet {
        &mut self.skills
    }

    pub fn saving_throws(&self) -> &SavingThrowSet {
        &self.saving_throws
    }

    pub fn saving_throws_mut(&mut self) -> &mut SavingThrowSet {
        &mut self.saving_throws
    }

    pub fn loadout(&self) -> &Loadout {
        &self.loadout
    }

    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        self.add_effects(armor.effects().clone());
        self.loadout.equip_armor(armor)
    }

    pub fn unequip_armor(&mut self) -> Option<Armor> {
        let unequiped_armor = self.loadout.unequip_armor();
        if let Some(armor) = &unequiped_armor {
            self.remove_effects(armor.effects());
        }
        unequiped_armor
    }

    pub fn equip_item(
        &mut self,
        slot: GeneralEquipmentSlot,
        item: EquipmentItem,
    ) -> Result<Option<EquipmentItem>, TryEquipError> {
        let unequipped_item = self.loadout.equip_item(slot, item)?;
        if let Some(item) = &unequipped_item {
            self.remove_effects(item.effects());
        }
        let effects = self.loadout().item_in_slot(slot).unwrap().effects().clone();
        self.add_effects(effects);
        Ok(unequipped_item)
    }

    pub fn unequip_item(&mut self, slot: GeneralEquipmentSlot) -> Option<EquipmentItem> {
        let unequipped_item = self.loadout.unequip_item(slot);
        if let Some(item) = &unequipped_item {
            self.remove_effects(item.effects());
        }
        unequipped_item
    }

    pub fn equip_weapon(
        &mut self,
        weapon: Weapon,
        hand: HandSlot,
    ) -> Result<Vec<Weapon>, TryEquipError> {
        let unequipped_weapons = self.loadout.equip_weapon(weapon, hand)?;
        for weapon in &unequipped_weapons {
            self.add_effects(weapon.effects().clone());
        }
        Ok(unequipped_weapons)
    }

    pub fn unequip_weapon(&mut self, weapon_type: &WeaponType, hand: HandSlot) -> Option<Weapon> {
        let unequipped_weapon = self.loadout.unequip_weapon(weapon_type, hand);
        if let Some(weapon) = &unequipped_weapon {
            self.remove_effects(weapon.effects());
        }
        unequipped_weapon
    }

    pub fn resistances(&self) -> &DamageResistances {
        &self.resistances
    }

    pub fn resistances_mut(&mut self) -> &mut DamageResistances {
        &mut self.resistances
    }

    pub fn effects(&self) -> &Vec<Effect> {
        &self.effects
    }

    pub fn add_effect(&mut self, effect: Effect) {
        (effect.on_apply)(self);
        self.effects.push(effect);
    }

    pub fn add_effects(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            self.add_effect(effect);
        }
    }

    pub fn remove_effect(&mut self, effect: &Effect) {
        (effect.on_unapply)(self);
        self.effects.retain(|e| e != effect);
    }

    pub fn remove_effects(&mut self, effects: &Vec<Effect>) {
        for effect in effects {
            self.remove_effect(effect);
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Character::new("John Doe", HashMap::new(), 10)
    }
}

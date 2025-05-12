use crate::combat::damage::*;
use crate::item::equipment::armor::Armor;
use crate::item::equipment::equipment::EquipmentItem;
use crate::item::equipment::equipment::GeneralEquipmentSlot;
use crate::item::equipment::equipment::HandSlot;
use crate::stats::ability::*;
use crate::stats::d20_check::*;
use crate::stats::modifier::*;
use crate::stats::proficiency::Proficiency;
use crate::stats::skill::*;

use std::collections::HashMap;

use strum::IntoEnumIterator;

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
    pub ability_scores: HashMap<Ability, AbilityScore>,
    pub skills: HashMap<Skill, D20Check>,
    pub saving_throws: HashMap<Ability, D20Check>,
    pub resistances: DamageResistances,
    armor: Option<Armor>,
    melee_weapons: HashMap<HandSlot, Option<EquipmentItem>>,
    ranged_weapons: HashMap<HandSlot, Option<EquipmentItem>>,
    equipment: HashMap<GeneralEquipmentSlot, Option<EquipmentItem>>,
}

impl Character {
    pub fn new(
        name: &str,
        class_levels: HashMap<CharacterClass, u8>,
        max_hp: i32,
        ability_scores: HashMap<Ability, AbilityScore>,
        skills: HashMap<Skill, D20Check>,
        saving_throws: HashMap<Ability, D20Check>,
        resistances: DamageResistances,
    ) -> Self {
        // Ensure all abilities and skills are initialized
        let mut ability_scores_mut = ability_scores.clone();
        for ability in Ability::iter() {
            if !ability_scores.contains_key(&ability) {
                ability_scores_mut.insert(ability, AbilityScore::default(ability));
            }
        }
        let mut skills_mut = skills.clone();
        for skill in Skill::iter() {
            if !skills.contains_key(&skill) {
                skills_mut.insert(skill, D20Check::new(Proficiency::None));
            }
        }
        let mut saving_throws_mut = saving_throws.clone();
        for ability in Ability::iter() {
            if !saving_throws.contains_key(&ability) {
                saving_throws_mut.insert(ability, D20Check::new(Proficiency::None));
            }
        }
        Self {
            name: name.to_string(),
            class_levels,
            max_hp,
            current_hp: max_hp,
            ability_scores: ability_scores_mut,
            skills: skills_mut,
            saving_throws: saving_throws_mut,
            resistances,
            armor: None,
            melee_weapons: HashMap::new(),
            ranged_weapons: HashMap::new(),
            equipment: HashMap::new(),
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

    pub fn ability(&self, ability: Ability) -> &AbilityScore {
        self.ability_scores.get(&ability).unwrap()
    }

    pub fn ability_total(&self, ability: Ability) -> i32 {
        self.ability(ability).total()
    }

    pub fn ability_modifier(&self, ability: Ability) -> ModifierSet {
        self.ability_scores
            .get(&ability)
            .map(|a| a.ability_modifier())
            .unwrap()
    }

    pub fn add_ability_modifier(
        &mut self,
        ability: Ability,
        source: ModifierSource,
        modifier: i32,
    ) {
        if let Some(ability_score) = self.ability_scores.get_mut(&ability) {
            ability_score.modifiers.add_modifier(source, modifier);
        }
    }

    pub fn remove_ability_modifier(&mut self, ability: Ability, source: &ModifierSource) {
        if let Some(ability_score) = self.ability_scores.get_mut(&ability) {
            ability_score.modifiers.remove_modifier(source);
        }
    }

    pub fn skill_check(&self, skill: Skill) -> D20CheckResult {
        let mut skill_check = self.skills.get(&skill).unwrap().clone();
        skill_check
            .modifiers
            .add_modifier_set(&self.skill_modifier(skill));
        skill_check.perform()
    }

    pub fn skill_modifier(&self, skill: Skill) -> ModifierSet {
        let skill_check = self.skills.get(&skill).unwrap();
        let ability = skill_ability(skill);
        self.ability_check_modifier_set(ability, skill_check)
    }

    pub fn add_skill_modifier(&mut self, skill: Skill, source: ModifierSource, modifier: i32) {
        if let Some(skill_check) = self.skills.get_mut(&skill) {
            skill_check.modifiers.add_modifier(source, modifier);
        }
    }

    pub fn remove_skill_modifier(&mut self, skill: Skill, source: &ModifierSource) {
        if let Some(skill_check) = self.skills.get_mut(&skill) {
            skill_check.modifiers.remove_modifier(source)
        }
    }

    pub fn saving_throw(&self, ability: Ability) -> D20CheckResult {
        let mut saving_throw_check = self.saving_throws.get(&ability).unwrap().clone();
        saving_throw_check
            .modifiers
            .add_modifier_set(&self.saving_throw_modifier(ability));
        saving_throw_check.perform()
    }

    pub fn saving_throw_modifier(&self, ability: Ability) -> ModifierSet {
        let saving_throw_check = self.saving_throws.get(&ability).unwrap();
        self.ability_check_modifier_set(ability, saving_throw_check)
    }

    fn ability_check_modifier_set(&self, ability: Ability, d20_check: &D20Check) -> ModifierSet {
        let mut modifiers = d20_check.modifiers.clone();
        modifiers.add_modifier(
            ModifierSource::Ability(ability),
            self.ability_modifier(ability).total(),
        );
        modifiers.add_modifier(
            ModifierSource::Proficiency(d20_check.proficiency),
            d20_check.proficiency.bonus(self.proficiency_bonus()),
        );
        modifiers
    }

    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        let equipped_armor = self.unequip_armor();
        armor.on_equip(self);
        self.armor = Some(armor);
        equipped_armor
    }

    pub fn unequip_armor(&mut self) -> Option<Armor> {
        if let Some(armor) = self.armor.take() {
            armor.on_unequip(self);
            Some(armor)
        } else {
            None
        }
    }

    pub fn armor_class(&self) -> ModifierSet {
        if let Some(armor) = &self.armor {
            armor.armor_class(self)
        } else {
            let mut armor_class = ModifierSet::new();
            armor_class.add_modifier(ModifierSource::Custom("Base".to_string()), 10);
            armor_class
        }
    }

    pub fn equip_item(
        &mut self,
        slot: GeneralEquipmentSlot,
        item: EquipmentItem,
    ) -> Option<EquipmentItem> {
        let equipped_item = self.unequip_item(slot);
        item.on_equip(self);
        self.equipment.insert(slot, Some(item));
        equipped_item
    }

    pub fn unequip_item(&mut self, slot: GeneralEquipmentSlot) -> Option<EquipmentItem> {
        if let Some(item) = self.equipment.remove(&slot) {
            item.as_ref().map(|i| i.on_unequip(self));
            item
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::proficiency::Proficiency;

    #[test]
    fn test_character_creation() {
        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 3);
        class_levels.insert(CharacterClass::Wizard, 2);

        let mut abilities = HashMap::new();
        abilities.insert(Ability::Strength, AbilityScore::new(Ability::Strength, 16));

        let mut skills = HashMap::new();
        skills.insert(Skill::Athletics, D20Check::new(Proficiency::Proficient));

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            abilities,
            skills,
            HashMap::new(),
            DamageResistances::new(),
        );

        assert_eq!(character.name, "Thorin");
        assert_eq!(character.max_hp, 20);
        assert_eq!(character.current_hp, 20);
    }

    #[test]
    fn test_character_total_level() {
        let mut class_levels = HashMap::new();
        class_levels.insert(CharacterClass::Fighter, 3);
        class_levels.insert(CharacterClass::Wizard, 2);

        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        assert_eq!(character.total_level(), 5);
    }

    #[test]
    fn test_character_proficiency_bonus() {
        let class_levels = HashMap::new();
        let character = Character::new(
            "Thorin",
            class_levels,
            20,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        );

        assert_eq!(character.proficiency_bonus(), 2);
    }
}

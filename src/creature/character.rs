use crate::combat::damage::*;
use crate::item::equipment::armor::Armor;
use crate::item::equipment::equipment::EquipmentItem;
use crate::item::equipment::equipment::GeneralEquipmentSlot;
use crate::item::equipment::equipment::HandSlot;
use crate::item::equipment::weapon::Weapon;
use crate::item::equipment::weapon::WeaponCategory;
use crate::item::equipment::weapon::WeaponProperties;
use crate::item::equipment::weapon::WeaponType;
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
    // TODO: Might have to make this more granular later
    // TODO: Should it just be a bool? Not sure if you can have expertise in a weapon
    weapon_proficiencies: HashMap<WeaponCategory, Proficiency>,
    // Equipped items
    armor: Option<Armor>,
    melee_weapons: HashMap<HandSlot, Option<Weapon>>,
    ranged_weapons: HashMap<HandSlot, Option<Weapon>>,
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
            weapon_proficiencies: HashMap::new(),
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

    pub fn has_weapon_in_hand(&self, weapon_type: WeaponType, hand: HandSlot) -> bool {
        self.weapon_map(weapon_type).contains_key(&hand)
    }

    pub fn weapon_in_hand(&self, weapon_type: WeaponType, hand: HandSlot) -> Option<&Weapon> {
        self.weapon_map(weapon_type)
            .get(&hand)
            .and_then(|w| w.as_ref())
    }

    pub fn equip_weapon(&mut self, weapon: Weapon, hand: HandSlot) -> Vec<Weapon> {
        let mut unequipped_weapons = Vec::new();
        if let Some(unequipped_weapon) = self.unequip_weapon(weapon.weapon_type.clone(), hand) {
            unequipped_weapons.push(unequipped_weapon);
        }
        if weapon.has_property(&WeaponProperties::TwoHanded) {
            if let Some(unequipped_weapon) =
                self.unequip_weapon(weapon.weapon_type.clone(), hand.other())
            {
                unequipped_weapons.push(unequipped_weapon);
            }
        }
        weapon.on_equip(self);
        let weapon_type = weapon.weapon_type.clone();
        self.weapon_map_mut(weapon_type).insert(hand, Some(weapon));
        unequipped_weapons
    }

    pub fn unequip_weapon(&mut self, weapon_type: WeaponType, hand: HandSlot) -> Option<Weapon> {
        if let Some(weapon) = self.weapon_map_mut(weapon_type).remove(&hand) {
            weapon.as_ref().map(|w| w.on_unequip(self));
            weapon
        } else {
            None
        }
    }

    pub fn attack_roll(&self, weapon: &Weapon) -> D20CheckResult {
        let mut attack_roll = D20Check::new(
            self.weapon_proficiencies
                .get(&weapon.category)
                .unwrap_or(&Proficiency::None)
                .clone(),
        );
        // TODO: Effect hook to determine advantage/disadvantage

        let ability = weapon.determine_ability(self);
        attack_roll.modifiers.add_modifier(
            ModifierSource::Ability(ability),
            self.ability_modifier(ability).total(),
        );

        attack_roll.perform()
    }

    fn weapon_map(&self, weapon_type: WeaponType) -> &HashMap<HandSlot, Option<Weapon>> {
        match weapon_type {
            WeaponType::Melee => &self.melee_weapons,
            WeaponType::Ranged => &self.ranged_weapons,
        }
    }

    fn weapon_map_mut(
        &mut self,
        weapon_type: WeaponType,
    ) -> &mut HashMap<HandSlot, Option<Weapon>> {
        match weapon_type {
            WeaponType::Melee => &mut self.melee_weapons,
            WeaponType::Ranged => &mut self.ranged_weapons,
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        Character::new(
            "John Doe",
            HashMap::new(),
            10,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            DamageResistances::new(),
        )
    }
}

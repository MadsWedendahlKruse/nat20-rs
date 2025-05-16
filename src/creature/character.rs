use crate::combat::damage::*;
use crate::effects::effects::Effect;
use crate::item::equipment::armor::Armor;
use crate::item::equipment::equipment::EquipmentItem;
use crate::item::equipment::equipment::EquipmentSlot;
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
use crate::stats::saving_throw::create_saving_throw_set;
use crate::stats::saving_throw::SavingThrowSet;
use crate::stats::skill::*;

use std::collections::HashMap;

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
    pub ability_scores: AbilityScoreSet,
    skills: SkillSet,
    saving_throws: SavingThrowSet,
    pub resistances: DamageResistances,
    // TODO: Might have to make this more granular later (not just martial/simple)
    // TODO: Should it just be a bool? Not sure if you can have expertise in a weapon
    pub weapon_proficiencies: HashMap<WeaponCategory, Proficiency>,
    // Equipped items
    armor: Option<Armor>,
    melee_weapons: HashMap<HandSlot, Option<Weapon>>,
    ranged_weapons: HashMap<HandSlot, Option<Weapon>>,
    equipment: HashMap<GeneralEquipmentSlot, Option<EquipmentItem>>,
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
            armor: None,
            melee_weapons: HashMap::new(),
            ranged_weapons: HashMap::new(),
            equipment: HashMap::new(),
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

    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        let unequipped_armor = self.unequip_armor();
        armor.on_equip(self);
        self.armor = Some(armor);
        unequipped_armor
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
            armor_class.add_modifier(ModifierSource::Custom("Unarmored".to_string()), 10);
            armor_class
        }
    }

    pub fn equip_item(
        &mut self,
        slot: GeneralEquipmentSlot,
        item: EquipmentItem,
    ) -> Option<EquipmentItem> {
        if !item.kind.can_equip_in_slot(EquipmentSlot::General(slot)) {
            return None;
        }

        let unequipped_item = self.unequip_item(slot);
        item.on_equip(self);
        self.equipment.insert(slot, Some(item));
        unequipped_item
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

    pub fn attack_roll(&self, weapon_type: WeaponType, hand: HandSlot) -> D20CheckResult {
        // TODO: Unarmed attacks
        let mut attack_roll = self
            .weapon_in_hand(weapon_type, hand)
            .unwrap()
            .attack_roll(self);
        for effect in &self.effects {
            (effect.pre_attack_roll)(self, &mut attack_roll)
        }
        let mut result = attack_roll.perform(self.proficiency_bonus());
        for effect in &self.effects {
            (effect.post_attack_roll)(self, &mut result)
        }
        result
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

    pub fn effects(&self) -> &Vec<Effect> {
        &self.effects
    }

    pub fn add_effect(&mut self, effect: Effect) {
        (effect.on_apply)(self);
        self.effects.push(effect);
    }

    pub fn remove_effect(&mut self, effect: &Effect) {
        (effect.on_unapply)(self);
        self.effects.retain(|e| e != effect);
    }
}

impl Default for Character {
    fn default() -> Self {
        Character::new("John Doe", HashMap::new(), 10)
    }
}

use std::collections::HashMap;

use crate::creature::character::Character;
use crate::effects::effects::Effect;
use crate::item::equipment::armor::Armor;
use crate::item::equipment::equipment::*;
use crate::item::equipment::weapon::{Weapon, WeaponProperties, WeaponType};
use crate::stats::d20_check::{execute_d20_check, D20CheckResult};
use crate::stats::modifier::{ModifierSet, ModifierSource};

#[derive(Debug)]
pub enum TryEquipError {
    InvalidSlot,
    SlotOccupied,
    NotProficient,
    WrongWeaponType,
}

#[derive(Debug, Default)]
pub struct Loadout {
    pub armor: Option<Armor>,
    pub melee_weapons: HashMap<HandSlot, Option<Weapon>>,
    pub ranged_weapons: HashMap<HandSlot, Option<Weapon>>,
    pub equipment: HashMap<GeneralEquipmentSlot, Option<EquipmentItem>>,
}

impl Loadout {
    pub fn new() -> Self {
        Self {
            armor: None,
            melee_weapons: HashMap::new(),
            ranged_weapons: HashMap::new(),
            equipment: HashMap::new(),
        }
    }

    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        let unequipped = self.armor.take();
        self.armor = Some(armor);
        unequipped
    }

    pub fn unequip_armor(&mut self) -> Option<Armor> {
        if let Some(armor) = self.armor.take() {
            Some(armor)
        } else {
            None
        }
    }

    pub fn armor_class(&self, character: &Character) -> ModifierSet {
        if let Some(armor) = &self.armor {
            armor.armor_class(character)
        } else {
            // TODO: Not sure if this is the right way to handle unarmored characters
            let mut armor_class = ModifierSet::new();
            armor_class.add_modifier(ModifierSource::Custom("Unarmored".to_string()), 10);
            armor_class
        }
    }

    pub fn equip_item(
        &mut self,
        slot: GeneralEquipmentSlot,
        item: EquipmentItem,
    ) -> Result<Option<EquipmentItem>, TryEquipError> {
        let equip_slot = EquipmentSlot::General(slot);
        if !item.kind.can_equip_in_slot(equip_slot) {
            return Err(TryEquipError::InvalidSlot);
        }
        let unequipped = self.unequip_item(slot);
        self.equipment.insert(slot, Some(item));
        Ok(unequipped)
    }

    pub fn unequip_item(&mut self, slot: GeneralEquipmentSlot) -> Option<EquipmentItem> {
        if let Some(item) = self.equipment.remove(&slot) {
            item
        } else {
            None
        }
    }

    pub fn item_in_slot(&self, slot: GeneralEquipmentSlot) -> Option<&EquipmentItem> {
        self.equipment.get(&slot).and_then(|w| w.as_ref())
    }

    pub fn equip_weapon(
        &mut self,
        weapon: Weapon,
        hand: HandSlot,
    ) -> Result<Vec<Weapon>, TryEquipError> {
        let mut unequipped = Vec::new();

        if weapon.weapon_type != WeaponType::Melee && weapon.weapon_type != WeaponType::Ranged {
            return Err(TryEquipError::WrongWeaponType);
        }

        if let Some(existing) = self.unequip_weapon(&weapon.weapon_type, hand) {
            unequipped.push(existing);
        }

        if weapon.has_property(&WeaponProperties::TwoHanded) {
            if let Some(existing) = self.unequip_weapon(&weapon.weapon_type, hand.other()) {
                unequipped.push(existing);
            }
        }

        self.weapon_map_mut(&weapon.weapon_type)
            .insert(hand, Some(weapon));
        Ok(unequipped)
    }

    pub fn unequip_weapon(&mut self, weapon_type: &WeaponType, hand: HandSlot) -> Option<Weapon> {
        if let Some(weapon) = self.weapon_map_mut(weapon_type).remove(&hand) {
            weapon
        } else {
            None
        }
    }

    pub fn weapon_map(&self, weapon_type: &WeaponType) -> &HashMap<HandSlot, Option<Weapon>> {
        match weapon_type {
            WeaponType::Melee => &self.melee_weapons,
            WeaponType::Ranged => &self.ranged_weapons,
        }
    }

    pub fn weapon_map_mut(
        &mut self,
        weapon_type: &WeaponType,
    ) -> &mut HashMap<HandSlot, Option<Weapon>> {
        match weapon_type {
            WeaponType::Melee => &mut self.melee_weapons,
            WeaponType::Ranged => &mut self.ranged_weapons,
        }
    }

    pub fn weapon_in_hand(&self, weapon_type: &WeaponType, hand: HandSlot) -> Option<&Weapon> {
        self.weapon_map(weapon_type)
            .get(&hand)
            .and_then(|w| w.as_ref())
    }

    pub fn has_weapon_in_hand(&self, weapon_type: &WeaponType, hand: HandSlot) -> bool {
        self.weapon_map(weapon_type)
            .get(&hand)
            .map(|w| w.is_some())
            .unwrap_or(false)
    }

    pub fn attack_roll(
        &self,
        character: &Character,
        weapon_type: WeaponType,
        hand: HandSlot,
    ) -> D20CheckResult {
        // TODO: Unarmed attacks
        let attack_roll = self
            .weapon_in_hand(&weapon_type, hand)
            .unwrap()
            .attack_roll(character);

        execute_d20_check(
            attack_roll,
            character,
            &character.effects(),
            |hook, character, check| (hook.pre_attack_roll)(character, check),
            |hook, character, result| (hook.post_attack_roll)(character, result),
        )
    }
}

use std::fmt::Debug;

use crate::creature::character::Character;
use crate::effects::effects::Effect;
use crate::item::item::{Item, ItemRarity};

// Armor and weapons behave differently compared to other equipment, so they need special handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Armor,
    General(GeneralEquipmentSlot),
    Melee(HandSlot),
    Ranged(HandSlot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneralEquipmentSlot {
    Headwear,
    Cloak,
    Gloves,
    Boots,
    Amulet,
    Ring(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandSlot {
    Main,
    Off,
}

impl HandSlot {
    pub fn other(&self) -> HandSlot {
        match self {
            HandSlot::Main => HandSlot::Off,
            HandSlot::Off => HandSlot::Main,
        }
    }
}

pub struct EquipmentItem {
    pub item: Item,
    pub kind: EquipmentType,
    effects: Vec<Effect>,
}

impl EquipmentItem {
    pub fn new(
        name: String,
        description: String,
        weight: f32,
        value: u32,
        rarity: ItemRarity,
        kind: EquipmentType,
    ) -> Self {
        Self {
            item: Item {
                name,
                description,
                weight,
                value,
                rarity,
            },
            kind,
            effects: Vec::new(),
        }
    }

    pub fn on_equip(&self, character: &mut Character) {
        for effect in &self.effects {
            character.add_effect(effect.clone());
        }
    }

    pub fn on_unequip(&self, character: &mut Character) {
        for effect in &self.effects {
            character.remove_effect(effect);
        }
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    // TODO: Not sure if it's actually needed to remove effects from equipment.
    pub fn remove_effect(&mut self, effect: &Effect) {
        self.effects.retain(|e| e != effect);
    }
}

use std::fmt;

impl fmt::Debug for EquipmentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EquipmentItem")
            .field("item", &self.item)
            .field("kind", &self.kind)
            // .field("on_equip", &"fn(...)")
            // .field("on_unequip", &"fn(...)")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum EquipmentType {
    Headwear,
    Cloak,
    Armor,
    Gloves,
    Boots,
    Amulet,
    Ring,
    MeleeWeapon,
    RangedWeapon,
}

impl EquipmentType {
    pub fn can_equip_in_slot(&self, slot: EquipmentSlot) -> bool {
        match (self, slot) {
            (EquipmentType::Headwear, EquipmentSlot::General(GeneralEquipmentSlot::Headwear)) => {
                true
            }
            (EquipmentType::Cloak, EquipmentSlot::General(GeneralEquipmentSlot::Cloak)) => true,
            (EquipmentType::Armor, EquipmentSlot::Armor) => true,
            (EquipmentType::Gloves, EquipmentSlot::General(GeneralEquipmentSlot::Gloves)) => true,
            (EquipmentType::Boots, EquipmentSlot::General(GeneralEquipmentSlot::Boots)) => true,
            (EquipmentType::Amulet, EquipmentSlot::General(GeneralEquipmentSlot::Amulet)) => true,
            (EquipmentType::Ring, EquipmentSlot::General(GeneralEquipmentSlot::Ring(0)))
            | (EquipmentType::Ring, EquipmentSlot::General(GeneralEquipmentSlot::Ring(1))) => true,
            (EquipmentType::MeleeWeapon, EquipmentSlot::Melee(_)) => true,
            (EquipmentType::RangedWeapon, EquipmentSlot::Ranged(_)) => true,
            _ => false,
        }
    }
}

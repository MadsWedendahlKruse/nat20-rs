use std::fmt::Debug;

use strum::{Display, EnumIter};

use crate::components::{
    id::EffectId,
    items::item::{Item, ItemRarity},
};

// Armor and weapons behave differently compared to other equipment, so they need special handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum EquipmentSlot {
    Armor,
    General(GeneralEquipmentSlot),
    Melee(HandSlot),
    Ranged(HandSlot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display)]
pub enum GeneralEquipmentSlot {
    Headwear,
    Cloak,
    Gloves,
    Boots,
    Amulet,
    Ring(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentItem {
    pub item: Item,
    pub kind: EquipmentType,
    effects: Vec<EffectId>,
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

    pub fn add_effect(&mut self, effect: EffectId) {
        self.effects.push(effect);
    }

    // TODO: Not sure if it's actually needed to remove effects from equipment.
    pub fn remove_effect(&mut self, effect: &EffectId) {
        self.effects.retain(|e| e != effect);
    }

    pub fn effects(&self) -> &Vec<EffectId> {
        &self.effects
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    pub fn valid_slots(&self) -> Vec<EquipmentSlot> {
        match self {
            EquipmentType::Headwear => vec![EquipmentSlot::General(GeneralEquipmentSlot::Headwear)],
            EquipmentType::Cloak => vec![EquipmentSlot::General(GeneralEquipmentSlot::Cloak)],
            EquipmentType::Armor => vec![EquipmentSlot::Armor],
            EquipmentType::Gloves => vec![EquipmentSlot::General(GeneralEquipmentSlot::Gloves)],
            EquipmentType::Boots => vec![EquipmentSlot::General(GeneralEquipmentSlot::Boots)],
            EquipmentType::Amulet => vec![EquipmentSlot::General(GeneralEquipmentSlot::Amulet)],
            EquipmentType::Ring => vec![
                EquipmentSlot::General(GeneralEquipmentSlot::Ring(0)),
                EquipmentSlot::General(GeneralEquipmentSlot::Ring(1)),
            ],
            EquipmentType::MeleeWeapon => vec![EquipmentSlot::Melee(HandSlot::Main)],
            EquipmentType::RangedWeapon => vec![EquipmentSlot::Ranged(HandSlot::Main)],
        }
    }
}

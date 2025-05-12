use std::fmt::Debug;

use crate::creature::character::Character;
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

pub struct EquipmentItem {
    pub item: Item,
    pub kind: EquipmentType,
    on_equip: Vec<Box<dyn Fn(&mut Character)>>,
    on_unequip: Vec<Box<dyn Fn(&mut Character)>>,
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
            on_equip: Vec::new(),
            on_unequip: Vec::new(),
        }
    }

    pub fn on_equip(&self, character: &mut Character) {
        for func in &self.on_equip {
            func(character);
        }
    }

    pub fn add_on_equip<F>(&mut self, func: F)
    where
        F: Fn(&mut Character) + 'static,
    {
        self.on_equip.push(Box::new(func));
    }

    pub fn on_unequip(&self, character: &mut Character) {
        for func in &self.on_unequip {
            func(character);
        }
    }

    pub fn add_on_unequip<F>(&mut self, func: F)
    where
        F: Fn(&mut Character) + 'static,
    {
        self.on_unequip.push(Box::new(func));
    }
}

use std::fmt;

impl fmt::Debug for EquipmentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EquipmentItem")
            .field("item", &self.item)
            .field("kind", &self.kind)
            .field("on_equip", &"fn(...)")
            .field("on_unequip", &"fn(...)")
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
    // MeleeWeapon,
    // RangedWeapon,
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
            // EquipmentType::MeleeWeapon => vec![EquipmentSlot::Melee(HandSlot::Main)],
        }
    }
}

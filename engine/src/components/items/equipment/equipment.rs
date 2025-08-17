use std::fmt::Debug;

use crate::components::{
    id::EffectId,
    items::{
        equipment::slots::{EquipmentSlot, SlotProvider},
        item::Item,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum EquipmentKind {
    Headwear,
    Cloak,
    Gloves,
    Boots,
    Amulet,
    Ring,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentItem {
    pub item: Item,
    pub kind: EquipmentKind,
    pub effects: Vec<EffectId>,
}

impl SlotProvider for EquipmentItem {
    fn valid_slots(&self) -> &'static [EquipmentSlot] {
        match self.kind {
            EquipmentKind::Headwear => &[EquipmentSlot::Headwear],
            EquipmentKind::Cloak => &[EquipmentSlot::Cloak],
            EquipmentKind::Gloves => &[EquipmentSlot::Gloves],
            EquipmentKind::Boots => &[EquipmentSlot::Boots],
            EquipmentKind::Amulet => &[EquipmentSlot::Amulet],
            EquipmentKind::Ring => &[EquipmentSlot::Ring1, EquipmentSlot::Ring2],
        }
    }
}

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        id::EffectId,
        items::item::Item,
        modifier::{ModifierSet, ModifierSource},
    },
    registry,
};

use super::equipment::EquipmentItem;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArmorType {
    Clothing,
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone)]
pub struct Armor {
    pub equipment: EquipmentItem,
    pub armor_type: ArmorType,
    armor_class: ModifierSet,
    pub max_dexterity_bonus: i32,
    pub stealth_disadvantage: bool,
}

impl Armor {
    fn new(
        mut equipment: EquipmentItem,
        armor_type: ArmorType,
        armor_class: i32,
        max_dexterity_bonus: i32,
        stealth_disadvantage: bool,
    ) -> Armor {
        let modifier_source: ModifierSource =
            ModifierSource::Item(equipment.item.name.clone().to_string());

        let mut armor_class_modifiers = ModifierSet::new();
        armor_class_modifiers.add_modifier(modifier_source.clone(), armor_class);

        if stealth_disadvantage {
            equipment.add_effect(registry::effects::ARMOR_STEALTH_DISADVANTAGE_ID.clone());
        }

        Armor {
            equipment,
            armor_type,
            armor_class: armor_class_modifiers,
            max_dexterity_bonus,
            stealth_disadvantage,
        }
    }

    pub fn item(&self) -> &Item {
        &self.equipment.item
    }

    pub fn clothing(equipment: EquipmentItem) -> Armor {
        Armor::new(equipment, ArmorType::Clothing, 10, i32::MAX, false)
    }

    pub fn light(equipment: EquipmentItem, armor_class: i32) -> Armor {
        Armor::new(equipment, ArmorType::Light, armor_class, i32::MAX, false)
    }

    pub fn medium(equipment: EquipmentItem, armor_class: i32, stealth_disadvantage: bool) -> Armor {
        Armor::new(
            equipment,
            ArmorType::Medium,
            armor_class,
            2,
            stealth_disadvantage,
        )
    }

    pub fn heavy(equipment: EquipmentItem, armor_class: i32) -> Armor {
        Armor::new(equipment, ArmorType::Heavy, armor_class, 0, true)
    }

    pub fn armor_class(&self, ability_scores: &AbilityScoreMap) -> ModifierSet {
        if self.max_dexterity_bonus == 0 {
            return self.armor_class.clone();
        }

        let dex_mod = ability_scores
            .ability_modifier(Ability::Dexterity)
            .total()
            .min(self.max_dexterity_bonus);

        let mut armor_class_modifiers = self.armor_class.clone();
        armor_class_modifiers.add_modifier(ModifierSource::Ability(Ability::Dexterity), dex_mod);
        armor_class_modifiers
    }

    pub fn effects(&self) -> &Vec<EffectId> {
        self.equipment.effects()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::fixtures;

    use super::*;

    #[test]
    fn clothing_stats() {
        let armor = fixtures::armor::clothing();
        assert_eq!(armor.armor_type, ArmorType::Clothing);
        assert_eq!(armor.armor_class.total(), 10);
        assert_eq!(armor.max_dexterity_bonus, i32::MAX);
        assert_eq!(armor.stealth_disadvantage, false);
    }

    #[test]
    fn light_armor_stats() {
        let armor = fixtures::armor::light_armor();
        assert_eq!(armor.armor_type, ArmorType::Light);
        assert_eq!(armor.armor_class.total(), 12);
        assert_eq!(armor.max_dexterity_bonus, i32::MAX);
        assert_eq!(armor.stealth_disadvantage, false);
    }

    #[test]
    fn medium_armor_stats() {
        let armor = fixtures::armor::medium_armor();
        assert_eq!(armor.armor_type, ArmorType::Medium);
        assert_eq!(armor.armor_class.total(), 14);
        assert_eq!(armor.max_dexterity_bonus, 2);
        assert_eq!(armor.stealth_disadvantage, false);
    }

    #[test]
    fn heavy_armor_stats() {
        let armor = fixtures::armor::heavy_armor();
        assert_eq!(armor.armor_type, ArmorType::Heavy);
        assert_eq!(armor.armor_class.total(), 18);
        assert_eq!(armor.max_dexterity_bonus, 0);
        assert_eq!(armor.stealth_disadvantage, true);
    }
}

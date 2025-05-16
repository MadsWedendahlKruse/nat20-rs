use std::sync::Arc;

use crate::{
    creature::character::Character,
    effects::{
        effects::{Effect, EffectDuration},
        hooks::SkillCheckHook,
    },
    stats::{
        ability::Ability,
        d20_check::AdvantageType,
        modifier::{ModifierSet, ModifierSource},
        skill::Skill,
    },
};

use super::equipment::EquipmentItem;

#[derive(Debug, Clone, PartialEq)]
pub enum ArmorType {
    Clothing,
    Light,
    Medium,
    Heavy,
}

#[derive(Debug)]
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
        let item_name = Arc::new(equipment.item.name.clone());
        let modifier_source: ModifierSource = ModifierSource::Item(item_name.clone().to_string());

        let mut armor_class_modifiers = ModifierSet::new();
        armor_class_modifiers.add_modifier(modifier_source.clone(), armor_class);

        if stealth_disadvantage {
            let mut stealth_disadvantage_effect =
                Effect::new(modifier_source.clone(), EffectDuration::Persistent);

            let mut skill_check_hook = SkillCheckHook::new(Skill::Stealth);
            skill_check_hook.check_hook = Arc::new(move |_, d20_check| {
                d20_check
                    .advantage_tracker_mut()
                    .add(AdvantageType::Disadvantage, modifier_source.clone());
            });

            stealth_disadvantage_effect.skill_check_hook = Some(skill_check_hook);

            equipment.add_effect(stealth_disadvantage_effect);
        }

        Armor {
            equipment,
            armor_type,
            armor_class: armor_class_modifiers,
            max_dexterity_bonus,
            stealth_disadvantage,
        }
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

    pub fn armor_class(&self, character: &Character) -> ModifierSet {
        if self.max_dexterity_bonus == 0 {
            return self.armor_class.clone();
        }

        let dex_mod = character
            .ability_scores()
            .modifier(Ability::Dexterity)
            .total()
            .min(self.max_dexterity_bonus);

        let mut armor_class_modifiers = self.armor_class.clone();
        armor_class_modifiers.add_modifier(ModifierSource::Ability(Ability::Dexterity), dex_mod);
        armor_class_modifiers
    }

    pub fn on_equip(&self, character: &mut Character) {
        self.equipment.on_equip(character);
    }

    pub fn on_unequip(&self, character: &mut Character) {
        self.equipment.on_unequip(character);
    }
}

#[cfg(test)]
mod tests {
    use crate::item::{equipment::equipment::EquipmentType, item::ItemRarity};

    use super::*;

    #[test]
    fn clothing() {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Clothes".to_string(),
            "A test clothing item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        let armor = Armor::clothing(equipment);
        assert_eq!(armor.armor_type, ArmorType::Clothing);
        assert_eq!(armor.armor_class.total(), 10);
        assert_eq!(armor.max_dexterity_bonus, i32::MAX);
        assert_eq!(armor.stealth_disadvantage, false);
    }

    #[test]
    fn light_armor() {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Leather Armor".to_string(),
            "A test light armor item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        let armor = Armor::light(equipment, 12);
        assert_eq!(armor.armor_type, ArmorType::Light);
        assert_eq!(armor.armor_class.total(), 12);
        assert_eq!(armor.max_dexterity_bonus, i32::MAX);
        assert_eq!(armor.stealth_disadvantage, false);
    }

    #[test]
    fn medium_armor() {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Chain Shirt".to_string(),
            "A test medium armor item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        let armor = Armor::medium(equipment, 14, false);
        assert_eq!(armor.armor_type, ArmorType::Medium);
        assert_eq!(armor.armor_class.total(), 14);
        assert_eq!(armor.max_dexterity_bonus, 2);
        assert_eq!(armor.stealth_disadvantage, false);
    }

    #[test]
    fn heavy_armor() {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Plate Armor".to_string(),
            "A test heavy armor item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        let armor = Armor::heavy(equipment, 18);
        assert_eq!(armor.armor_type, ArmorType::Heavy);
        assert_eq!(armor.armor_class.total(), 18);
        assert_eq!(armor.max_dexterity_bonus, 0);
        assert_eq!(armor.stealth_disadvantage, true);
    }
}

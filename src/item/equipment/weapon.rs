use std::collections::{HashMap, HashSet};

use crate::{
    combat::damage::{DamageRoll, DamageRollResult},
    creature::character::Character,
    dice::dice::DiceSet,
    stats::{ability::Ability, modifier::ModifierSource},
};

use super::equipment::{EquipmentItem, EquipmentType, HandSlot};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WeaponCategory {
    Simple,
    Martial,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeaponType {
    Melee,
    Ranged,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WeaponProperties {
    // TODO: Ammunition,
    Finesse,
    Heavy,
    Light,
    // TODO: Loading,
    /// (normal range, long range).
    /// Disadvantage on attack rolls beyond normal range.
    /// Can't attack beyond long range
    /// TODO: Units? The rules use feet, but metric is superior.
    Range(u32, u32),
    Reach,
    Thrown,
    TwoHanded,
    /// Damage if wielded with two hands
    Versatile(DiceSet),
}

// These are really extra abilities, so might have to handle them differently
// pub enum MasteryProperty {
//     Cleave,
//     Graze,
//     Nick,
//     Push,
//     Sap,
//     Slow,
//     Topple,
//     Vex,
// }

#[derive(Debug)]
pub struct Weapon {
    pub equipment: EquipmentItem,
    pub category: WeaponCategory,
    pub weapon_type: WeaponType,
    pub properties: HashSet<WeaponProperties>,
    pub damage_roll: DamageRoll,
    pub enchantment: u32,
    ability: Ability,
}

impl Weapon {
    pub fn new(
        equipment: EquipmentItem,
        category: WeaponCategory,
        properties: HashSet<WeaponProperties>,
        damage_roll: DamageRoll,
        enchantment: u32,
    ) -> Self {
        let weapon_type = match equipment.kind {
            EquipmentType::MeleeWeapon => WeaponType::Melee,
            EquipmentType::RangedWeapon => WeaponType::Ranged,
            _ => panic!("Invalid weapon type"),
        };
        let ability = match weapon_type {
            WeaponType::Melee => Ability::Strength,
            WeaponType::Ranged => Ability::Dexterity,
        };
        Self {
            equipment,
            category,
            weapon_type,
            properties,
            damage_roll,
            enchantment,
            ability,
        }
    }

    pub fn has_property(&self, property: &WeaponProperties) -> bool {
        self.properties.contains(property)
    }

    pub fn damage_roll(&self, character: &Character, hand: HandSlot) -> DamageRoll {
        let mut damage_roll = self.damage_roll.clone();

        // Check if the weapon is versatile and the character is wielding it in two hands
        let versatile_dice = self.properties.iter().find_map(|prop| {
            if let WeaponProperties::Versatile(dice) = prop {
                Some(dice)
            } else {
                None
            }
        });
        if versatile_dice.is_some()
            && !character.has_weapon_in_hand(self.weapon_type.clone(), hand.other())
        {
            damage_roll.primary.dice_roll.dice = versatile_dice.unwrap().clone();
        }

        let ability = self.determine_ability(&character);
        damage_roll.primary.dice_roll.modifiers.add_modifier(
            ModifierSource::Ability(ability),
            character.ability_modifier(ability).total(),
        );
        damage_roll.primary.dice_roll.modifiers.add_modifier(
            ModifierSource::Item("Enchantment".to_string()),
            self.enchantment as i32,
        );
        damage_roll
    }

    pub fn determine_ability(&self, character: &Character) -> Ability {
        if self.has_property(&WeaponProperties::Finesse) {
            // Return the higher of the two abilities
            let str = character.ability_total(Ability::Strength);
            let dex = character.ability_total(Ability::Dexterity);
            if str > dex {
                Ability::Strength
            } else {
                Ability::Dexterity
            }
        } else {
            self.ability
        }
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
    use super::*;
    use crate::combat::damage::{DamageComponent, DamageType};
    use crate::dice::dice::{DiceSet, DiceSetRoll, DieSize};
    use crate::item::equipment::equipment::{EquipmentItem, EquipmentType};
    use crate::item::item::ItemRarity;
    use crate::stats::modifier::ModifierSet;

    #[test]
    fn create_weapon() {
        let equipment = EquipmentItem::new(
            "Longsword".to_string(),
            "A longsword".to_string(),
            5.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let damage_roll = DamageRoll {
            primary: DamageComponent {
                dice_roll: DiceSetRoll {
                    dice: DiceSet {
                        num_dice: 1,
                        die_size: DieSize::D8,
                    },
                    modifiers: ModifierSet::new(),
                    label: "Longsword".to_string(),
                },
                damage_type: DamageType::Slashing,
            },
            bonus: Vec::new(),
            label: "Longsword".to_string(),
        };
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            damage_roll,
            1,
        );

        assert_eq!(weapon.equipment.item.name, "Longsword");
        assert_eq!(weapon.category, WeaponCategory::Martial);
        assert_eq!(weapon.weapon_type, WeaponType::Melee);
        assert_eq!(weapon.properties.len(), 1);
        assert_eq!(weapon.damage_roll.primary.dice_roll.dice.num_dice, 1);
        assert_eq!(
            weapon.damage_roll.primary.dice_roll.dice.die_size,
            DieSize::D8
        );
        println!("{:?}", weapon);
    }

    #[test]
    fn incorrect_equipment_type() {
        let result = std::panic::catch_unwind(|| {
            let equipment = EquipmentItem::new(
                "Longsword".to_string(),
                "A longsword".to_string(),
                5.0,
                1,
                ItemRarity::Common,
                EquipmentType::Armor, // Incorrect type
            );
            let damage_roll = DamageRoll {
                primary: DamageComponent {
                    dice_roll: DiceSetRoll {
                        dice: DiceSet {
                            num_dice: 1,
                            die_size: DieSize::D8,
                        },
                        modifiers: ModifierSet::new(),
                        label: "Longsword".to_string(),
                    },
                    damage_type: DamageType::Slashing,
                },
                bonus: Vec::new(),
                label: "Longsword".to_string(),
            };
            Weapon::new(
                equipment,
                WeaponCategory::Martial,
                HashSet::from([WeaponProperties::Finesse]),
                damage_roll,
                1,
            );
        });
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast_ref::<&str>(),
            Some(&"Invalid weapon type")
        );
    }
}

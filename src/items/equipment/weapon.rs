use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    actions::action::{
        Action, ActionContext, ActionKind, TargetType, TargetingContext, TargetingKind,
    },
    combat::damage::{AttackRoll, DamageRoll, DamageSource, DamageType},
    creature::character::Character,
    dice::dice::{DiceSet, DieSize},
    effects::effects::Effect,
    registry,
    stats::{
        ability::Ability, d20_check::D20Check, modifier::ModifierSource, proficiency::Proficiency,
    },
    utils::id::ActionId,
};

use super::equipment::{EquipmentItem, EquipmentType, HandSlot};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WeaponCategory {
    Simple,
    Martial,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// TODO: Units? The rules use feet, but metric is (obviously ;)) superior.
    Range(u32, u32),
    Reach,
    Thrown,
    TwoHanded,
    /// Damage if wielded with two hands
    Versatile(DiceSet),
    Enchantment(u32),
}

// These are really extra abilities, so might have to handle them differently
// TODO: Handle these as weapon_actions
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
    equipment: EquipmentItem,
    pub category: WeaponCategory,
    pub weapon_type: WeaponType,
    pub properties: HashSet<WeaponProperties>,
    pub damage_roll: DamageRoll,
    ability: Ability,
    weapon_actions: Vec<Action>,
}

impl Weapon {
    pub fn new(
        equipment: EquipmentItem,
        category: WeaponCategory,
        properties: HashSet<WeaponProperties>,
        num_dice: u32,
        die_size: DieSize,
        damage_type: DamageType,
        // weapon_actions: Vec<Action>,
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
        let damage_roll = DamageRoll::new(
            num_dice,
            die_size,
            damage_type,
            DamageSource::Weapon(weapon_type.clone(), properties.clone()),
            equipment.item.name.clone(),
        );
        let weapon_actions = vec![Action {
            id: ActionId::from_str(equipment.item.name.clone() + "_attack"),
            kind: ActionKind::AttackRollDamage {
                // TODO: Some of this seems a bit circular?
                attack_roll: Arc::new(|character: &Character, action_context: &ActionContext| {
                    let (weapon_type, hand) = match action_context {
                        ActionContext::Weapon { weapon_type, hand } => (weapon_type, hand),
                        _ => panic!("Action context must be Weapon"),
                    };
                    character
                        .loadout()
                        .weapon_in_hand(weapon_type, *hand)
                        .unwrap()
                        .attack_roll(character)
                }),
                damage: Arc::new(|character: &Character, action_context: &ActionContext| {
                    let (weapon_type, hand) = match action_context {
                        ActionContext::Weapon { weapon_type, hand } => (weapon_type, hand),
                        _ => panic!("Action context must be Weapon"),
                    };
                    character
                        .loadout()
                        .weapon_in_hand(weapon_type, *hand)
                        .unwrap()
                        .damage_roll(character, *hand)
                }),
                damage_on_failure: None,
            },
            targeting: Arc::new(|_character: &Character, _action_context: &ActionContext| {
                TargetingContext {
                    kind: TargetingKind::Single,
                    range: 5,
                    valid_target_types: vec![TargetType::Character],
                }
            }),
            resource_cost: HashMap::from([(registry::resources::ACTION.clone(), 1)]),
        }];
        Self {
            equipment,
            category,
            weapon_type,
            properties,
            damage_roll,
            ability,
            weapon_actions,
        }
    }

    pub fn name(&self) -> &str {
        &self.equipment.item.name
    }

    pub fn has_property(&self, property: &WeaponProperties) -> bool {
        self.properties.contains(property)
    }

    pub fn attack_roll(&self, character: &Character) -> AttackRoll {
        let mut attack_roll = D20Check::new(
            character
                .weapon_proficiencies
                .get(&self.category)
                .unwrap_or(&Proficiency::None)
                .clone(),
        );

        let ability = self.determine_ability(character);
        attack_roll.add_modifier(
            ModifierSource::Ability(ability),
            character.ability_scores().ability_modifier(ability).total(),
        );

        let enchantment = self.enchantment();
        if enchantment > 0 {
            attack_roll.add_modifier(
                ModifierSource::Item("Enchantment".to_string()),
                enchantment as i32,
            );
        }

        AttackRoll::new(attack_roll, DamageSource::from_weapon(self))
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
            && !character
                .loadout()
                .has_weapon_in_hand(&self.weapon_type, hand.other())
        {
            damage_roll.primary.dice_roll.dice = versatile_dice.unwrap().clone();
        }

        let ability = self.determine_ability(&character);
        damage_roll.primary.dice_roll.modifiers.add_modifier(
            ModifierSource::Ability(ability),
            character.ability_scores().ability_modifier(ability).total(),
        );

        let enchantment = self.enchantment();
        if enchantment > 0 {
            damage_roll.primary.dice_roll.modifiers.add_modifier(
                ModifierSource::Item("Enchantment".to_string()),
                enchantment as i32,
            );
        }

        damage_roll
    }

    fn enchantment(&self) -> u32 {
        self.properties
            .iter()
            .find_map(|prop| {
                if let WeaponProperties::Enchantment(enchantment) = prop {
                    Some(*enchantment)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn determine_ability(&self, character: &Character) -> Ability {
        if self.has_property(&WeaponProperties::Finesse) {
            // Return the higher of the two abilities
            let str = character.ability_scores().total(Ability::Strength);
            let dex = character.ability_scores().total(Ability::Dexterity);
            if str > dex {
                Ability::Strength
            } else {
                Ability::Dexterity
            }
        } else {
            self.ability
        }
    }

    pub fn effects(&self) -> &Vec<Effect> {
        self.equipment.effects()
    }

    pub fn weapon_actions(&self) -> &Vec<Action> {
        &self.weapon_actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::damage::DamageType;
    use crate::dice::dice::DieSize;
    use crate::items::equipment::equipment::{EquipmentItem, EquipmentType};
    use crate::items::item::ItemRarity;
    use crate::stats::ability::AbilityScore;
    use crate::test_utils::fixtures;

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
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse, WeaponProperties::Enchantment(1)]),
            1,
            DieSize::D8,
            DamageType::Slashing,
        );

        assert_eq!(weapon.equipment.item.name, "Longsword");
        assert_eq!(weapon.category, WeaponCategory::Martial);
        assert_eq!(weapon.weapon_type, WeaponType::Melee);
        assert_eq!(weapon.properties.len(), 2);
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
            Weapon::new(
                equipment,
                WeaponCategory::Martial,
                HashSet::from([WeaponProperties::Finesse]),
                1,
                DieSize::D8,
                DamageType::Slashing,
            );
        });
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast_ref::<&str>(),
            Some(&"Invalid weapon type")
        );
    }

    #[test]
    fn weapon_has_property() {
        let equipment = EquipmentItem::new(
            "Dagger".to_string(),
            "A small dagger".to_string(),
            1.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Simple,
            HashSet::from([WeaponProperties::Finesse, WeaponProperties::Light]),
            1,
            DieSize::D4,
            DamageType::Piercing,
        );
        assert!(weapon.has_property(&WeaponProperties::Finesse));
        assert!(weapon.has_property(&WeaponProperties::Light));
        assert!(!weapon.has_property(&WeaponProperties::Heavy));
    }

    #[test]
    fn weapon_enchantment_property() {
        let equipment = EquipmentItem::new(
            "Magic Sword".to_string(),
            "A sword with enchantment".to_string(),
            3.0,
            1,
            ItemRarity::Uncommon,
            EquipmentType::MeleeWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Enchantment(2)]),
            1,
            DieSize::D6,
            DamageType::Slashing,
        );
        assert_eq!(weapon.enchantment(), 2);
    }

    #[test]
    fn weapon_determine_ability_finesse() {
        let equipment = EquipmentItem::new(
            "Rapier".to_string(),
            "A finesse weapon".to_string(),
            2.5,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            1,
            DieSize::D8,
            DamageType::Piercing,
        );
        let mut character = fixtures::creatures::heroes::fighter();
        character.ability_scores_mut().set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 20),
        );
        assert_eq!(weapon.determine_ability(&character), Ability::Dexterity);
    }

    #[test]
    fn weapon_name_and_effects() {
        let equipment = EquipmentItem::new(
            "Warhammer".to_string(),
            "A heavy warhammer".to_string(),
            8.0,
            1,
            ItemRarity::Rare,
            EquipmentType::MeleeWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::new(),
            1,
            DieSize::D10,
            DamageType::Bludgeoning,
        );
        assert_eq!(weapon.name(), "Warhammer");
        assert!(weapon.effects().is_empty());
    }

    #[test]
    fn weapon_actions_exist() {
        let equipment = EquipmentItem::new(
            "Shortbow".to_string(),
            "A ranged weapon".to_string(),
            2.0,
            1,
            ItemRarity::Common,
            EquipmentType::RangedWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Simple,
            HashSet::new(),
            1,
            DieSize::D6,
            DamageType::Piercing,
        );
        assert!(!weapon.weapon_actions().is_empty());
    }
}

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use strum::{Display, EnumIter};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        d20_check::D20Check,
        damage::{AttackRoll, DamageRoll, DamageSource, DamageType},
        dice::{DiceSet, DieSize},
        id::{ActionId, EffectId},
        modifier::{ModifierSet, ModifierSource},
        proficiency::{Proficiency, ProficiencyLevel},
    },
    registry,
};

use super::equipment::{EquipmentItem, EquipmentType};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
pub enum WeaponCategory {
    Simple,
    Martial,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum WeaponType {
    Melee,
    Ranged,
}

impl Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

impl Display for WeaponProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponProperties::Enchantment(level) => write!(f, "Enchantment +{}", level),
            _ => write!(f, "{:?}", self),
        }
    }
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

#[derive(Debug, Clone)]
pub struct WeaponProficiencyMap {
    map: HashMap<WeaponCategory, Proficiency>,
}

impl WeaponProficiencyMap {
    pub fn new() -> Self {
        WeaponProficiencyMap {
            map: HashMap::new(),
        }
    }

    pub fn set_proficiency(&mut self, category: WeaponCategory, proficiency: Proficiency) {
        self.map.insert(category, proficiency);
    }

    pub fn proficiency(&self, category: &WeaponCategory) -> Proficiency {
        self.map.get(category).cloned().unwrap_or(Proficiency::new(
            ProficiencyLevel::None,
            ModifierSource::None,
        ))
    }
}

const MELEE_RANGE_DEFAULT: u32 = 5;
const MELEE_RANGE_REACH: u32 = 10;

#[derive(Debug, Clone)]
pub struct Weapon {
    equipment: EquipmentItem,
    category: WeaponCategory,
    weapon_type: WeaponType,
    properties: HashSet<WeaponProperties>,
    damage_roll: DamageRoll,
    ability: Ability,
    weapon_actions: Vec<ActionId>,
}

impl Weapon {
    pub fn new(
        equipment: EquipmentItem,
        category: WeaponCategory,
        properties: HashSet<WeaponProperties>,
        damage: Vec<(u32, DieSize, DamageType)>,
        extra_weapon_actions: Vec<ActionId>,
    ) -> Self {
        let (weapon_type, ability) = match equipment.kind {
            EquipmentType::MeleeWeapon => (WeaponType::Melee, Ability::Strength),
            EquipmentType::RangedWeapon => (WeaponType::Ranged, Ability::Dexterity),
            _ => panic!("Invalid equipment kind"),
        };

        let mut weapon_actions = vec![registry::actions::WEAPON_ATTACK_ID.clone()];

        if damage.is_empty() {
            panic!("Weapon must have at least one damage type");
        }
        let (num_dice, die_size, damage_type) = damage[0];
        let source = DamageSource::Weapon(weapon_type.clone());
        let mut damage_roll = DamageRoll::new(
            num_dice,
            die_size,
            damage_type,
            source.clone(),
            equipment.item.name.clone(),
        );
        for i in 1..damage.len() {
            let (num_dice, die_size, damage_type) = damage[i];
            damage_roll.add_bonus(num_dice, die_size, damage_type, equipment.item.name.clone());
        }

        weapon_actions.extend(extra_weapon_actions);

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

    pub fn equipment(&self) -> &EquipmentItem {
        &self.equipment
    }

    pub fn category(&self) -> &WeaponCategory {
        &self.category
    }

    pub fn weapon_type(&self) -> &WeaponType {
        &self.weapon_type
    }

    pub fn properties(&self) -> &HashSet<WeaponProperties> {
        &self.properties
    }

    pub fn has_property(&self, property: &WeaponProperties) -> bool {
        self.properties.contains(property)
    }

    pub fn attack_roll(
        &self,
        ability_scores: &AbilityScoreMap,
        weapon_proficiency: &Proficiency,
    ) -> AttackRoll {
        let mut attack_roll = D20Check::new(weapon_proficiency.clone());

        self.add_ability_modifier(ability_scores, &mut attack_roll.modifiers_mut());

        let enchantment = self.enchantment();
        if enchantment > 0 {
            attack_roll.add_modifier(
                ModifierSource::Item("Enchantment".to_string()),
                enchantment as i32,
            );
        }

        AttackRoll::new(attack_roll, DamageSource::from_weapon(self))
    }

    pub fn damage_roll(
        &self,
        ability_scores: &AbilityScoreMap,
        wielding_both_hands: bool,
    ) -> DamageRoll {
        let mut damage_roll = self.damage_roll.clone();

        // Check if the weapon is versatile and the character is wielding it in two hands
        let versatile_dice = self.properties.iter().find_map(|prop| {
            if let WeaponProperties::Versatile(dice) = prop {
                Some(dice)
            } else {
                None
            }
        });
        if versatile_dice.is_some() && !wielding_both_hands {
            damage_roll.primary.dice_roll.dice = versatile_dice.unwrap().clone();
        }

        self.add_ability_modifier(ability_scores, &mut damage_roll.primary.dice_roll.modifiers);

        let enchantment = self.enchantment();
        if enchantment > 0 {
            damage_roll.primary.dice_roll.modifiers.add_modifier(
                ModifierSource::Item("Enchantment".to_string()),
                enchantment as i32,
            );
        }

        damage_roll
    }

    fn add_ability_modifier(&self, ability_scores: &AbilityScoreMap, modifiers: &mut ModifierSet) {
        let ability = self.determine_ability(ability_scores);
        modifiers.add_modifier(
            ModifierSource::Ability(ability),
            ability_scores.ability_modifier(ability).total(),
        );
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

    pub fn determine_ability(&self, ability_scores: &AbilityScoreMap) -> Ability {
        if self.has_property(&WeaponProperties::Finesse) {
            // Return the higher of the two abilities
            let str = ability_scores.total(Ability::Strength);
            let dex = ability_scores.total(Ability::Dexterity);
            if str > dex {
                Ability::Strength
            } else {
                Ability::Dexterity
            }
        } else {
            self.ability
        }
    }

    // TODO: Can this be done in a nicer way?
    /// Returns the normal range and the long range of the weapon.
    /// When attacking a target beyond normal range, you have Disadvantage on the
    /// attack roll. You can’t attack a target beyond the long range.
    /// Note that for melee weapons the normal and long range is the same.
    pub fn range(&self) -> (u32, u32) {
        for property in &self.properties {
            match property {
                WeaponProperties::Range(normal_range, long_range) => {
                    return (*normal_range, *long_range);
                }
                WeaponProperties::Reach => {
                    return (MELEE_RANGE_REACH, MELEE_RANGE_REACH);
                }
                _ => {}
            }
        }
        return (MELEE_RANGE_DEFAULT, MELEE_RANGE_DEFAULT);
    }

    pub fn effects(&self) -> &Vec<EffectId> {
        self.equipment.effects()
    }

    pub fn weapon_actions(&self) -> &Vec<ActionId> {
        &self.weapon_actions
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{ability::AbilityScore, items::item::ItemRarity};

    use super::*;

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
            vec![(1, DieSize::D8, DamageType::Slashing)],
            vec![],
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
                vec![(1, DieSize::D8, DamageType::Slashing)],
                vec![],
            );
        });
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast_ref::<&str>(),
            Some(&"Invalid equipment kind")
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
            vec![(1, DieSize::D4, DamageType::Piercing)],
            vec![],
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
            vec![(1, DieSize::D6, DamageType::Slashing)],
            vec![],
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
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
        );
        let mut ability_scores = AbilityScoreMap::new();
        ability_scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 10));
        ability_scores.set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 18),
        );
        assert_eq!(
            weapon.determine_ability(&ability_scores),
            Ability::Dexterity
        );
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
            vec![(1, DieSize::D10, DamageType::Bludgeoning)],
            vec![],
        );
        assert_eq!(weapon.equipment().item.name, "Warhammer");
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
            vec![(1, DieSize::D6, DamageType::Piercing)],
            vec![],
        );
        assert!(!weapon.weapon_actions().is_empty());
    }
}

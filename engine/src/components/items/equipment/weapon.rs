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
        items::{
            equipment::slots::{EquipmentSlot, SlotProvider},
            item::Item,
        },
        modifier::{ModifierSet, ModifierSource},
        proficiency::{Proficiency, ProficiencyLevel},
    },
    registry,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Display)]
pub enum WeaponCategory {
    Simple,
    Martial,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum WeaponKind {
    Melee,
    Ranged,
}

impl Display for WeaponKind {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    item: Item,
    category: WeaponCategory,
    kind: WeaponKind,
    properties: HashSet<WeaponProperties>,
    damage_roll: DamageRoll,
    ability: Ability,
    weapon_actions: Vec<ActionId>,
    effects: Vec<EffectId>,
}

impl Weapon {
    pub fn new(
        item: Item,
        kind: WeaponKind,
        category: WeaponCategory,
        properties: HashSet<WeaponProperties>,
        damage: Vec<(u32, DieSize, DamageType)>,
        extra_weapon_actions: Vec<ActionId>,
        effects: Vec<EffectId>,
    ) -> Self {
        if matches!(kind, WeaponKind::Ranged)
            && !properties
                .iter()
                .any(|p| matches!(p, WeaponProperties::Range(_, _)))
        {
            panic!("Ranged weapons must have a range property");
        }

        let ability = match kind {
            WeaponKind::Melee => Ability::Strength,
            WeaponKind::Ranged => Ability::Dexterity,
        };

        let mut weapon_actions = vec![registry::actions::WEAPON_ATTACK_ID.clone()];

        if damage.is_empty() {
            panic!("Weapon must have at least one damage type");
        }
        let (num_dice, die_size, damage_type) = damage[0];
        let source = DamageSource::Weapon(kind.clone());
        let mut damage_roll = DamageRoll::new(
            num_dice,
            die_size,
            damage_type,
            source.clone(),
            item.name.clone(),
        );
        for i in 1..damage.len() {
            let (num_dice, die_size, damage_type) = damage[i];
            damage_roll.add_bonus(num_dice, die_size, damage_type, item.name.clone());
        }

        weapon_actions.extend(extra_weapon_actions);

        Self {
            item,
            category,
            kind,
            properties,
            damage_roll,
            ability,
            weapon_actions,
            effects,
        }
    }

    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn category(&self) -> &WeaponCategory {
        &self.category
    }

    pub fn kind(&self) -> &WeaponKind {
        &self.kind
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
        if versatile_dice.is_some() && wielding_both_hands {
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
        &self.effects
    }

    pub fn weapon_actions(&self) -> &Vec<ActionId> {
        &self.weapon_actions
    }
}

impl SlotProvider for Weapon {
    fn valid_slots(&self) -> &'static [EquipmentSlot] {
        match self.kind {
            WeaponKind::Melee => &[EquipmentSlot::MeleeMainHand, EquipmentSlot::MeleeOffHand],
            WeaponKind::Ranged => &[EquipmentSlot::RangedMainHand, EquipmentSlot::RangedOffHand],
        }
    }

    fn required_slots(&self) -> &'static [EquipmentSlot] {
        if self.has_property(&WeaponProperties::TwoHanded) {
            match self.kind {
                WeaponKind::Melee => &[EquipmentSlot::MeleeMainHand, EquipmentSlot::MeleeOffHand],
                WeaponKind::Ranged => {
                    &[EquipmentSlot::RangedMainHand, EquipmentSlot::RangedOffHand]
                }
            }
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{ability::AbilityScore, id::ItemId, items::item::ItemRarity};

    use super::*;

    #[test]
    fn create_weapon() {
        let item = Item {
            id: ItemId::from_str("item.longsword"),
            name: "Longsword".to_string(),
            description: "A longsword".to_string(),
            weight: 5.0,
            value: 1,
            rarity: ItemRarity::Common,
        };
        let weapon = Weapon::new(
            item.clone(),
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse, WeaponProperties::Enchantment(1)]),
            vec![(1, DieSize::D8, DamageType::Slashing)],
            vec![],
            vec![],
        );

        assert_eq!(weapon.category(), &WeaponCategory::Martial);
        assert_eq!(weapon.kind(), &WeaponKind::Melee);
        assert_eq!(weapon.properties().len(), 2);
        assert_eq!(weapon.damage_roll.primary.dice_roll.dice.num_dice, 1);
        assert_eq!(
            weapon.damage_roll.primary.dice_roll.dice.die_size,
            DieSize::D8
        );
        assert_eq!(weapon.item.name, "Longsword");
        println!("{:?}", weapon);
    }

    #[test]
    #[should_panic(expected = "Ranged weapons must have a range property")]
    fn ranged_weapon_without_range_panics() {
        let item = Item {
            id: ItemId::from_str("Shortbow"),
            name: "Shortbow".to_string(),
            description: "A ranged weapon".to_string(),
            weight: 2.0,
            value: 1,
            rarity: ItemRarity::Common,
        };
        Weapon::new(
            item,
            WeaponKind::Ranged,
            WeaponCategory::Simple,
            HashSet::new(),
            vec![(1, DieSize::D6, DamageType::Piercing)],
            vec![],
            vec![],
        );
    }

    #[test]
    fn weapon_has_property() {
        let item = Item {
            id: ItemId::from_str("Dagger"),
            name: "Dagger".to_string(),
            description: "A small dagger".to_string(),
            weight: 1.0,
            value: 1,
            rarity: ItemRarity::Common,
        };
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Simple,
            HashSet::from([WeaponProperties::Finesse, WeaponProperties::Light]),
            vec![(1, DieSize::D4, DamageType::Piercing)],
            vec![],
            vec![],
        );
        assert!(weapon.has_property(&WeaponProperties::Finesse));
        assert!(weapon.has_property(&WeaponProperties::Light));
        assert!(!weapon.has_property(&WeaponProperties::Heavy));
    }

    #[test]
    fn weapon_enchantment_property() {
        let item = Item::new(
            ItemId::from_str("item.magic_sword"),
            "Magic Sword".to_string(),
            "A sword with enchantment".to_string(),
            3.0,
            1,
            ItemRarity::Uncommon,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Enchantment(2)]),
            vec![(1, DieSize::D6, DamageType::Slashing)],
            vec![],
            vec![],
        );
        assert_eq!(weapon.enchantment(), 2);
    }

    #[test]
    fn weapon_determine_ability_finesse() {
        let item = Item::new(
            ItemId::from_str("item.rapier"),
            "Rapier".to_string(),
            "A finesse weapon".to_string(),
            2.5,
            1,
            ItemRarity::Common,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
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
        let item = Item::new(
            ItemId::from_str("item.warhammer"),
            "Warhammer".to_string(),
            "A heavy warhammer".to_string(),
            8.0,
            1,
            ItemRarity::Rare,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::new(),
            vec![(1, DieSize::D10, DamageType::Bludgeoning)],
            vec![],
            vec![],
        );
        assert_eq!(weapon.item.name, "Warhammer");
        assert!(weapon.effects().is_empty());
    }

    #[test]
    fn weapon_actions_exist() {
        let item = Item::new(
            ItemId::from_str("item.shortbow"),
            "Shortbow".to_string(),
            "A ranged weapon".to_string(),
            2.0,
            1,
            ItemRarity::Common,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Ranged,
            WeaponCategory::Simple,
            HashSet::from([WeaponProperties::Range(80, 320)]),
            vec![(1, DieSize::D6, DamageType::Piercing)],
            vec![],
            vec![],
        );
        assert!(!weapon.weapon_actions().is_empty());
    }

    #[test]
    fn weapon_range_melee_default() {
        let item = Item::new(
            ItemId::from_str("item.club"),
            "Club".to_string(),
            "A simple club".to_string(),
            2.0,
            1,
            ItemRarity::Common,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Simple,
            HashSet::new(),
            vec![(1, DieSize::D4, DamageType::Bludgeoning)],
            vec![],
            vec![],
        );
        assert_eq!(weapon.range(), (5, 5));
    }

    #[test]
    fn weapon_range_melee_reach() {
        let item = Item::new(
            ItemId::from_str("item.whip"),
            "Whip".to_string(),
            "A whip with reach".to_string(),
            3.0,
            1,
            ItemRarity::Uncommon,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Reach]),
            vec![(1, DieSize::D4, DamageType::Slashing)],
            vec![],
            vec![],
        );
        assert_eq!(weapon.range(), (10, 10));
    }

    #[test]
    fn weapon_range_ranged() {
        let item = Item::new(
            ItemId::from_str("item.longbow"),
            "Longbow".to_string(),
            "A long ranged bow".to_string(),
            2.0,
            1,
            ItemRarity::Rare,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Ranged,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Range(150, 600)]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
            vec![],
        );
        assert_eq!(weapon.range(), (150, 600));
    }

    #[test]
    fn slot_provider_valid_slots_melee() {
        let item = Item::new(
            ItemId::from_str("item.axe"),
            "Axe".to_string(),
            "A hand axe".to_string(),
            2.0,
            1,
            ItemRarity::Common,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Melee,
            WeaponCategory::Simple,
            HashSet::new(),
            vec![(1, DieSize::D6, DamageType::Slashing)],
            vec![],
            vec![],
        );
        let slots = <Weapon as SlotProvider>::valid_slots(&weapon);
        assert!(slots.contains(&EquipmentSlot::MeleeMainHand));
        assert!(slots.contains(&EquipmentSlot::MeleeOffHand));
    }

    #[test]
    fn slot_provider_valid_slots_ranged() {
        let item = Item::new(
            ItemId::from_str("item.crossbow"),
            "Crossbow".to_string(),
            "A light crossbow".to_string(),
            3.0,
            1,
            ItemRarity::Uncommon,
        );
        let weapon = Weapon::new(
            item,
            WeaponKind::Ranged,
            WeaponCategory::Simple,
            HashSet::from([WeaponProperties::Range(80, 320)]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
            vec![],
        );
        let slots = <Weapon as SlotProvider>::valid_slots(&weapon);
        assert!(slots.contains(&EquipmentSlot::RangedMainHand));
        assert!(slots.contains(&EquipmentSlot::RangedOffHand));
    }
}

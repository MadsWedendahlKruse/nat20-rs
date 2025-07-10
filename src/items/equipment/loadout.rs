use std::{collections::HashMap, fmt};

use crate::{
    actions::action::{self, ActionContext, ActionProvider},
    combat::damage::AttackRollResult,
    creature::character::Character,
    items::equipment::{
        armor::Armor,
        equipment::{EquipmentItem, EquipmentSlot, GeneralEquipmentSlot, HandSlot},
        weapon::{Weapon, WeaponProperties, WeaponType},
    },
    registry,
    stats::{
        d20_check::D20CheckResult,
        modifier::{ModifierSet, ModifierSource},
    },
    utils::id::{ActionId, ResourceId},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TryEquipError {
    InvalidSlot,
    SlotOccupied,
    NotProficient,
    WrongWeaponType,
}

#[derive(Debug, Default)]
pub struct Loadout {
    pub armor: Option<Armor>,
    weapons: HashMap<WeaponType, HashMap<HandSlot, Option<Weapon>>>,
    pub equipment: HashMap<GeneralEquipmentSlot, Option<EquipmentItem>>,
}

impl Loadout {
    pub fn new() -> Self {
        Self {
            armor: None,
            weapons: HashMap::new(),
            equipment: HashMap::new(),
        }
    }

    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        let unequipped = self.armor.take();
        self.armor = Some(armor);
        unequipped
    }

    pub fn unequip_armor(&mut self) -> Option<Armor> {
        if let Some(armor) = self.armor.take() {
            Some(armor)
        } else {
            None
        }
    }

    pub fn armor_class(&self, character: &Character) -> ModifierSet {
        if let Some(armor) = &self.armor {
            let mut armor_class = armor.armor_class(character);
            for effect in character.effects() {
                (effect.on_armor_class)(&character, &mut armor_class);
            }
            armor_class
        } else {
            // TODO: Not sure if this is the right way to handle unarmored characters
            let mut armor_class = ModifierSet::new();
            armor_class.add_modifier(ModifierSource::Custom("Unarmored".to_string()), 10);
            armor_class
        }
    }

    pub fn does_attack_hit(
        &self,
        character: &Character,
        attack_roll_result: &D20CheckResult,
    ) -> bool {
        let armor_class = self.armor_class(character);
        if attack_roll_result.is_crit_fail {
            return false;
        }
        attack_roll_result.is_crit || attack_roll_result.total >= armor_class.total() as u32
    }

    pub fn equip_item(
        &mut self,
        slot: GeneralEquipmentSlot,
        item: EquipmentItem,
    ) -> Result<Option<EquipmentItem>, TryEquipError> {
        let equip_slot = EquipmentSlot::General(slot);
        if !item.kind.can_equip_in_slot(equip_slot) {
            return Err(TryEquipError::InvalidSlot);
        }
        let unequipped = self.unequip_item(slot);
        self.equipment.insert(slot, Some(item));
        Ok(unequipped)
    }

    pub fn unequip_item(&mut self, slot: GeneralEquipmentSlot) -> Option<EquipmentItem> {
        if let Some(item) = self.equipment.remove(&slot) {
            item
        } else {
            None
        }
    }

    pub fn item_in_slot(&self, slot: GeneralEquipmentSlot) -> Option<&EquipmentItem> {
        self.equipment.get(&slot).and_then(|w| w.as_ref())
    }

    pub fn equip_weapon(
        &mut self,
        weapon: Weapon,
        hand: HandSlot,
    ) -> Result<Vec<Weapon>, TryEquipError> {
        let mut unequipped = Vec::new();
        let weapon_type = weapon.weapon_type.clone();

        if let Some(existing) = self.unequip_weapon(&weapon_type, hand) {
            unequipped.push(existing);
        }

        if weapon.has_property(&WeaponProperties::TwoHanded) {
            if let Some(existing) = self.unequip_weapon(&weapon_type, hand.other()) {
                unequipped.push(existing);
            }
        }

        self.weapons
            .entry(weapon_type)
            .or_insert_with(HashMap::new)
            .insert(hand, Some(weapon));
        Ok(unequipped)
    }

    pub fn unequip_weapon(&mut self, weapon_type: &WeaponType, hand: HandSlot) -> Option<Weapon> {
        if let Some(weapon) = self
            .weapons
            .get_mut(weapon_type)
            .and_then(|w| w.remove(&hand))
        {
            weapon
        } else {
            None
        }
    }

    pub fn weapon_in_hand(&self, weapon_type: &WeaponType, hand: &HandSlot) -> Option<&Weapon> {
        self.weapons
            .get(weapon_type)
            .and_then(|w| w.get(hand))
            .and_then(|w| w.as_ref())
    }

    pub fn has_weapon_in_hand(&self, weapon_type: &WeaponType, hand: &HandSlot) -> bool {
        self.weapon_in_hand(weapon_type, hand).is_some()
    }

    pub fn is_wielding_weapon_with_both_hands(&self, weapon_type: &WeaponType) -> bool {
        if let Some(main_hand_weapon) = self.weapon_in_hand(weapon_type, &HandSlot::Main) {
            // Check that:
            // 1. The main hand weapon is two-handed or versatile.
            // 2. The off hand is empty
            // (Instead of checking for a specific Versatile(DiceSet), just check for any Versatile property)
            return (main_hand_weapon.has_property(&WeaponProperties::TwoHanded)
                || main_hand_weapon
                    .properties
                    .iter()
                    .any(|p| matches!(p, WeaponProperties::Versatile(_))))
                && !self.has_weapon_in_hand(weapon_type, &HandSlot::Off);
        }
        false
    }

    pub fn attack_roll(
        &self,
        character: &Character,
        weapon_type: &WeaponType,
        hand: &HandSlot,
    ) -> AttackRollResult {
        // TODO: Unarmed attacks
        let attack_roll = self
            .weapon_in_hand(weapon_type, hand)
            .unwrap()
            .attack_roll(character);

        // TODO: How do we handle something like Fighting Style Archery, which modifies the attack roll for only ranged weapons?

        attack_roll.roll(character)
    }
}

impl ActionProvider for Loadout {
    fn all_actions(&self) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
        let mut actions = HashMap::new();

        // TODO: There has to be a nicer way to do this
        for (weapon_type, weapon_map) in self.weapons.iter() {
            for (hand, weapon_opt) in weapon_map.iter() {
                if let Some(weapon) = weapon_opt {
                    let weapon_actions = weapon.weapon_actions();
                    for action_id in weapon_actions {
                        if let Some((action, _)) = registry::actions::ACTION_REGISTRY.get(action_id)
                        {
                            let context = ActionContext::Weapon {
                                weapon_type: weapon_type.clone(),
                                hand: *hand,
                            };
                            let resource_cost = &action.resource_cost().clone();
                            actions
                                .entry(action_id.clone())
                                .and_modify(
                                    |a: &mut (Vec<ActionContext>, HashMap<ResourceId, u8>)| {
                                        a.0.push(context.clone());
                                        a.1.extend(resource_cost.clone());
                                    },
                                )
                                .or_insert((vec![context], resource_cost.clone()));
                        }
                    }
                }
            }
        }

        actions
    }

    fn available_actions(
        &self,
    ) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
        self.all_actions()
    }
}

impl fmt::Display for Loadout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Loadout:\n")?;

        if self.weapons.is_empty() {
            write!(f, "\tNo weapons equipped\n")?;
        } else {
            for (weapon_type, weapon_map) in &self.weapons {
                write!(f, "\t{:?} Weapon(s):\n", weapon_type)?;
                for (hand, weapon) in weapon_map {
                    if let Some(w) = weapon {
                        write!(f, "\t\t{} in {:?} hand\n", w.name(), hand)?;
                    }
                }
            }
        }

        if let Some(armor) = &self.armor {
            write!(f, "\tArmor: {}\n", armor.equipment.item.name)?;
        } else {
            write!(f, "\tNo armor equipped\n")?;
        }

        if self.equipment.is_empty() {
            write!(f, "\tNo equipment items equipped\n")?;
        } else {
            for (slot, item) in &self.equipment {
                if let Some(equip_item) = item {
                    write!(f, "\t{:?}: {}\n", slot, equip_item.item.name)?;
                } else {
                    write!(f, "\t{:?}: None\n", slot)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use crate::test_utils::fixtures;

    use super::*;

    #[test]
    fn empty_loadout() {
        let loadout = Loadout::new();
        assert!(loadout.armor.is_none());
        assert!(loadout.weapons.is_empty());
        assert!(loadout.equipment.is_empty());
    }

    #[test]
    fn equip_unequip_armor() {
        let mut loadout = Loadout::new();

        let armor = fixtures::armor::heavy_armor();
        let unequipped = loadout.equip_armor(armor);
        assert!(unequipped.is_none());
        assert!(loadout.armor.is_some());

        let unequipped = loadout.unequip_armor();
        assert!(unequipped.is_some());
        assert!(loadout.armor.is_none());

        let unequipped = loadout.unequip_armor();
        assert!(unequipped.is_none());
        assert!(loadout.armor.is_none());
    }

    #[test]
    fn equip_armor_twice() {
        let mut loadout = Loadout::new();

        let armor1 = fixtures::armor::heavy_armor();
        let unequipped1 = loadout.equip_armor(armor1);
        assert!(unequipped1.is_none());
        assert!(loadout.armor.is_some());

        let armor2 = fixtures::armor::medium_armor();
        let unequipped2 = loadout.equip_armor(armor2);
        assert!(unequipped2.is_some());
        assert!(loadout.armor.is_some());
    }

    #[test]
    fn equip_unequip_item() {
        let mut loadout = Loadout::new();

        let item = fixtures::equipment::boots();
        let unequipped = loadout.equip_item(GeneralEquipmentSlot::Boots, item);
        assert!(unequipped.is_ok());
        assert!(loadout.item_in_slot(GeneralEquipmentSlot::Boots).is_some());

        let unequipped = loadout.unequip_item(GeneralEquipmentSlot::Boots);
        assert!(unequipped.is_some());
        assert!(loadout.item_in_slot(GeneralEquipmentSlot::Boots).is_none());

        let unequipped = loadout.unequip_item(GeneralEquipmentSlot::Boots);
        assert!(unequipped.is_none());
        assert!(loadout.item_in_slot(GeneralEquipmentSlot::Boots).is_none());
    }

    #[test]
    fn equip_item_twice() {
        let mut loadout = Loadout::new();

        let item1 = fixtures::equipment::boots();
        let unequipped1 = loadout.equip_item(GeneralEquipmentSlot::Boots, item1);
        assert!(unequipped1.unwrap().is_none());
        assert!(loadout.item_in_slot(GeneralEquipmentSlot::Boots).is_some());

        let item2 = fixtures::equipment::boots();
        let unequipped2 = loadout.equip_item(GeneralEquipmentSlot::Boots, item2);
        assert!(unequipped2.unwrap().is_some());
        assert!(loadout.item_in_slot(GeneralEquipmentSlot::Boots).is_some());
    }

    #[test]
    fn equip_unequip_weapon() {
        let mut loadout = Loadout::new();

        let weapon = fixtures::weapons::dagger_light();
        let unequipped = loadout.equip_weapon(weapon, HandSlot::Main);
        assert!(unequipped.is_ok());
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .is_some());

        let unequipped = loadout.unequip_weapon(&WeaponType::Melee, HandSlot::Main);
        assert!(unequipped.is_some());
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .is_none());

        let unequipped = loadout.unequip_weapon(&WeaponType::Melee, HandSlot::Main);
        assert!(unequipped.is_none());
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .is_none());
    }

    #[test]
    fn equip_weapon_twice() {
        let mut loadout = Loadout::new();

        let weapon1 = fixtures::weapons::dagger_light();
        let unequipped1 = loadout.equip_weapon(weapon1, HandSlot::Main);
        assert_eq!(unequipped1.unwrap().len(), 0);
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .is_some());

        let weapon2 = fixtures::weapons::dagger_light();
        let unequipped2 = loadout.equip_weapon(weapon2, HandSlot::Main);
        assert_eq!(unequipped2.unwrap().len(), 1);
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .is_some());
    }

    #[test]
    fn equip_two_handed_weapon_should_unequip_other_hand() {
        let mut loadout = Loadout::new();

        let weapon_main_hand = fixtures::weapons::dagger_light();
        let weapon_off_hand = fixtures::weapons::dagger_light();
        for (hand, weapon) in HashMap::from([
            (HandSlot::Main, weapon_main_hand),
            (HandSlot::Off, weapon_off_hand),
        ]) {
            let unequipped = loadout.equip_weapon(weapon, hand);
            assert!(unequipped.is_ok());
            assert!(loadout.weapon_in_hand(&WeaponType::Melee, &hand).is_some());
        }

        let weapon_two_handed = fixtures::weapons::greatsword_two_handed();
        let unequipped = loadout.equip_weapon(weapon_two_handed, HandSlot::Main);
        println!("{:?}", unequipped);
        assert!(unequipped.is_ok());
        assert_eq!(unequipped.unwrap().len(), 2);
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .is_some());
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Off)
            .is_none());
        assert!(loadout
            .weapon_in_hand(&WeaponType::Melee, &HandSlot::Main)
            .unwrap()
            .has_property(&WeaponProperties::TwoHanded));
    }

    #[test]
    fn equip_in_wrong_slot() {
        let mut loadout = Loadout::new();

        let item = fixtures::equipment::boots();
        let result = loadout.equip_item(GeneralEquipmentSlot::Headwear, item);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TryEquipError::InvalidSlot);
    }

    #[test]
    fn available_actions_no_weapons() {
        // TODO: Should return unarmed attack
        let loadout = Loadout::new();
        let actions = loadout.all_actions();
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn available_actions_melee_and_ranged_weapon() {
        let mut loadout = Loadout::new();

        let weapon1 = fixtures::weapons::dagger_light();
        loadout.equip_weapon(weapon1, HandSlot::Main).unwrap();

        let weapon2 = fixtures::weapons::longbow();
        loadout.equip_weapon(weapon2, HandSlot::Main).unwrap();

        let actions = loadout.all_actions();
        for action in &actions {
            println!("{:?}", action);
        }

        // Both melee and ranged attacks use the same ActionId, but their
        // contexts are different
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[&registry::actions::WEAPON_ATTACK_ID].0.len(), 2);
        for (_, (contexts, _)) in actions {
            for context in contexts {
                match context {
                    ActionContext::Weapon { weapon_type, hand } => {
                        if weapon_type == WeaponType::Melee {
                            assert_eq!(hand, HandSlot::Main);
                        } else if weapon_type == WeaponType::Ranged {
                            assert_eq!(hand, HandSlot::Main);
                        } else {
                            panic!("Unexpected weapon type: {:?}", weapon_type);
                        }
                    }
                    _ => panic!("Unexpected action context: {:?}", context),
                }
            }
        }
    }
}

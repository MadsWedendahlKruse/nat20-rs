use std::collections::HashMap;

use hecs::{Entity, World};
use strum::IntoEnumIterator;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionContext, ActionProvider},
        damage::{AttackRoll, AttackRollResult, DamageRoll},
        id::{ActionId, EffectId},
        items::{
            equipment::{
                armor::{Armor, ArmorClass},
                equipment::EquipmentItem,
                slots::{EquipmentSlot, SlotProvider},
                weapon::{Weapon, WeaponKind, WeaponProficiencyMap, WeaponProperties},
            },
            inventory::ItemContainer,
            item::Item,
        },
        modifier::ModifierSet,
        resource::ResourceCostMap,
    },
    registry,
    systems::{self},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TryEquipError {
    InvalidSlot {
        slot: EquipmentSlot,
        equipment: EquipmentInstance,
    },
    SlotOccupied,
    NotProficient,
    WrongWeaponType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EquipmentInstance {
    Armor(Armor),
    Weapon(Weapon),
    Equipment(EquipmentItem),
}

impl EquipmentInstance {
    pub fn effects(&self) -> &Vec<EffectId> {
        match self {
            EquipmentInstance::Armor(armor) => armor.effects(),
            EquipmentInstance::Weapon(weapon) => weapon.effects(),
            EquipmentInstance::Equipment(equipment) => &equipment.effects,
        }
    }
}

impl SlotProvider for EquipmentInstance {
    fn valid_slots(&self) -> &'static [EquipmentSlot] {
        match self {
            EquipmentInstance::Armor(armor) => armor.valid_slots(),
            EquipmentInstance::Weapon(weapon) => weapon.valid_slots(),
            EquipmentInstance::Equipment(equipment) => equipment.valid_slots(),
        }
    }

    fn required_slots(&self) -> &'static [EquipmentSlot] {
        match self {
            EquipmentInstance::Weapon(weapon) => weapon.required_slots(),
            _ => &[],
        }
    }
}

impl ItemContainer for EquipmentInstance {
    fn item(&self) -> &Item {
        match self {
            EquipmentInstance::Armor(armor) => &armor.item,
            EquipmentInstance::Weapon(weapon) => weapon.item(),
            EquipmentInstance::Equipment(equipment) => &equipment.item,
        }
    }
}

macro_rules! impl_into_equipment_instance {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl Into<EquipmentInstance> for $ty {
                fn into(self) -> EquipmentInstance {
                    EquipmentInstance::$variant(self)
                }
            }
        )*
    };
}

impl_into_equipment_instance! {
    Armor => Armor,
    Weapon => Weapon,
    EquipmentItem => Equipment,
}

#[derive(Debug, Clone, Default)]
pub struct Loadout {
    equipment: HashMap<EquipmentSlot, EquipmentInstance>,
}

impl Loadout {
    pub fn new() -> Self {
        Self {
            equipment: HashMap::new(),
        }
    }

    pub fn item_in_slot(&self, slot: &EquipmentSlot) -> Option<&EquipmentInstance> {
        self.equipment.get(slot)
    }

    pub fn unequip(&mut self, slot: &EquipmentSlot) -> Option<EquipmentInstance> {
        self.equipment.remove(slot)
    }

    pub fn unequip_slots(&mut self, slots: &[EquipmentSlot]) -> Vec<EquipmentInstance> {
        slots.iter().filter_map(|slot| self.unequip(slot)).collect()
    }

    pub fn equip_in_slot(
        &mut self,
        slot: &EquipmentSlot,
        equipment: EquipmentInstance,
    ) -> Result<Vec<EquipmentInstance>, TryEquipError> {
        if !equipment.valid_slots().contains(slot) {
            return Err(TryEquipError::InvalidSlot {
                slot: *slot,
                equipment,
            });
        }
        let mut unequipped_items = self.unequip_slots(&equipment.required_slots());
        if let Some(existing) = self.equipment.insert(*slot, equipment) {
            unequipped_items.push(existing);
        }
        Ok(unequipped_items)
    }

    pub fn find_slot_for_item(&self, equipment: &EquipmentInstance) -> EquipmentSlot {
        let valid_slots = equipment.valid_slots();
        if valid_slots.len() == 1 {
            return valid_slots[0].clone();
        }
        // If there are multiple valid slots, find an available one
        let mut avaible_slot = valid_slots
            .iter()
            .find(|slot| self.item_in_slot(slot).is_none());
        if avaible_slot.is_none() {
            // If no available slot, just use the first valid slot
            // This is a fallback and may not be ideal, but it ensures we have a slot
            avaible_slot = Some(&valid_slots[0]);
        }
        avaible_slot.unwrap().clone()
    }

    pub fn equip<T>(&mut self, equipment: T) -> Result<Vec<EquipmentInstance>, TryEquipError>
    where
        T: Into<EquipmentInstance>,
    {
        let equipment = equipment.into();
        self.equip_in_slot(&self.find_slot_for_item(&equipment), equipment)
    }

    pub fn armor(&self) -> Option<&Armor> {
        if let Some(EquipmentInstance::Armor(armor)) = self.equipment.get(&EquipmentSlot::Armor) {
            Some(armor)
        } else {
            None
        }
    }

    pub fn armor_class(&self, world: &World, entity: Entity) -> ArmorClass {
        if let Some(armor) = &self.armor() {
            let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(world, entity);
            let mut armor_class = armor.armor_class(&ability_scores);
            for effect in systems::effects::effects(world, entity).iter() {
                (effect.on_armor_class)(world, entity, &mut armor_class);
            }
            armor_class
        } else {
            // TODO: Not sure if this is the right way to handle unarmored characters
            ArmorClass {
                base: 10,
                max_dexterity_bonus: 0,
                modifiers: ModifierSet::new(),
            }
        }
    }

    pub fn does_attack_hit(
        &self,
        world: &World,
        entity: Entity,
        attack_roll_result: &AttackRollResult,
    ) -> bool {
        let armor_class = self.armor_class(world, entity);
        if attack_roll_result.roll_result.is_crit_fail {
            return false;
        }
        attack_roll_result.roll_result.is_crit
            || attack_roll_result.roll_result.total >= armor_class.total() as u32
    }

    pub fn weapon_in_hand(&self, slot: &EquipmentSlot) -> Option<&Weapon> {
        if !slot.is_weapon_slot() {
            return None;
        }
        if let Some(EquipmentInstance::Weapon(weapon)) = self.item_in_slot(slot) {
            Some(weapon)
        } else {
            None
        }
    }

    pub fn has_weapon_in_hand(&self, slot: &EquipmentSlot) -> bool {
        self.weapon_in_hand(slot).is_some()
    }

    pub fn is_wielding_weapon_with_both_hands(&self, weapon_kind: &WeaponKind) -> bool {
        let (main_hand_slot, off_hand_slot) = match weapon_kind {
            WeaponKind::Melee => (EquipmentSlot::MeleeMainHand, EquipmentSlot::MeleeOffHand),
            WeaponKind::Ranged => (EquipmentSlot::RangedMainHand, EquipmentSlot::RangedOffHand),
        };
        if let Some(main_hand_weapon) = self.weapon_in_hand(&main_hand_slot) {
            // Check that:
            // 1. The main hand weapon is two-handed or versatile.
            // 2. The off hand is empty
            // (Instead of checking for a specific Versatile(DiceSet), just check for any Versatile property)
            return (main_hand_weapon.has_property(&WeaponProperties::TwoHanded)
                || main_hand_weapon
                    .properties()
                    .iter()
                    .any(|p| matches!(p, WeaponProperties::Versatile(_))))
                && !self.has_weapon_in_hand(&off_hand_slot);
        }
        false
    }

    pub fn attack_roll(&self, world: &World, entity: Entity, slot: &EquipmentSlot) -> AttackRoll {
        // TODO: Unarmed attacks
        let weapon = self
            .weapon_in_hand(slot)
            .expect("No weapon equipped in the specified slot");
        weapon.attack_roll(
            &systems::helpers::get_component::<AbilityScoreMap>(world, entity),
            &systems::helpers::get_component::<WeaponProficiencyMap>(world, entity)
                .proficiency(&weapon.category()),
        )
    }

    pub fn damage_roll(&self, world: &World, entity: Entity, slot: &EquipmentSlot) -> DamageRoll {
        let weapon = self
            .weapon_in_hand(slot)
            .expect("No weapon equipped in the specified slot");
        weapon.damage_roll(
            &systems::helpers::get_component::<AbilityScoreMap>(world, entity),
            self.is_wielding_weapon_with_both_hands(weapon.kind()),
        )
    }
}

impl ActionProvider for Loadout {
    fn all_actions(&self) -> HashMap<ActionId, (Vec<ActionContext>, ResourceCostMap)> {
        let mut actions = HashMap::new();

        // TODO: There has to be a nicer way to do this
        for slot in EquipmentSlot::weapon_slots() {
            if let Some(weapon) = self.weapon_in_hand(slot) {
                let weapon_actions = weapon.weapon_actions();
                for action_id in weapon_actions {
                    if let Some((action, _)) = registry::actions::ACTION_REGISTRY.get(action_id) {
                        let context = ActionContext::Weapon { slot: slot.clone() };
                        let resource_cost = &action.resource_cost().clone();
                        actions
                            .entry(action_id.clone())
                            .and_modify(|a: &mut (Vec<ActionContext>, ResourceCostMap)| {
                                a.0.push(context.clone());
                                a.1.extend(resource_cost.clone());
                            })
                            .or_insert((vec![context], resource_cost.clone()));
                    }
                }
            }
        }

        actions
    }

    fn available_actions(&self) -> HashMap<ActionId, (Vec<ActionContext>, ResourceCostMap)> {
        self.all_actions()
    }
}

// impl fmt::Display for Loadout {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Loadout:\n")?;

//         if self.weapons.is_empty() {
//             write!(f, "\tNo weapons equipped\n")?;
//         } else {
//             for (weapon_type, weapon_map) in &self.weapons {
//                 write!(f, "\t{:?} Weapon(s):\n", weapon_type)?;
//                 for (hand, weapon) in weapon_map {
//                     write!(
//                         f,
//                         "\t\t{} in {:?} hand\n",
//                         weapon.equipment().item.name,
//                         hand
//                     )?;
//                 }
//             }
//         }

//         if let Some(armor) = &self.armor {
//             write!(f, "\tArmor: {}\n", armor.equipment.item.name)?;
//         } else {
//             write!(f, "\tNo armor equipped\n")?;
//         }

//         if self.equipment.is_empty() {
//             write!(f, "\tNo equipment items equipped\n")?;
//         } else {
//             for (slot, item) in &self.equipment {
//                 write!(f, "\t{:?}: {}\n", slot, item.item.name)?;
//             }
//         }

//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use crate::registry;
    use crate::test_utils::fixtures;

    use super::*;

    #[test]
    fn empty_loadout() {
        let loadout = Loadout::new();
        assert!(loadout.armor().is_none());
        assert!(loadout.equipment.is_empty());
    }

    #[test]
    fn equip_unequip_armor() {
        let mut loadout = Loadout::new();

        let armor = fixtures::armor::heavy_armor();
        let slot = EquipmentSlot::Armor;
        let unequipped = loadout.equip_in_slot(&slot, EquipmentInstance::Armor(armor.clone()));
        assert!(unequipped.unwrap().is_empty());
        assert_eq!(loadout.armor(), Some(&armor));

        let unequipped = loadout.unequip(&slot);
        assert_eq!(unequipped, Some(EquipmentInstance::Armor(armor.clone())));
        assert!(loadout.armor().is_none());

        let unequipped = loadout.unequip(&slot);
        assert!(unequipped.is_none());
        assert!(loadout.armor().is_none());
    }

    #[test]
    fn equip_armor_twice() {
        let mut loadout = Loadout::new();

        let armor1 = fixtures::armor::heavy_armor();
        let slot = EquipmentSlot::Armor;
        let unequipped1 = loadout.equip_in_slot(&slot, EquipmentInstance::Armor(armor1.clone()));
        assert!(unequipped1.unwrap().is_empty());
        assert_eq!(loadout.armor(), Some(&armor1));

        let armor2 = fixtures::armor::medium_armor();
        let unequipped2 = loadout.equip_in_slot(&slot, EquipmentInstance::Armor(armor2.clone()));
        assert!(
            unequipped2
                .unwrap()
                .contains(&EquipmentInstance::Armor(armor1))
        );
        assert_eq!(loadout.armor(), Some(&armor2));
    }

    #[test]
    fn equip_unequip_item() {
        let mut loadout = Loadout::new();

        let item = fixtures::equipment::boots();
        let slot = item.valid_slots()[0].clone();
        let unequipped = loadout.equip_in_slot(&slot, EquipmentInstance::Equipment(item.clone()));
        assert!(unequipped.unwrap().is_empty());
        assert!(loadout.item_in_slot(&slot).is_some());

        let unequipped = loadout.unequip(&slot);
        assert_eq!(unequipped, Some(EquipmentInstance::Equipment(item.clone())));
        assert!(loadout.item_in_slot(&slot).is_none());

        let unequipped = loadout.unequip(&slot);
        assert!(unequipped.is_none());
        assert!(loadout.item_in_slot(&slot).is_none());
    }

    #[test]
    fn equip_item_twice() {
        let mut loadout = Loadout::new();

        let item1 = fixtures::equipment::boots();
        let slot = EquipmentSlot::Boots;
        let unequipped1 = loadout.equip_in_slot(&slot, EquipmentInstance::Equipment(item1.clone()));
        assert!(unequipped1.unwrap().is_empty());
        assert!(loadout.item_in_slot(&slot).is_some());

        let item2 = fixtures::equipment::boots();
        let unequipped2 = loadout.equip_in_slot(&slot, EquipmentInstance::Equipment(item2.clone()));
        assert!(
            unequipped2
                .unwrap()
                .contains(&EquipmentInstance::Equipment(item1))
        );
        assert!(loadout.item_in_slot(&slot).is_some());
    }

    #[test]
    fn equip_unequip_weapon() {
        let mut loadout = Loadout::new();

        let weapon = fixtures::weapons::dagger_light();
        let slot = weapon.valid_slots()[0];
        let unequipped = loadout.equip_in_slot(&slot, EquipmentInstance::Weapon(weapon.clone()));
        assert!(unequipped.is_ok());
        assert!(loadout.weapon_in_hand(&slot).is_some());

        let unequipped = loadout.unequip(&slot);
        assert!(unequipped.is_some());
        assert!(loadout.weapon_in_hand(&slot).is_none());
    }

    #[test]
    fn equip_weapon_twice() {
        let mut loadout = Loadout::new();

        let weapon1 = fixtures::weapons::dagger_light();
        let slot = weapon1.valid_slots()[0];
        let unequipped1 = loadout.equip_in_slot(&slot, EquipmentInstance::Weapon(weapon1.clone()));
        assert_eq!(unequipped1.unwrap().len(), 0);
        assert!(loadout.weapon_in_hand(&slot).is_some());

        let weapon2 = fixtures::weapons::dagger_light();
        let unequipped2 = loadout.equip_in_slot(&slot, EquipmentInstance::Weapon(weapon2.clone()));
        assert_eq!(unequipped2.unwrap().len(), 1);
        assert!(loadout.weapon_in_hand(&slot).is_some());
    }

    #[test]
    fn equip_two_handed_weapon_should_unequip_other_hand() {
        let mut loadout = Loadout::new();

        let weapon_main_hand = fixtures::weapons::dagger_light();
        let weapon_off_hand = fixtures::weapons::dagger_light();
        let main_slot = EquipmentSlot::MeleeMainHand;
        let off_slot = EquipmentSlot::MeleeOffHand;

        let unequipped_main = loadout.equip_in_slot(
            &main_slot,
            EquipmentInstance::Weapon(weapon_main_hand.clone()),
        );
        assert!(unequipped_main.is_ok());
        assert!(loadout.weapon_in_hand(&main_slot).is_some());

        let unequipped_off = loadout.equip_in_slot(
            &off_slot,
            EquipmentInstance::Weapon(weapon_off_hand.clone()),
        );
        assert!(unequipped_off.is_ok());
        assert!(loadout.weapon_in_hand(&off_slot).is_some());

        let weapon_two_handed = fixtures::weapons::greatsword_two_handed();
        let unequipped = loadout.equip_in_slot(
            &main_slot,
            EquipmentInstance::Weapon(weapon_two_handed.clone()),
        );
        println!("{:?}", unequipped);
        assert!(unequipped.is_ok());
        // Should unequip both hands if required_slots includes both
        assert!(loadout.weapon_in_hand(&main_slot).is_some());
        assert!(loadout.weapon_in_hand(&off_slot).is_none());
    }

    #[test]
    fn equip_in_wrong_slot() {
        let mut loadout = Loadout::new();

        let item = fixtures::equipment::boots();
        // Try to equip boots in the Headwear slot, which should be invalid
        let slot = EquipmentSlot::Headwear;
        let result = loadout.equip_in_slot(&slot, EquipmentInstance::Equipment(item.clone()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            TryEquipError::InvalidSlot {
                slot,
                equipment: EquipmentInstance::Equipment(item),
            }
        );
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
        loadout.equip(weapon1);

        let weapon2 = fixtures::weapons::longbow();
        loadout.equip(weapon2);

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
                    ActionContext::Weapon { slot } => {}
                    _ => panic!("Unexpected action context: {:?}", context),
                }
            }
        }
    }
}

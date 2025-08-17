use hecs::{Entity, World};

use crate::{
    components::items::{
        equipment::{
            equipment::{EquipmentSlot, HandSlot},
            loadout::{Loadout, TryEquipError},
            weapon::{self, WeaponProperties, WeaponType},
        },
        inventory::{Inventory, ItemContainer, ItemInstance},
        item::Item,
    },
    systems,
};

pub fn equip<T>(
    world: &mut World,
    entity: Entity,
    item: T,
) -> Result<Vec<ItemInstance>, TryEquipError>
where
    T: Into<ItemInstance>,
{
    let item = item.into();
    let item_name = item.item().name.clone();

    match item {
        ItemInstance::Armor(armor) => {
            if let Some(unequippped_armor) = systems::loadout::equip_armor(world, entity, armor) {
                Ok(vec![ItemInstance::Armor(unequippped_armor)])
            } else {
                Ok(vec![])
            }
        }

        ItemInstance::Equipment(equipment) => {
            let valid_slots = equipment.kind.valid_slots();

            let slot = if valid_slots.len() == 1 {
                valid_slots[0].clone()
            } else {
                // TODO: Do something else
                valid_slots[0].clone()
            };
            let slot = match slot {
                EquipmentSlot::General(slot) => slot,
                _ => {
                    return Err(TryEquipError::InvalidSlot {
                        item: item_name,
                        slot,
                    });
                }
            };

            if let Some(unequipped_item) =
                systems::loadout::equip_item(world, entity, &slot, equipment.clone())?
            {
                Ok(vec![ItemInstance::Equipment(unequipped_item)])
            } else {
                Ok(vec![])
            }
        }

        ItemInstance::Weapon(weapon) => {
            let weapon_type = weapon.weapon_type();

            let hand = if weapon.has_property(&WeaponProperties::TwoHanded) {
                HandSlot::Main
            } else {
                if systems::helpers::get_component::<Loadout>(world, entity)
                    .has_weapon_in_hand(weapon_type, &HandSlot::Main)
                {
                    HandSlot::Off
                } else {
                    HandSlot::Main
                }
            };

            let unequipped_weapons = systems::loadout::equip_weapon(world, entity, weapon, hand)?;
            Ok(unequipped_weapons
                .into_iter()
                .map(ItemInstance::Weapon)
                .collect())
        }

        ItemInstance::Item(item) => panic!("Cannot equip a regular item: {}", item_name),
    }
}

pub fn unequip(
    world: &mut World,
    entity: Entity,
    slot: &EquipmentSlot,
) -> Result<Option<ItemInstance>, TryEquipError> {
    match slot {
        EquipmentSlot::Armor => Ok(option_to_item_instance(systems::loadout::unequip_armor(
            world, entity,
        ))),

        EquipmentSlot::Melee(hand) => Ok(option_to_item_instance(
            systems::loadout::unequip_weapon(world, entity, &WeaponType::Melee, *hand),
        )),

        EquipmentSlot::Ranged(hand) => Ok(option_to_item_instance(
            systems::loadout::unequip_weapon(world, entity, &WeaponType::Ranged, *hand),
        )),

        EquipmentSlot::General(slot) => Ok(option_to_item_instance(
            systems::loadout::unequip_item(world, entity, slot),
        )),
    }
}

fn option_to_item_instance<T>(item: Option<T>) -> Option<ItemInstance>
where
    T: Into<ItemInstance>,
{
    match item {
        Some(item) => Some(item.into()),
        None => None,
    }
}

pub fn add(world: &mut World, entity: Entity, item: ItemInstance) {
    systems::helpers::get_component_mut::<Inventory>(world, entity).add(item);
}

pub fn remove(world: &mut World, entity: Entity, index: usize) -> Option<ItemInstance> {
    systems::helpers::get_component_mut::<Inventory>(world, entity).remove(index)
}

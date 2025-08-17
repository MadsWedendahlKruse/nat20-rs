use hecs::{Entity, World};

use crate::{
    components::items::{
        equipment::{
            loadout::{EquipmentInstance, TryEquipError},
            slots::EquipmentSlot,
        },
        inventory::{Inventory, ItemInstance},
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
    let equipment: EquipmentInstance = item.into();

    let unequippped_items = systems::loadout::equip(world, entity, equipment)?;

    Ok(unequippped_items
        .iter()
        .map(|item| <EquipmentInstance as Into<ItemInstance>>::into(item.clone()))
        .collect::<Vec<ItemInstance>>())
}

pub fn unequip(world: &mut World, entity: Entity, slot: &EquipmentSlot) -> Option<ItemInstance> {
    let unequipped_item = systems::loadout::unequip(world, entity, slot);
    unequipped_item.map(|item| item.into())
}

pub fn add<T>(world: &mut World, entity: Entity, item: T)
where
    T: Into<ItemInstance>,
{
    systems::helpers::get_component_mut::<Inventory>(world, entity).add(item.into());
}

pub fn remove(world: &mut World, entity: Entity, index: usize) -> Option<ItemInstance> {
    systems::helpers::get_component_mut::<Inventory>(world, entity).remove(index)
}

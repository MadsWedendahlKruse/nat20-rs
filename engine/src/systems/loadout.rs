use hecs::{Entity, Ref, World};

use crate::{
    components::{
        actions::targeting::TargetingRange,
        items::equipment::{
            armor::ArmorClass,
            loadout::{EquipmentInstance, Loadout, TryEquipError},
            slots::EquipmentSlot,
        },
    },
    systems,
};

pub fn loadout(world: &World, entity: Entity) -> Ref<'_, Loadout> {
    systems::helpers::get_component::<Loadout>(world, entity)
}

pub fn loadout_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, Loadout> {
    systems::helpers::get_component_mut::<Loadout>(world, entity)
}

pub fn equip_in_slot<T>(
    world: &mut World,
    entity: Entity,
    slot: &EquipmentSlot,
    equipment: T,
) -> Result<Vec<EquipmentInstance>, TryEquipError>
where
    T: Into<EquipmentInstance>,
{
    let equipment = equipment.into();
    let unequipped_items = loadout_mut(world, entity).equip_in_slot(slot, equipment)?;
    for item in &unequipped_items {
        systems::effects::remove_effects(world, entity, item.effects());
    }
    let effects = loadout(world, entity)
        .item_in_slot(slot)
        .unwrap()
        .effects()
        .clone();
    systems::effects::add_effects(world, entity, &effects);
    Ok(unequipped_items)
}

pub fn equip<T>(
    world: &mut World,
    entity: Entity,
    equipment: T,
) -> Result<Vec<EquipmentInstance>, TryEquipError>
where
    T: Into<EquipmentInstance>,
{
    let equipment = equipment.into();
    // TODO: Slightly less performant than calling `equip_in_slot` directly
    let effects = equipment.effects().clone();
    let unequipped_items = loadout_mut(world, entity).equip(equipment)?;
    for item in &unequipped_items {
        systems::effects::remove_effects(world, entity, item.effects());
    }
    systems::effects::add_effects(world, entity, &effects);
    Ok(unequipped_items)
}

pub fn unequip(
    world: &mut World,
    entity: Entity,
    slot: &EquipmentSlot,
) -> Option<EquipmentInstance> {
    let unequipped_item = loadout_mut(world, entity).unequip(slot);
    if let Some(item) = &unequipped_item {
        systems::effects::remove_effects(world, entity, item.effects());
    }
    unequipped_item
}

pub fn armor_class(world: &World, entity: Entity) -> ArmorClass {
    loadout(world, entity).armor_class(world, entity)
}

pub fn can_equip(world: &World, entity: Entity, equipment: &EquipmentInstance) -> bool {
    loadout(world, entity).can_equip(equipment)
}

pub fn weapon_range(world: &World, entity: Entity, slot: &EquipmentSlot) -> Option<TargetingRange> {
    loadout(world, entity)
        .weapon_in_hand(slot)
        .map(|w| w.range())
        .cloned()
}

use hecs::{Entity, Ref, World};

use crate::{
    components::{
        items::equipment::{
            armor::Armor,
            equipment::{EquipmentItem, GeneralEquipmentSlot, HandSlot},
            loadout::{Loadout, TryEquipError},
            weapon::{Weapon, WeaponType},
        },
        modifier::ModifierSet,
    },
    systems,
};

pub fn loadout(world: &World, entity: Entity) -> Ref<'_, Loadout> {
    systems::helpers::get_component::<Loadout>(world, entity)
}

pub fn loadout_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, Loadout> {
    systems::helpers::get_component_mut::<Loadout>(world, entity)
}

pub fn equip_armor(world: &mut World, entity: Entity, armor: Armor) -> Option<Armor> {
    let unequipped_armor = loadout_mut(world, entity).equip_armor(armor.clone());
    if let Some(armor) = &unequipped_armor {
        systems::effects::remove_effects(world, entity, armor.effects());
    }
    systems::effects::add_effects(world, entity, armor.effects().clone());
    unequipped_armor
}

pub fn unequip_armor(world: &mut World, entity: Entity) -> Option<Armor> {
    let unequiped_armor = loadout_mut(world, entity).unequip_armor();
    if let Some(armor) = &unequiped_armor {
        systems::effects::remove_effects(world, entity, armor.effects());
    }
    unequiped_armor
}

pub fn armor_class(world: &World, entity: Entity) -> ModifierSet {
    loadout(world, entity).armor_class(world, entity)
}

pub fn equip_item(
    world: &mut World,
    entity: Entity,
    slot: &GeneralEquipmentSlot,
    item: EquipmentItem,
) -> Result<Option<EquipmentItem>, TryEquipError> {
    let unequipped_item = loadout_mut(world, entity).equip_item(slot, item)?;
    if let Some(item) = &unequipped_item {
        systems::effects::remove_effects(world, entity, item.effects());
    }
    let effects = loadout(world, entity)
        .item_in_slot(slot)
        .unwrap()
        .effects()
        .clone();
    systems::effects::add_effects(world, entity, effects);
    Ok(unequipped_item)
}

pub fn unequip_item(
    world: &mut World,
    entity: Entity,
    slot: &GeneralEquipmentSlot,
) -> Option<EquipmentItem> {
    let unequipped_item = loadout_mut(world, entity).unequip_item(slot);
    if let Some(item) = &unequipped_item {
        systems::effects::remove_effects(world, entity, item.effects());
    }
    unequipped_item
}

pub fn equip_weapon(
    world: &mut World,
    entity: Entity,
    weapon: Weapon,
    hand: HandSlot,
) -> Result<Vec<Weapon>, TryEquipError> {
    let weapon_type = weapon.weapon_type().clone();
    let unequipped_weapons = loadout_mut(world, entity).equip_weapon(weapon, hand)?;
    for weapon in &unequipped_weapons {
        systems::effects::remove_effects(world, entity, weapon.effects());
    }
    let effects = loadout(world, entity)
        .weapon_in_hand(&weapon_type, &hand)
        .unwrap()
        .effects()
        .clone();
    systems::effects::add_effects(world, entity, effects);
    Ok(unequipped_weapons)
}

pub fn unequip_weapon(
    world: &mut World,
    entity: Entity,
    weapon_type: &WeaponType,
    hand: HandSlot,
) -> Option<Weapon> {
    let unequipped_weapon = loadout_mut(world, entity).unequip_weapon(weapon_type, hand);
    if let Some(weapon) = &unequipped_weapon {
        systems::effects::remove_effects(world, entity, weapon.effects());
    }
    unequipped_weapon
}

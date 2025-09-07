use hecs::{Entity, World};

use crate::components::{
    damage::{AttackRoll, AttackRollResult, DamageRoll},
    items::equipment::{loadout::Loadout, slots::EquipmentSlot},
};

pub fn attack_hits(world: &World, target: Entity, attack_roll: &AttackRollResult) -> bool {
    if let Some(loadout) = world.get::<&Loadout>(target).ok() {
        loadout.does_attack_hit(world, target, attack_roll)
    } else {
        // If the target does not have a loadout, assume the attack hits?
        true
    }
}

pub fn attack_roll(world: &World, entity: Entity, slot: &EquipmentSlot) -> AttackRoll {
    if let Some(loadout) = world.get::<&Loadout>(entity).ok() {
        loadout.attack_roll(world, entity, slot)
    } else {
        panic!("Entity {:?} does not have a loadout", entity);
    }
}

pub fn attack_roll_against_target(
    world: &World,
    entity: Entity,
    slot: &EquipmentSlot,
    target: Entity,
) -> AttackRollResult {
    let mut attack_roll_result = attack_roll(world, entity, slot).roll(world, entity);
    attack_roll_result.roll_result.success = attack_hits(world, target, &attack_roll_result);
    attack_roll_result
}

pub fn damage_roll(world: &World, entity: Entity, slot: &EquipmentSlot) -> DamageRoll {
    if let Some(loadout) = world.get::<&Loadout>(entity).ok() {
        loadout.damage_roll(world, entity, slot)
    } else {
        panic!("Entity {:?} does not have a loadout", entity);
    }
}

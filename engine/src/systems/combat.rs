use hecs::{Entity, World};

use crate::components::{
    damage::{AttackRoll, AttackRollResult, DamageRoll},
    items::equipment::{equipment::HandSlot, loadout::Loadout, weapon::WeaponType},
};

pub fn attack_hits(world: &World, target: Entity, attack_roll: &AttackRollResult) -> bool {
    if let Some(loadout) = world.get::<&Loadout>(target).ok() {
        loadout.does_attack_hit(world, target, attack_roll)
    } else {
        // If the target does not have a loadout, assume the attack hits?
        true
    }
}

pub fn attack_roll(
    world: &World,
    entity: Entity,
    weapon_type: &WeaponType,
    hand: &HandSlot,
) -> AttackRoll {
    if let Some(loadout) = world.get::<&Loadout>(entity).ok() {
        loadout.attack_roll(world, entity, weapon_type, hand)
    } else {
        panic!("Entity {:?} does not have a loadout", entity);
    }
}

pub fn damage_roll(
    world: &World,
    entity: Entity,
    weapon_type: &WeaponType,
    hand: &HandSlot,
) -> DamageRoll {
    if let Some(loadout) = world.get::<&Loadout>(entity).ok() {
        loadout.damage_roll(world, entity, weapon_type, hand)
    } else {
        panic!("Entity {:?} does not have a loadout", entity);
    }
}

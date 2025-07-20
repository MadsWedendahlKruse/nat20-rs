use hecs::{Entity, World};

use crate::components::{damage::AttackRollResult, items::equipment::loadout::Loadout};

pub fn attack_hits(world: &World, target: Entity, attack_roll: &AttackRollResult) -> bool {
    if let Some(loadout) = world.get::<&Loadout>(target).ok() {
        loadout.does_attack_hit(world, target, attack_roll)
    } else {
        // If the target does not have a loadout, assume the attack hits?
        true
    }
}

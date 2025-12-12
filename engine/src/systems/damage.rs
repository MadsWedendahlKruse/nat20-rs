use hecs::Entity;

use crate::{
    components::{
        actions::action::{ActionContext, AttackRollFunction, DamageFunction},
        damage::{AttackRoll, AttackRollResult, DamageRoll, DamageRollResult},
        items::equipment::{loadout::Loadout, slots::EquipmentSlot},
    },
    engine::game_state::GameState,
    systems,
};

pub fn damage_roll(
    mut damage_roll: DamageRoll,
    game_state: &mut GameState,
    entity: Entity,
    crit: bool,
) -> DamageRollResult {
    let world = &game_state.world;

    for effect in systems::effects::effects(world, entity).iter() {
        (effect.pre_damage_roll)(world, entity, &mut damage_roll);
    }

    let mut result = damage_roll.roll_raw(crit);

    for effect in systems::effects::effects(world, entity).iter() {
        (effect.post_damage_roll)(&mut game_state.script_engines, world, entity, &mut result);
    }

    result
}

pub fn damage_roll_fn(
    damage_roll_fn: &DamageFunction,
    game_state: &mut GameState,
    entity: Entity,
    context: &ActionContext,
    crit: bool,
) -> DamageRollResult {
    let roll = damage_roll_fn(&game_state.world, entity, context);
    damage_roll(roll, game_state, entity, crit)
}

pub fn attack_roll(
    mut attack_roll: AttackRoll,
    game_state: &mut GameState,
    entity: Entity,
) -> AttackRollResult {
    let world = &game_state.world;

    for effect in systems::effects::effects(world, entity).iter() {
        (effect.pre_attack_roll)(world, entity, &mut attack_roll);
    }

    let mut result = {
        let level =
            systems::helpers::level(world, entity).expect("Entity must have a level component");
        attack_roll.roll_raw(level.proficiency_bonus())
    };

    for effect in systems::effects::effects(world, entity).iter() {
        (effect.post_attack_roll)(world, entity, &mut result);
    }

    result
}

pub fn attack_roll_fn(
    attack_roll_fn: &AttackRollFunction,
    game_state: &mut GameState,
    entity: Entity,
    context: &ActionContext,
) -> AttackRollResult {
    let roll = attack_roll_fn(&game_state.world, entity, context);
    attack_roll(roll, game_state, entity)
}

pub fn damage_roll_weapon(
    game_state: &mut GameState,
    entity: Entity,
    slot: &EquipmentSlot,
    crit: bool,
) -> DamageRollResult {
    damage_roll(
        systems::loadout::weapon_damage_roll(&game_state.world, entity, slot),
        game_state,
        entity,
        crit,
    )
}

pub fn attack_roll_weapon(
    game_state: &mut GameState,
    entity: Entity,
    slot: &EquipmentSlot,
) -> AttackRollResult {
    attack_roll(
        systems::loadout::weapon_attack_roll(&game_state.world, entity, slot),
        game_state,
        entity,
    )
}

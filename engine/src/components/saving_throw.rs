use hecs::{Entity, World};

use crate::{
    components::effects::hooks::SavingThrowHook,
    systems::{self},
};

use super::{
    ability::Ability,
    d20_check::{D20Check, D20CheckDC, D20CheckResult, D20CheckSet},
};

pub type SavingThrowSet = D20CheckSet<Ability, SavingThrowHook>;

pub fn get_saving_throw_hooks(
    ability: Ability,
    world: &World,
    entity: Entity,
) -> Vec<SavingThrowHook> {
    systems::effects::effects(world, entity)
        .iter()
        .filter_map(|e| e.on_saving_throw.clone()) // clone the hook, not the whole vec
        .filter(|hook| hook.key == ability)
        .collect()
}

pub fn apply_check_hook(hook: &SavingThrowHook, world: &World, entity: Entity, d20: &mut D20Check) {
    (hook.check_hook)(world, entity, d20);
}

pub fn apply_result_hook(
    hook: &SavingThrowHook,
    world: &World,
    entity: Entity,
    result: &mut D20CheckResult,
) {
    (hook.result_hook)(world, entity, result);
}

pub fn create_saving_throw_set() -> SavingThrowSet {
    SavingThrowSet::new(
        get_saving_throw_hooks,
        apply_check_hook,
        apply_result_hook,
        |k| k,
    )
}

pub type SavingThrowDC = D20CheckDC<Ability>;

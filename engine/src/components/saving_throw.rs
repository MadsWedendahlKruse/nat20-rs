use hecs::{Entity, World};

use crate::{
    components::{
        ability::Ability,
        d20_check::{D20CheckDC, D20CheckSet},
        effects::hooks::D20CheckHooks,
    },
    systems::{self},
};

pub type SavingThrowSet = D20CheckSet<Ability>;

pub type SavingThrowDC = D20CheckDC<Ability>;

pub fn get_saving_throw_hooks(
    ability: Ability,
    world: &World,
    entity: Entity,
) -> Vec<D20CheckHooks> {
    systems::effects::effects(world, entity)
        .iter()
        .filter_map(|e| e.on_saving_throw.get(&ability))
        .cloned()
        .collect()
}

pub fn create_saving_throw_set() -> SavingThrowSet {
    SavingThrowSet::new(|k| k, get_saving_throw_hooks)
}

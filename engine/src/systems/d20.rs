use hecs::{Entity, World};

use crate::{
    components::{
        ability::Ability,
        d20::{D20CheckDC, D20CheckResult},
        saving_throw::{SavingThrowKind, SavingThrowSet},
        skill::{Skill, SkillSet},
    },
    systems,
};

pub fn saving_throw(world: &World, entity: Entity, kind: SavingThrowKind) -> D20CheckResult {
    systems::helpers::get_component::<SavingThrowSet>(world, entity).check(kind, world, entity)
}

pub fn saving_throw_dc(
    world: &World,
    entity: Entity,
    dc: &D20CheckDC<SavingThrowKind>,
) -> D20CheckResult {
    systems::helpers::get_component::<SavingThrowSet>(world, entity).check_dc(dc, world, entity)
}

pub fn skill_check(world: &World, entity: Entity, skill: Skill) -> D20CheckResult {
    systems::helpers::get_component::<SkillSet>(world, entity).check(skill, world, entity)
}

pub fn skill_check_dc(world: &World, entity: Entity, dc: &D20CheckDC<Skill>) -> D20CheckResult {
    systems::helpers::get_component::<SkillSet>(world, entity).check_dc(dc, world, entity)
}

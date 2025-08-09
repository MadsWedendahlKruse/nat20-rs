use hecs::{Entity, World};

use crate::{
    components::{
        ability::Ability,
        d20_check::{D20CheckDC, D20CheckResult},
        saving_throw::SavingThrowSet,
        skill::{Skill, SkillSet},
    },
    systems,
};

pub fn saving_throw(world: &World, entity: Entity, ability: Ability) -> D20CheckResult {
    systems::helpers::get_component::<SavingThrowSet>(world, entity).check(ability, world, entity)
}

pub fn saving_throw_dc(world: &World, entity: Entity, dc: &D20CheckDC<Ability>) -> D20CheckResult {
    systems::helpers::get_component::<SavingThrowSet>(world, entity).check_dc(dc, world, entity)
}

pub fn skill_check(world: &World, entity: Entity, skill: Skill) -> D20CheckResult {
    systems::helpers::get_component::<SkillSet>(world, entity).check(skill, world, entity)
}

pub fn skill_check_dc(world: &World, entity: Entity, dc: &D20CheckDC<Skill>) -> D20CheckResult {
    systems::helpers::get_component::<SkillSet>(world, entity).check_dc(dc, world, entity)
}

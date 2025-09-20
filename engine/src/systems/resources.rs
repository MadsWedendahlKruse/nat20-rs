use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::ActionCooldownMap,
        resource::{RechargeRule, ResourceAmountMap, ResourceError, ResourceMap},
    },
    systems,
};

// TODO: No idea where to put this
pub fn recharge(world: &mut World, entity: Entity, rest_type: &RechargeRule) {
    for (_, resource) in
        systems::helpers::get_component_mut::<ResourceMap>(world, entity).iter_mut()
    {
        resource.recharge_full(rest_type);
    }

    systems::helpers::get_component_mut::<ActionCooldownMap>(world, entity)
        .retain(|_, recharge_rule| !recharge_rule.is_recharged_by(rest_type));
}

pub fn can_afford(world: &World, entity: Entity, cost: &ResourceAmountMap) -> bool {
    systems::helpers::get_component::<ResourceMap>(world, entity).can_afford_all(cost)
}

pub fn spend(
    world: &mut World,
    entity: Entity,
    cost: &ResourceAmountMap,
) -> Result<(), ResourceError> {
    systems::helpers::get_component_mut::<ResourceMap>(world, entity).spend_all(cost)
}

pub fn restore(
    world: &mut World,
    entity: Entity,
    restoration: &ResourceAmountMap,
) -> Result<(), ResourceError> {
    systems::helpers::get_component_mut::<ResourceMap>(world, entity).restore_all(restoration)
}

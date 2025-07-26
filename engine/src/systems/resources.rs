use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::ActionCooldownMap,
        resource::{RechargeRule, ResourceMap},
    },
    systems,
};

// TODO: No idea where to put this
pub fn recharge(world: &mut World, entity: Entity, rest_type: &RechargeRule) {
    for (_, resource) in
        systems::helpers::get_component_mut::<ResourceMap>(world, entity).iter_mut()
    {
        resource.recharge(rest_type);
    }

    systems::helpers::get_component_mut::<ActionCooldownMap>(world, entity)
        .retain(|_, recharge_rule| !recharge_rule.is_recharged_by(rest_type));
}

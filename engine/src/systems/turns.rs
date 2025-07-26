use hecs::{Entity, World};

use crate::{components::resource::RechargeRule, systems};

pub fn on_turn_start(world: &mut World, entity: Entity) {
    systems::resources::recharge(world, entity, &RechargeRule::OnTurn);

    let expired_effects: Vec<_> = {
        let mut effects = systems::effects::effects_mut(world, entity);

        for effect in effects.iter_mut() {
            effect.increment_turns();
        }

        // Collect expired effects first to avoid double mutable borrow
        effects
            .iter()
            .filter(|effect| effect.is_expired())
            .cloned()
            .collect()
    };

    for effect in expired_effects {
        (effect.on_unapply)(world, entity);
    }

    systems::effects::effects_mut(world, entity).retain(|effect| !effect.is_expired());
}

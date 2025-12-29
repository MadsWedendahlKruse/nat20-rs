use hecs::{Entity, World};

use crate::{
    components::{health::hit_points::HitPoints, resource::RechargeRule},
    systems,
};

pub fn pass_time(world: &mut World, entity: Entity, passed_time: &RechargeRule) {
    systems::resources::recharge(world, entity, passed_time);

    // TODO: Technically this is always true?
    if RechargeRule::Turn.is_recharged_by(passed_time) {
        systems::movement::recharge_movement(world, entity);
    }

    let expired_effects: Vec<_> = {
        let mut effects = systems::effects::effects_mut(world, entity);

        for effect in effects.iter_mut() {
            effect.increment_turns_amount(passed_time.turns().unwrap_or(0));
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

    match passed_time {
        RechargeRule::ShortRest => {
            // SRD says we should spend Hit Dice here, but for now it's easier
            // to just heal half our max HP
            let half_max_hp = systems::helpers::get_component::<HitPoints>(world, entity).max() / 2;
            systems::health::heal(world, entity, half_max_hp);
        }

        RechargeRule::LongRest => {
            // TODO: Do we need to do anything else here?
            systems::health::heal_full(world, entity);
        }

        _ => {}
    }
}

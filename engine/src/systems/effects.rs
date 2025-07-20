use hecs::{Entity, Ref, World};

use crate::{
    components::{effects::effects::Effect, id::EffectId},
    registry, systems,
};

/// This gets used so often that it deserves its own function
pub fn effects(world: &World, entity: Entity) -> Ref<'_, Vec<Effect>> {
    systems::helpers::get_component::<Vec<Effect>>(world, entity)
}

pub fn effects_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, Vec<Effect>> {
    systems::helpers::get_component_mut::<Vec<Effect>>(world, entity)
}

fn get_effect(effect_id: &EffectId) -> Effect {
    registry::effects::EFFECT_REGISTRY
        .get(effect_id)
        .expect("Effect not found in registry")
        .clone()
}

pub fn add_effect(world: &mut World, entity: Entity, effect_id: &EffectId) {
    let effect = get_effect(effect_id);
    (effect.on_apply)(world, entity);
    effects_mut(world, entity).push(effect);
}

pub fn add_effects(world: &mut World, entity: Entity, effects: Vec<EffectId>) {
    for effect in effects {
        add_effect(world, entity, &effect);
    }
}

pub fn remove_effect(world: &mut World, entity: Entity, effect_id: &EffectId) {
    let effect = get_effect(effect_id);
    (effect.on_unapply)(world, entity);
    effects_mut(world, entity).retain(|e| e.id() != effect_id);
}

pub fn remove_effects(world: &mut World, entity: Entity, effects: &[EffectId]) {
    for effect in effects {
        remove_effect(world, entity, effect);
    }
}

use hecs::{Entity, Ref, World};

use crate::{
    components::{
        actions::action::ActionContext, effects::effects::Effect, id::EffectId,
        modifier::ModifierSource,
    },
    registry::registry::EffectsRegistry,
    systems,
};

/// This gets used so often that it deserves its own function
pub fn effects(world: &World, entity: Entity) -> Ref<'_, Vec<Effect>> {
    systems::helpers::get_component::<Vec<Effect>>(world, entity)
}

pub fn effects_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, Vec<Effect>> {
    systems::helpers::get_component_mut::<Vec<Effect>>(world, entity)
}

pub fn get_effect(effect_id: &EffectId) -> Effect {
    EffectsRegistry::get(effect_id)
        .expect(&format!(
            "Effect with ID `{}` not found in the registry",
            effect_id
        ))
        .clone()
}

pub fn add_effect(
    world: &mut World,
    entity: Entity,
    effect_id: &EffectId,
    source: &ModifierSource,
    context: Option<&ActionContext>,
) {
    let mut effect = get_effect(effect_id);
    effect.source = source.clone();
    (effect.on_apply)(world, entity, context);
    if let Some(replaces) = &effect.replaces {
        remove_effect(world, entity, replaces);
    }
    effects_mut(world, entity).push(effect);
}

pub fn add_effects(
    world: &mut World,
    entity: Entity,
    effects: &Vec<EffectId>,
    source: &ModifierSource,
    context: Option<&ActionContext>,
) {
    for effect in effects {
        add_effect(world, entity, effect, source, context);
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

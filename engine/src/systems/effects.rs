use core::panic;
use std::sync::Arc;

use hecs::{Entity, Ref, World};
use tracing::debug;

use crate::{
    components::{
        actions::action::ActionContext,
        effects::effect::{EffectInstance, EffectInstanceTemplate, EffectLifetime},
        id::EffectId,
        modifier::ModifierSource,
    },
    engine::{game_state::GameState, time::TurnKey},
    registry::registry::EffectsRegistry,
    systems,
};

/// This gets used so often that it deserves its own function
pub fn effects(world: &World, entity: Entity) -> Ref<'_, Vec<EffectInstance>> {
    systems::helpers::get_component::<Vec<EffectInstance>>(world, entity)
}

pub fn effects_mut(world: &mut World, entity: Entity) -> hecs::RefMut<'_, Vec<EffectInstance>> {
    systems::helpers::get_component_mut::<Vec<EffectInstance>>(world, entity)
}

fn add_effect_instance(
    game_state: &mut GameState,
    entity: Entity,
    effect_instance: EffectInstance,
    context: Option<&ActionContext>,
) {
    apply_and_replace(&mut game_state.world, entity, &effect_instance, context);

    match effect_instance.lifetime {
        EffectLifetime::AtTurnBoundary {
            entity: entity_anchor,
            boundary,
            remaining,
        } => {
            let turn_key = TurnKey {
                encounter_id: *game_state
                    .encounter_for_entity(&entity_anchor)
                    .expect("Entity with turn-boundary effect must be in an encounter"),
                entity: entity_anchor,
                boundary,
            };

            game_state.turn_scheduler.register(
                turn_key,
                remaining,
                Arc::new({
                    let effect_id = effect_instance.effect_id.clone();
                    move |world: &mut World| {
                        remove_effect(world, entity, &effect_id);
                    }
                }),
            );
        }
        EffectLifetime::Permanent => {
            panic!("Permanent effects be added with add_permanent_effect()");
        }
        _ => {}
    }

    effects_mut(&mut game_state.world, entity).push(effect_instance);
}

pub fn add_effect_template(
    game_state: &mut GameState,
    applier: Entity,
    target: Entity,
    source: ModifierSource,
    template: &EffectInstanceTemplate,
    context: Option<&ActionContext>,
) {
    let effect_instance = template.instantiate(applier, target, source);
    debug!(
        "Entity {:?} is adding effect instance {:?} to entity {:?}",
        applier, effect_instance, target
    );
    add_effect_instance(game_state, target, effect_instance, context);
}

pub fn add_permanent_effect(
    world: &mut World,
    entity: Entity,
    effect_id: EffectId,
    source: &ModifierSource,
    context: Option<&ActionContext>,
) {
    let effect_instance = EffectInstance::permanent(effect_id.clone(), source.clone());
    apply_and_replace(world, entity, &effect_instance, context);
    effects_mut(world, entity).push(effect_instance);
}

pub fn add_permanent_effects(
    world: &mut World,
    entity: Entity,
    effects: Vec<EffectId>,
    source: &ModifierSource,
    context: Option<&ActionContext>,
) {
    for effect_id in effects {
        add_permanent_effect(world, entity, effect_id, source, context);
    }
}

fn apply_and_replace(
    world: &mut World,
    entity: Entity,
    effect_instance: &EffectInstance,
    context: Option<&ActionContext>,
) {
    let effect = effect_instance.effect();
    (effect.on_apply)(world, entity, context);
    if let Some(replaces) = &effect.replaces {
        remove_effect(world, entity, replaces);
    }
}

pub fn remove_effect(world: &mut World, entity: Entity, effect_id: &EffectId) {
    debug!("Removing effect {:?} from entity {:?}", effect_id, entity);
    // TODO: Is this all we need to do here?
    let effect = EffectsRegistry::get(effect_id)
        .expect(format!("Effect definition not found for ID `{}`", effect_id).as_str());
    (effect.on_unapply)(world, entity);
    effects_mut(world, entity).retain(|e| e.effect_id != *effect_id);
}

pub fn remove_effects(world: &mut World, entity: Entity, effects: &[EffectId]) {
    for effect in effects {
        remove_effect(world, entity, effect);
    }
}

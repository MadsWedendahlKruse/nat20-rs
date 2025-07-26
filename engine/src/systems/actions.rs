use std::collections::HashMap;

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{
                Action, ActionContext, ActionCooldownMap, ActionKindSnapshot, ActionMap,
                ActionProvider,
            },
            targeting::TargetingContext,
        },
        id::ActionId,
        items::equipment::loadout::Loadout,
        resource::{RechargeRule, ResourceCostMap, ResourceMap},
        spells::spellbook::Spellbook,
    },
    engine::world,
    registry, systems,
};

pub fn get_action(action_id: &ActionId) -> Option<Action> {
    // Start by checking if the action exists in the action registry
    if let Some((action, _)) = registry::actions::ACTION_REGISTRY.get(action_id) {
        return Some(action.clone());
    }
    // If not found, check the spell registry
    let spell_id = action_id.to_spell_id();
    if let Some(spell) = registry::spells::SPELL_REGISTRY.get(&spell_id) {
        return Some(spell.action().clone());
    }
    None
}

pub fn on_cooldown(world: &World, entity: Entity, action_id: &ActionId) -> Option<RechargeRule> {
    if let Some(cooldowns) = world.get::<&ActionCooldownMap>(entity).ok() {
        cooldowns.get(action_id).cloned()
    } else {
        None
    }
}

pub fn all_actions(
    world: &World,
    entity: Entity,
) -> HashMap<ActionId, (Vec<ActionContext>, ResourceCostMap)> {
    let mut actions = systems::helpers::get_component_clone::<ActionMap>(world, entity);

    actions.extend(systems::helpers::get_component::<Spellbook>(world, entity).all_actions());

    actions.extend(systems::helpers::get_component::<Loadout>(world, entity).all_actions());

    actions
}

pub fn available_actions(
    world: &World,
    entity: Entity,
) -> HashMap<ActionId, (Vec<ActionContext>, ResourceCostMap)> {
    let mut actions = systems::helpers::get_component_clone::<ActionMap>(world, entity);

    actions.extend(systems::helpers::get_component::<Spellbook>(world, entity).available_actions());

    actions.extend(systems::helpers::get_component::<Loadout>(world, entity).available_actions());

    // Remove actions that are on cooldown or where the character does not
    // have the required resources
    actions.retain(|action_id, (action_contexts, resource_cost)| {
        if on_cooldown(world, entity, action_id).is_some() {
            // Action is on cooldown
            return false;
        }

        for action_context in action_contexts {
            for effect in systems::effects::effects(world, entity).iter() {
                (effect.on_resource_cost)(world, entity, action_context, resource_cost);
            }
        }

        let resources = systems::helpers::get_component::<ResourceMap>(world, entity);
        for (resource_id, amount) in resource_cost {
            if let Some(resource) = resources.get(resource_id) {
                if resource.current_uses() < *amount {
                    // Not enough resources for this action
                    return false;
                }
            } else {
                // Resource not found
                return false;
            }
        }

        true
    });

    actions
}

pub fn perform_action(
    world: &mut World,
    entity: Entity,
    action_id: &ActionId,
    context: &ActionContext,
    num_snapshots: usize,
) -> Vec<ActionKindSnapshot> {
    // TODO: Handle missing action
    let mut action =
        get_action(action_id).expect("Action not found in character's actions or registry");
    if let Some(cooldown) = action.cooldown {
        systems::helpers::get_component_mut::<ActionCooldownMap>(world, entity)
            .insert(action_id.clone(), cooldown);
    }
    action.perform(world, entity, &context, num_snapshots)
}

pub fn targeting_context(
    world: &World,
    entity: Entity,
    action_id: &ActionId,
    context: &ActionContext,
) -> TargetingContext {
    // TODO: Handle missing action
    get_action(action_id).unwrap().targeting()(world, entity, context)
}

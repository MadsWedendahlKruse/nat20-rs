use std::collections::HashMap;

use hecs::{Entity, World};
use rand::rand_core::le;

use crate::{
    components::{
        actions::{
            action::{
                Action, ActionContext, ActionCooldownMap, ActionKindSnapshot, ActionMap,
                ActionProvider, ActionResult, ReactionKind, ReactionSet,
            },
            targeting::TargetingContext,
        },
        id::{ActionId, ResourceId},
        items::equipment::loadout::Loadout,
        resource::{RechargeRule, ResourceCostMap, ResourceMap},
        spells::spellbook::Spellbook,
    },
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

pub fn add_actions(world: &mut World, entity: Entity, actions: &[ActionId]) {
    let mut action_map = systems::helpers::get_component_mut::<ActionMap>(world, entity);
    for action_id in actions {
        if let Some((action, context)) = registry::actions::ACTION_REGISTRY.get(action_id) {
            add_action_to_map(&mut action_map, action_id, action, context.clone());
        } else {
            panic!("Action {} not found in registry", action_id);
        }
    }
}

fn add_action_to_map(
    action_map: &mut ActionMap,
    action_id: &ActionId,
    action: &Action,
    context: Option<ActionContext>,
) {
    let resource_cost = &action.resource_cost().clone();
    action_map
        .entry(action_id.clone())
        .and_modify(|(action_context, action_resource_cost)| {
            action_context.push(context.clone().unwrap());
            action_resource_cost.extend(resource_cost.clone());
        })
        .or_insert((vec![context.clone().unwrap()], resource_cost.clone()));
}

pub fn on_cooldown(world: &World, entity: Entity, action_id: &ActionId) -> Option<RechargeRule> {
    if let Some(cooldowns) = world.get::<&ActionCooldownMap>(entity).ok() {
        cooldowns.get(action_id).cloned()
    } else {
        None
    }
}

pub fn all_actions(world: &World, entity: Entity) -> ActionMap {
    let mut actions = systems::helpers::get_component_clone::<ActionMap>(world, entity);

    actions.extend(systems::helpers::get_component::<Spellbook>(world, entity).all_actions());

    actions.extend(systems::helpers::get_component::<Loadout>(world, entity).all_actions());

    actions
}

pub enum ActionUsability {
    Usable,
    OnCooldown(RechargeRule),
    NotEnoughResources(ResourceCostMap),
    ResourceNotFound(ResourceId),
}

pub fn action_usable(
    world: &World,
    entity: Entity,
    action_id: &ActionId,
    contexts: &Vec<ActionContext>,
    resource_cost: &mut ResourceCostMap,
) -> ActionUsability {
    if let Some(cooldown) = on_cooldown(world, entity, action_id) {
        return ActionUsability::OnCooldown(cooldown);
    }

    for action_context in contexts {
        for effect in systems::effects::effects(world, entity).iter() {
            (effect.on_resource_cost)(world, entity, action_context, resource_cost);
        }
    }

    let resources = systems::helpers::get_component::<ResourceMap>(world, entity);
    for (resource_id, amount) in &*resource_cost {
        if let Some(resource) = resources.get(resource_id) {
            if resource.current_uses() < *amount {
                return ActionUsability::NotEnoughResources(resource_cost.clone());
            }
        } else {
            return ActionUsability::ResourceNotFound(resource_id.clone());
        }
    }

    ActionUsability::Usable
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

fn filter_reactions(actions: &ActionMap) -> ReactionSet {
    actions
        .iter()
        .filter_map(|(action_id, _)| {
            if let Some(action) = get_action(action_id) {
                if action.reaction_trigger.is_some() {
                    return Some(action_id.clone());
                }
            }
            None
        })
        .collect()
}

pub fn all_reactions(world: &World, entity: Entity) -> ReactionSet {
    filter_reactions(&all_actions(world, entity))
}

pub fn available_reactions(world: &World, entity: Entity) -> ReactionSet {
    filter_reactions(&available_actions(world, entity))
}

// TODO: Struct for the return type?
pub fn available_reactions_to_action(
    world: &World,
    reactor: Entity,
    actor: Entity,
    action_id: &ActionId,
    context: &ActionContext,
    targets: &[Entity],
) -> Vec<(ActionId, Vec<ActionContext>, ResourceCostMap, ReactionKind)> {
    let mut reactions = Vec::new();
    for (reaction_id, (contexts, resource_cost)) in
        systems::actions::available_actions(world, reactor)
    {
        // TODO: Would be nice if we didn't have to check two different registries
        let reaction = if let Some((reaction, _)) =
            registry::actions::ACTION_REGISTRY.get(&reaction_id)
        {
            reaction
        } else if let Some(spell) = registry::spells::SPELL_REGISTRY.get(&reaction_id.to_spell_id())
        {
            spell.action()
        } else {
            continue; // Skip if no reaction found
        };

        if let Some(trigger) = &reaction.reaction_trigger {
            if let Some(kind) = trigger(world, reactor, actor, action_id, context, targets) {
                reactions.push((
                    reaction_id.clone(),
                    contexts.clone(),
                    resource_cost.clone(),
                    kind,
                ));
            }
        }
    }
    reactions
}

pub fn apply_to_targets(
    world: &mut World,
    snapshots: Vec<ActionKindSnapshot>,
    targets: &Vec<Entity>,
) -> Vec<ActionResult> {
    targets
        .iter()
        .zip(snapshots.iter())
        .map(|(target, snapshot)| snapshot.apply_to_entity(world, *target))
        .collect()
}

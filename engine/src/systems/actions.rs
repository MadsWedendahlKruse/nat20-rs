use std::collections::HashMap;

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{
                Action, ActionContext, ActionCooldownMap, ActionMap, ActionProvider, ActionResult,
                ReactionResult, ReactionSet,
            },
            targeting::TargetingContext,
        },
        id::{ActionId, ResourceId},
        items::equipment::loadout::Loadout,
        resource::{RechargeRule, ResourceAmountMap, ResourceMap},
        spells::spellbook::Spellbook,
    },
    engine::{
        event::{self, ActionData, Event, EventId, EventKind, ReactionData},
        game_state::GameState,
    },
    registry, systems,
};

pub fn get_action(action_id: &ActionId) -> Option<&Action> {
    // Start by checking if the action exists in the action registry
    if let Some((action, _)) = registry::actions::ACTION_REGISTRY.get(action_id) {
        return Some(action);
    }
    // If not found, check the spell registry
    let spell_id = action_id.to_spell_id();
    if let Some(spell) = registry::spells::SPELL_REGISTRY.get(&spell_id) {
        return Some(spell.action());
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
        .and_modify(|action_data| {
            action_data.push((context.clone().unwrap(), resource_cost.clone()));
        })
        .or_insert(vec![(context.clone().unwrap(), resource_cost.clone())]);
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

    actions.extend(systems::helpers::get_component::<Spellbook>(world, entity).actions());

    actions.extend(systems::helpers::get_component::<Loadout>(world, entity).actions());

    actions
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActionUsability {
    Usable,
    OnCooldown(RechargeRule),
    NotEnoughResources(ResourceAmountMap),
    ResourceNotFound(ResourceId),
}

pub fn action_usable(
    world: &World,
    entity: Entity,
    action_id: &ActionId,
    // TODO: Is context really not needed here?
    action_context: &ActionContext,
    resource_cost: &ResourceAmountMap,
) -> ActionUsability {
    if let Some(cooldown) = on_cooldown(world, entity, action_id) {
        return ActionUsability::OnCooldown(cooldown);
    }

    let resources = systems::helpers::get_component::<ResourceMap>(world, entity);
    for (resource_id, amount) in &*resource_cost {
        if let Some(resource) = resources.get(resource_id) {
            if !resource.can_afford(amount) {
                return ActionUsability::NotEnoughResources(resource_cost.clone());
            }
        } else {
            return ActionUsability::ResourceNotFound(resource_id.clone());
        }
    }

    ActionUsability::Usable
}

pub fn available_actions(world: &World, entity: Entity) -> ActionMap {
    let mut actions = systems::helpers::get_component_clone::<ActionMap>(world, entity);

    actions.extend(systems::helpers::get_component::<Spellbook>(world, entity).actions());

    actions.extend(systems::helpers::get_component::<Loadout>(world, entity).actions());

    actions.retain(|action_id, action_data| {
        action_data.retain_mut(|(action_context, resource_cost)| {
            for effect in systems::effects::effects(world, entity).iter() {
                (effect.on_resource_cost)(world, entity, action_context, resource_cost);
            }

            if !matches!(
                action_usable(world, entity, action_id, &action_context, resource_cost),
                ActionUsability::Usable,
            ) {
                return false;
            }
            true
        });

        !action_data.is_empty() // Keep the action if there's at least one usable context
    });

    actions
}

pub fn perform_action(game_state: &mut GameState, action_data: &ActionData) {
    let ActionData {
        actor: performer,
        action_id,
        context,
        resource_cost,
        targets,
    } = action_data;
    // TODO: Handle missing action
    let mut action = get_action(action_id)
        .cloned()
        .expect("Action not found in character's actions or registry");
    // Set the action on cooldown if applicable
    if let Some(cooldown) = action.cooldown {
        systems::helpers::get_component_mut::<ActionCooldownMap>(&mut game_state.world, *performer)
            .insert(action_id.clone(), cooldown);
    }
    action.perform(game_state, *performer, &context, resource_cost, &targets);
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
pub fn available_reactions_to_event(
    world: &World,
    reactor: Entity,
    event: &Event,
) -> Vec<ReactionData> {
    let mut reactions = Vec::new();

    for (reaction_id, contexts_and_costs) in systems::actions::available_actions(world, reactor) {
        let reaction = systems::actions::get_action(&reaction_id);
        if reaction.is_none() {
            continue;
        }
        let reaction = reaction.unwrap();

        if let Some(trigger) = &reaction.reaction_trigger {
            if trigger(reactor, event) {
                for (context, resource_cost) in &contexts_and_costs {
                    // TODO: Lots of duplicated information here
                    reactions.push(ReactionData {
                        reactor,
                        event: event.clone().into(),
                        reaction_id: reaction_id.clone(),
                        context: ActionContext::Reaction {
                            trigger_event: Box::new(event.clone()),
                            resource_cost: resource_cost.clone(),
                            context: Box::new(context.clone()),
                        },
                        resource_cost: resource_cost.clone(),
                    });
                }
            }
        }
    }

    reactions
}

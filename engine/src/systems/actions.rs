use hecs::{Entity, World};

use crate::{
    components::{
        actions::{
            action::{
                Action, ActionContext, ActionCooldownMap, ActionKind, ActionMap, ActionProvider,
                ReactionSet,
            },
            targeting::{
                AreaShape, TargetInstance, TargetingContext, TargetingError, TargetingKind,
            },
        },
        id::{ActionId, ResourceId},
        items::equipment::loadout::Loadout,
        resource::{RechargeRule, ResourceAmountMap, ResourceMap},
        spells::spellbook::Spellbook,
    },
    engine::{
        event::{ActionData, ActionError, Event, ReactionData},
        game_state::GameState,
        geometry::WorldGeometry,
    },
    registry,
    systems::{self, geometry::RaycastFilter, movement::PathResult},
};

pub fn get_action(action_id: &ActionId) -> Option<&Action> {
    // Start by checking if the action exists in the action registry
    if let Some((action, _)) = registry::actions::ACTION_REGISTRY.get(action_id) {
        return Some(action);
    }
    // If not found, check the spell registry
    let spell_id = action_id.into();
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

pub fn set_cooldown(
    world: &mut World,
    entity: Entity,
    action_id: &ActionId,
    cooldown: RechargeRule,
) {
    let mut cooldowns = systems::helpers::get_component_mut::<ActionCooldownMap>(world, entity);
    cooldowns.insert(action_id.clone(), cooldown);
}

pub fn all_actions(world: &World, entity: Entity) -> ActionMap {
    let mut actions = systems::helpers::get_component_clone::<ActionMap>(world, entity);

    actions.extend(systems::helpers::get_component::<Spellbook>(world, entity).actions());

    actions.extend(systems::helpers::get_component::<Loadout>(world, entity).actions());

    actions
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionUsabilityError {
    OnCooldown(RechargeRule),
    NotEnoughResources(ResourceAmountMap),
    ResourceNotFound(ResourceId),
    TargetingError(TargetingError),
}

pub fn action_usable(
    world: &World,
    entity: Entity,
    action_id: &ActionId,
    // TODO: Is context really not needed here?
    action_context: &ActionContext,
    resource_cost: &ResourceAmountMap,
) -> Result<(), ActionUsabilityError> {
    if let Some(cooldown) = on_cooldown(world, entity, action_id) {
        return Err(ActionUsabilityError::OnCooldown(cooldown));
    }

    let resources = systems::helpers::get_component::<ResourceMap>(world, entity);
    for (resource_id, amount) in resource_cost {
        if let Some(resource) = resources.get(resource_id) {
            if !resource.can_afford(amount) {
                return Err(ActionUsabilityError::NotEnoughResources(
                    resource_cost.clone(),
                ));
            }
        } else {
            return Err(ActionUsabilityError::ResourceNotFound(resource_id.clone()));
        }
    }

    Ok(())
}

pub fn action_usable_on_targets(
    world: &World,
    world_geometry: &WorldGeometry,
    actor: Entity,
    action_id: &ActionId,
    context: &ActionContext,
    resource_cost: &ResourceAmountMap,
    targets: &[TargetInstance],
) -> Result<(), ActionUsabilityError> {
    action_usable(world, actor, action_id, context, resource_cost)?;

    let targeting_context = targeting_context(world, actor, action_id, context);

    if let Err(targeting_error) =
        targeting_context.validate_targets(world, world_geometry, actor, targets)
    {
        return Err(ActionUsabilityError::TargetingError(targeting_error));
    }

    Ok(())
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

            if action_usable(world, entity, action_id, &action_context, resource_cost).is_err() {
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
        set_cooldown(&mut game_state.world, *performer, action_id, cooldown);
    }
    // Determine which entities are being targeted
    let entities = get_targeted_entities(game_state, performer, action_id, context, targets);
    action.perform(game_state, *performer, &context, resource_cost, &entities);
}

fn get_targeted_entities(
    game_state: &mut GameState,
    performer: &Entity,
    action_id: &ActionId,
    context: &ActionContext,
    targets: &Vec<TargetInstance>,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let targeting_context = targeting_context(&game_state.world, *performer, action_id, context);
    match targeting_context.kind {
        TargetingKind::SelfTarget | TargetingKind::Single | TargetingKind::Multiple { .. } => {
            for target in targets {
                match target {
                    TargetInstance::Entity(entity) => entities.push(*entity),
                    TargetInstance::Point(point) => {
                        if let Some(entity) =
                            systems::geometry::get_entity_at_point(&game_state.world, *point)
                        {
                            entities.push(entity);
                        }
                    }
                }
            }
        }

        TargetingKind::Area {
            shape,
            fixed_on_actor,
        } => {
            for target in targets {
                let point = match target {
                    TargetInstance::Entity(entity) => {
                        &systems::geometry::get_foot_position(&game_state.world, *entity).unwrap()
                    }

                    TargetInstance::Point(point) => point,
                };

                let (shape_hitbox, shape_pose) =
                    shape.parry3d_shape(&game_state.world, *performer, fixed_on_actor, point);

                let mut entities_in_shape = systems::geometry::entities_in_shape(
                    &game_state.world,
                    shape_hitbox,
                    &shape_pose,
                );

                // Check if any of the entities are behind cover and remove them
                // TODO: Not sure what the best way to do this is, I guess it
                // depends on the shape?

                match shape {
                    AreaShape::Sphere { .. } => {
                        entities_in_shape.retain(|entity| {
                            systems::geometry::line_of_sight_entity_point_filter(
                                &game_state.world,
                                &game_state.geometry,
                                *entity,
                                *point,
                                // TODO: Can't hide behind other entities?
                                &RaycastFilter::WorldOnly,
                            )
                            .has_line_of_sight
                        });
                    }

                    _ => {}
                }

                entities.extend(entities_in_shape);
            }
        }
    }
    entities
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
    world_geometry: &WorldGeometry,
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
                    if action_usable_on_targets(
                        world,
                        world_geometry,
                        reactor,
                        &reaction_id,
                        context,
                        resource_cost,
                        &[TargetInstance::Entity(event.actor().unwrap())],
                    )
                    .is_ok()
                    {
                        // TODO: Lots of duplicated information here
                        reactions.push(ReactionData {
                            reactor,
                            event: event.clone().into(),
                            reaction_id: reaction_id.clone(),
                            context: context.clone(),
                            resource_cost: resource_cost.clone(),
                        });
                    }
                }
            }
        }
    }

    reactions
}

pub fn perform_reaction(game_state: &mut GameState, reaction_data: &ReactionData) {
    let action = get_action(&reaction_data.reaction_id)
        .unwrap_or_else(|| panic!("Reaction action not found: {:?}", reaction_data.reaction_id));

    match &action.kind {
        ActionKind::Reaction { reaction } => {
            reaction(game_state, reaction_data);
        }
        _ => panic!(
            "ReactionData refers to non-Reaction ActionKind: {:?}",
            reaction_data.reaction_id
        ),
    }
}

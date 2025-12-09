use std::sync::Arc;

use hecs::{Entity, World};

use crate::{
    components::{
        ability::Ability,
        actions::{
            action::{
                Action, ActionContext, ActionCooldownMap, ActionKind, ActionKindResult, ActionMap,
                ActionProvider, ReactionResult, ReactionSet,
            },
            targeting::{
                AreaShape, TargetInstance, TargetingContext, TargetingError, TargetingKind,
            },
        },
        id::{ActionId, ResourceId, ScriptId, SpellId},
        items::equipment::loadout::Loadout,
        resource::{RechargeRule, ResourceAmountMap, ResourceMap},
        spells::spellbook::Spellbook,
    },
    engine::{
        event::{ActionData, CallbackResult, Event, EventCallback, EventKind, ReactionData},
        game_state::GameState,
        geometry::WorldGeometry,
    },
    registry::{
        self,
        registry::{ActionsRegistry, ScriptsRegistry, SpellsRegistry},
    },
    scripts::{
        script,
        script_api::{
            ReactionBodyContext, ReactionTriggerContext, ScriptEntityRole, ScriptEventRef,
            ScriptReactionPlan, ScriptSavingThrowSpec,
        },
        script_engine::ScriptEngineMap,
    },
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
        geometry::RaycastFilter,
    },
};

pub fn get_action(action_id: &ActionId) -> Option<&Action> {
    // Start by checking if the action exists in the action registry
    if let Some(action) = ActionsRegistry::get(action_id) {
        return Some(action);
    }

    // TODO: Placeholder untill migrated fully to ActionsRegistry
    if let Some((action, _)) = registry::actions::ACTION_REGISTRY.get(action_id) {
        return Some(action);
    }

    // If not found, check the spell registry
    let spell_id = action_id.into();
    if let Some(spell) = SpellsRegistry::get(&spell_id) {
        return Some(spell.action());
    }

    None
}

pub fn add_actions(world: &mut World, entity: Entity, actions: &[ActionId]) {
    let mut action_map = systems::helpers::get_component_mut::<ActionMap>(world, entity);
    for action_id in actions {
        if let Some(action) = systems::actions::get_action(action_id) {
            // TODO: Just assume the context is Other for now
            add_action_to_map(&mut action_map, action_id, action, ActionContext::Other);
        } else {
            panic!("Action {} not found in registry", action_id);
        }
    }
}

fn add_action_to_map(
    action_map: &mut ActionMap,
    action_id: &ActionId,
    action: &Action,
    context: ActionContext,
) {
    let resource_cost = &action.resource_cost().clone();
    action_map
        .entry(action_id.clone())
        .and_modify(|action_data| {
            action_data.push((context.clone(), resource_cost.clone()));
        })
        .or_insert(vec![(context, resource_cost.clone())]);
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

fn run_reaction_trigger(
    reaction_trigger: &ScriptId,
    reactor: Entity,
    event: &Event,
    script_engines: &mut ScriptEngineMap,
) -> bool {
    let script = ScriptsRegistry::get(reaction_trigger).expect(
        format!(
            "Reaction trigger script not found in registry: {:?}",
            reaction_trigger
        )
        .as_str(),
    );
    let engine = script_engines
        .get_mut(&script.language)
        .expect(format!("No script engine found for language: {:?}", script.language).as_str());
    let context = ReactionTriggerContext {
        reactor,
        event: event.clone(),
    };
    match engine.evaluate_reaction_trigger(script, &context) {
        Ok(result) => result,
        Err(err) => {
            println!(
                "Error evaluating reaction trigger script {:?} for reactor {:?}: {:?}",
                reaction_trigger, reactor, err
            );
            false
        }
    }
}

pub fn available_reactions_to_event(
    world: &World,
    world_geometry: &WorldGeometry,
    reactor: Entity,
    event: &Event,
    script_engines: &mut ScriptEngineMap,
) -> Vec<ReactionData> {
    let mut reactions = Vec::new();

    for (reaction_id, contexts_and_costs) in systems::actions::available_actions(world, reactor) {
        let reaction = systems::actions::get_action(&reaction_id);
        if reaction.is_none() {
            continue;
        }
        let reaction = reaction.unwrap();

        if let Some(trigger) = &reaction.reaction_trigger {
            println!(
                "[available_reactions_to_event] Evaluating reaction trigger {:?} for reactor {:?}",
                trigger, reactor
            );
            if run_reaction_trigger(trigger, reactor, event, script_engines) {
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
            // TODO: Helper method to get script and engine?
            let script = ScriptsRegistry::get(reaction)
                .expect(format!("Reaction script not found in registry: {:?}", reaction).as_str());
            let engine = game_state.script_engines.get_mut(&script.language).expect(
                format!("No script engine found for language: {:?}", script.language).as_str(),
            );
            let context = ReactionBodyContext {
                reaction_data: reaction_data.clone(),
            };
            match engine.evaluate_reaction_body(script, &context) {
                Ok(plan) => {
                    println!(
                        "[perform_reaction] Applying reaction plan from script {:?} for reactor {:?}",
                        reaction, reaction_data.reactor
                    );
                    apply_reaction_plan(game_state, &context, plan);
                }
                Err(err) => {
                    println!(
                        "Error evaluating reaction body script {:?} for reactor {:?}: {:?}",
                        reaction, reaction_data.reactor, err
                    );
                }
            }
        }
        _ => panic!(
            "ReactionData refers to non-Reaction ActionKind: {:?}",
            reaction_data.reaction_id
        ),
    }
}

fn apply_reaction_plan(
    game_state: &mut GameState,
    context: &ReactionBodyContext,
    plan: ScriptReactionPlan,
) {
    let reaction_data = &context.reaction_data;

    match plan {
        ScriptReactionPlan::None => {}

        ScriptReactionPlan::Sequence(plans) => {
            for p in plans {
                apply_reaction_plan(game_state, context, p);
            }
        }

        ScriptReactionPlan::ModifyD20Result { bonus } => {
            // Find the relevant D20 result for this event and apply bonus.
            // This is the same logic Tactical Mind would use.

            // modify_latest_d20_for_event(game_state, &reaction_data.event, bonus);
        }

        ScriptReactionPlan::CancelEvent {
            event,
            resources_to_refund,
        } => {
            let target_event = match event {
                ScriptEventRef::TriggerEvent => &reaction_data.event,
            };

            let mut resources_refunded = ResourceAmountMap::new();
            for resource_id in &resources_to_refund {
                // TODO: Which resources should we refund if it's *not* the trigger event?
                resources_refunded.insert(
                    resource_id.clone(),
                    context
                        .reaction_data
                        .resource_cost
                        .get(resource_id)
                        .cloned()
                        .unwrap(),
                );
            }

            let result = ReactionResult::CancelEvent {
                event: target_event.clone(),
                resources_refunded,
            };

            let process_event_result = game_state.process_event(Event::action_performed_event(
                game_state,
                reaction_data.reactor,
                &reaction_data.reaction_id,
                &reaction_data.context,
                &reaction_data.resource_cost,
                target_event.actor().unwrap(),
                ActionKindResult::Reaction { result },
            ));

            match process_event_result {
                Ok(_) => {
                    println!(
                        "[apply_reaction_plan] Cancelled event {:?} due to reaction by {:?}",
                        target_event.id, reaction_data.reactor
                    );
                }
                Err(err) => {
                    println!(
                        "Error processing CancelEvent reaction for reactor {:?}: {:?}",
                        reaction_data.reactor, err
                    );
                }
            }
        }

        ScriptReactionPlan::RequireSavingThrow {
            target,
            dc,
            on_success,
            on_failure,
        } => {
            // Resolve the target entity
            let target_entity = match target {
                ScriptEntityRole::Reactor => reaction_data.reactor,
                ScriptEntityRole::TriggerActor => context
                    .reaction_data
                    .event
                    .actor()
                    .expect("Trigger event has no actor"),
            };

            // Resolve the DC spec to a real D20CheckDCKind
            let dc_kind = D20CheckDCKind::SavingThrow((dc.saving_throw.function)(
                &game_state.world,
                target_entity,
                &reaction_data.context,
            ));

            // Emit the check event and attach callback to continue the plan.
            let check_event = systems::d20::check(game_state, target_entity, &dc_kind);

            let context_clone = context.clone();
            let on_success_plan = *on_success;
            let on_failure_plan = *on_failure;

            let callback: EventCallback = Arc::new(move |game_state, event| {
                if let EventKind::D20CheckResolved(_, result_kind, _) = &event.kind {
                    let success = match result_kind {
                        D20ResultKind::SavingThrow { result, .. } => result.success,
                        _ => panic!("RequireSavingThrow expects a saving throw result"),
                    };

                    let next_plan = if success {
                        on_success_plan.clone()
                    } else {
                        on_failure_plan.clone()
                    };

                    // Continue interpreting the reaction plan
                    apply_reaction_plan(game_state, &context_clone, next_plan);

                    CallbackResult::None
                } else {
                    panic!("RequireSavingThrow callback received unexpected event");
                }
            });

            let _ = game_state.process_event_with_callback(check_event, callback);
        }
    }
}

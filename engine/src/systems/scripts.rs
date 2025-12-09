use std::sync::Arc;

use hecs::World;

use crate::{
    components::{
        actions::action::{ActionKindResult, ReactionResult},
        id::ScriptId,
        modifier::ModifierSource,
        resource::ResourceAmountMap,
    },
    engine::{
        event::{CallbackResult, Event, EventCallback, EventKind},
        game_state::GameState,
    },
    registry::registry::ScriptsRegistry,
    scripts::{
        script_api::{
            ReactionBodyContext, ReactionTriggerContext, ScriptEntityRole, ScriptEventRef,
            ScriptReactionPlan,
        },
        script_engine::ScriptEngineMap,
    },
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub fn run_reaction_trigger(
    reaction_trigger: &ScriptId,
    context: &ReactionTriggerContext,
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
    match engine.evaluate_reaction_trigger(script, &context) {
        Ok(result) => result,
        Err(err) => {
            println!(
                "Error evaluating reaction trigger script {:?} for reactor {:?}: {:?}",
                reaction_trigger, context.reactor, err
            );
            false
        }
    }
}

pub fn run_reaction_body(
    reaction_body: &ScriptId,
    context: &ReactionBodyContext,
    script_engines: &mut ScriptEngineMap,
) -> ScriptReactionPlan {
    let script = ScriptsRegistry::get(reaction_body)
        .expect(format!("Reaction script not found in registry: {:?}", reaction_body).as_str());
    let engine = script_engines
        .get_mut(&script.language)
        .expect(format!("No script engine found for language: {:?}", script.language).as_str());
    match engine.evaluate_reaction_body(script, &context) {
        Ok(plan) => plan,
        Err(err) => {
            println!(
                "Error evaluating reaction body script {:?} for reactor {:?}: {:?}",
                reaction_body, context.reaction_data.reactor, err
            );
            ScriptReactionPlan::None
        }
    }
}

pub fn apply_reaction_plan(
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
            let bonus_value = bonus.evaluate(
                &game_state.world,
                reaction_data.reactor,
                &reaction_data.context,
            );

            let result = ReactionResult::ModifyEvent {
                modification: Arc::new({
                    let action_id = context.reaction_data.reaction_id.clone();
                    move |_world: &World, event: &mut Event| {
                        if let EventKind::D20CheckPerformed(_, ref mut existing_result, _) =
                            event.kind
                        {
                            match existing_result {
                                D20ResultKind::Skill { result, .. } => {
                                    result.add_bonus(
                                        ModifierSource::Action(action_id.clone()),
                                        bonus_value,
                                    );
                                }
                                _ => panic!(
                                    "ModifyD20Result applied to wrong result type: {:?}",
                                    existing_result
                                ),
                            }
                        } else {
                            panic!("ModifyD20Result applied to wrong event type: {:?}", event);
                        }
                    }
                }),
            };

            let process_event_result = game_state.process_event(Event::action_performed_event(
                game_state,
                reaction_data.reactor,
                &reaction_data.reaction_id,
                &reaction_data.context,
                &reaction_data.resource_cost,
                reaction_data.reactor,
                ActionKindResult::Reaction { result },
            ));

            match process_event_result {
                Ok(_) => {}
                Err(err) => {
                    println!(
                        "Error processing ModifyD20Result reaction for reactor {:?}: {:?}",
                        reaction_data.reactor, err
                    );
                }
            }
        }

        ScriptReactionPlan::RerollD20Result {
            bonus,
            force_use_new,
        } => {
            let bonus_value = if let Some(bonus_expr) = bonus {
                bonus_expr.evaluate(
                    &game_state.world,
                    reaction_data.reactor,
                    &reaction_data.context,
                )
            } else {
                0
            };

            let result = ReactionResult::ModifyEvent {
                modification: Arc::new({
                    let actor = reaction_data.event.actor().unwrap();
                    let action_id = context.reaction_data.reaction_id.clone();
                    move |world: &World, event: &mut Event| {
                        if let EventKind::D20CheckPerformed(
                            _,
                            ref mut existing_result,
                            ref dc_kind,
                        ) = event.kind
                        {
                            let mut new_roll = systems::d20::check_no_event(world, actor, dc_kind);
                            new_roll
                                .d20_result_mut()
                                .add_bonus(ModifierSource::Action(action_id.clone()), bonus_value);

                            if force_use_new {
                                *existing_result = new_roll;
                            } else {
                                // Choose the better of the two rolls
                                let existing_total = existing_result.d20_result().total();
                                let new_total = new_roll.d20_result().total();
                                if new_total > existing_total {
                                    *existing_result = new_roll;
                                }
                            }
                        } else {
                            panic!("RerollD20Result applied to wrong event type: {:?}", event);
                        }
                    }
                }),
            };

            let process_event_result = game_state.process_event(Event::action_performed_event(
                game_state,
                reaction_data.reactor,
                &reaction_data.reaction_id,
                &reaction_data.context,
                &reaction_data.resource_cost,
                reaction_data.reactor,
                ActionKindResult::Reaction { result },
            ));

            match process_event_result {
                Ok(_) => {}
                Err(err) => {
                    println!(
                        "Error processing RerollD20Result reaction for reactor {:?}: {:?}",
                        reaction_data.reactor, err
                    );
                }
            }
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
                Ok(_) => {}
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

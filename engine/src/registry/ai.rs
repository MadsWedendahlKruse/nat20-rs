use std::{collections::HashMap, sync::LazyLock};

use hecs::Entity;
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::{
    components::{
        actions::targeting::{TargetInstance, TargetingKind},
        ai::{AIController, AIDecision},
        id::AIControllerId,
    },
    engine::{
        event::{ActionData, ActionDecision, ActionDecisionKind, ActionPrompt, ActionPromptKind},
        game_state::GameState,
    },
    systems::{self, movement::TargetPathFindingResult},
};

pub static AI_CONTROLLER_REGISTRY: LazyLock<HashMap<AIControllerId, Box<dyn AIController>>> =
    LazyLock::new(|| {
        HashMap::from([(
            RANDOM_CONTROLLER_ID.clone(),
            Box::new(RandomController) as Box<dyn AIController>,
        )])
    });

pub static RANDOM_CONTROLLER_ID: LazyLock<AIControllerId> =
    LazyLock::new(|| AIControllerId::from_str("ai_controller.random"));

pub struct RandomController;

impl AIController for RandomController {
    fn decide(
        &self,
        game_state: &mut GameState,
        prompt: &ActionPrompt,
        actor: Entity,
    ) -> AIDecision {
        let rng = &mut rand::rng();

        // TODO: Validation that it's the actor's turn?

        match &prompt.kind {
            ActionPromptKind::Action { actor } => {
                let actions = systems::actions::available_actions(
                    &game_state.world,
                    *actor,
                    &mut game_state.script_engines,
                );

                // Pick a random action
                if actions.is_empty() {
                    // TODO: End turn?
                    return AIDecision::empty(*actor);
                }

                if let Some(action_id) = actions.keys().choose(rng)
                    && let Some(contexts_and_costs) = actions.get(action_id)
                    && let Some((context, resource_cost)) = contexts_and_costs.choose(rng)
                    && let Some(action) = systems::actions::get_action(action_id)
                {
                    let targeting = systems::actions::targeting_context(
                        &game_state.world,
                        *actor,
                        action_id,
                        &context,
                    );
                    let mut targets = Vec::new();

                    let possible_targets = if let Some(encounter_id) =
                        &game_state.in_combat.get(actor)
                        && let Some(encounter) = game_state.encounters.get(encounter_id)
                    {
                        encounter
                            .participants(&game_state.world, targeting.allowed_targets)
                            .into_iter()
                            .filter(|target| {
                                let target_attitude = systems::factions::mutual_attitude(
                                    &game_state.world,
                                    *actor,
                                    *target,
                                );
                                target_attitude
                                    == systems::ai::recommeneded_target_attitude(
                                        &game_state.world,
                                        *actor,
                                        &action.kind,
                                    )
                            })
                            .collect::<Vec<Entity>>()
                    } else {
                        return AIDecision::empty(*actor);
                    };

                    match targeting.kind {
                        TargetingKind::SelfTarget => targets.push(*actor),

                        TargetingKind::Single => {
                            if let Some(target) = possible_targets.iter().choose(rng) {
                                targets.push(*target);
                            }
                        }

                        TargetingKind::Multiple { max_targets } => {
                            let chosen_targets = possible_targets
                                .iter()
                                .choose_multiple(rng, max_targets.into());
                            targets.extend(chosen_targets);
                        }

                        TargetingKind::Area {
                            shape,
                            fixed_on_actor,
                        } => todo!(),
                    }

                    let action = ActionData {
                        actor: *actor,
                        action_id: action_id.clone(),
                        context: context.clone(),
                        resource_cost: resource_cost.clone(),
                        targets: targets
                            .iter()
                            .map(|entity| TargetInstance::Entity(*entity))
                            .collect(),
                    };

                    let path = match systems::movement::path_to_target(game_state, &action, true) {
                        Ok(result) => match result {
                            TargetPathFindingResult::AlreadyInRange => {
                                // Nothing to do
                                None
                            }
                            TargetPathFindingResult::PathFound(path_result) => Some(path_result),
                        },
                        Err(error) => {
                            // TODO: Not sure what to do here for AI
                            return AIDecision::empty(*actor);
                        }
                    };

                    return AIDecision {
                        actor: *actor,
                        decision: Some(ActionDecision {
                            response_to: prompt.id,
                            kind: ActionDecisionKind::Action {
                                action: ActionData {
                                    actor: *actor,
                                    action_id: action_id.clone(),
                                    context: context.clone(),
                                    resource_cost: resource_cost.clone(),
                                    targets: targets
                                        .iter()
                                        .map(|entity| TargetInstance::Entity(*entity))
                                        .collect(),
                                },
                            },
                        }),
                        path,
                    };
                } else {
                    AIDecision::empty(*actor)
                }
            }

            ActionPromptKind::Reactions { event, options } => {
                if let Some(options_for_actor) = options.get(&actor) {
                    if options_for_actor.is_empty() {
                        return AIDecision::empty(actor);
                    }

                    if let Some(choice) = options_for_actor.iter().choose(rng) {
                        return AIDecision {
                            actor,
                            decision: Some(ActionDecision {
                                response_to: prompt.id,
                                kind: ActionDecisionKind::Reaction {
                                    reactor: actor,
                                    event: event.clone(),
                                    choice: Some(choice.clone()),
                                },
                            }),
                            path: None,
                        };
                    } else {
                        return AIDecision::empty(actor);
                    }
                } else {
                    return AIDecision::empty(actor);
                }
            }
        }
    }
}

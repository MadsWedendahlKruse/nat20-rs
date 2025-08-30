use std::{collections::HashMap, sync::LazyLock};

use hecs::{Entity, World};
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::{
    components::{
        actions::targeting::{TargetType, TargetingKind},
        ai::AIController,
        id::AIControllerId,
    },
    engine::{
        encounter::{ActionDecision, ActionPrompt, Encounter, ParticipantsFilter},
        game_state::ActionData,
    },
    systems,
};

pub static AI_CONTROLLER_REGISTRY: LazyLock<HashMap<AIControllerId, Box<dyn AIController>>> =
    LazyLock::new(|| {
        HashMap::from([(
            RANDOM_CONTROLLER_ID.clone(),
            Box::new(RandomController) as Box<dyn AIController>,
        )])
    });

pub static RANDOM_CONTROLLER_ID: LazyLock<AIControllerId> =
    LazyLock::new(|| AIControllerId::from_str("brain.random"));

pub struct RandomController;

impl AIController for RandomController {
    fn decide(
        &self,
        world: &World,
        encounter: &Encounter,
        prompt: &ActionPrompt,
        actor: Entity,
    ) -> Option<ActionDecision> {
        let rng = &mut rand::rng();

        // TODO: Validation that it's the actor's turn?

        match prompt {
            ActionPrompt::Action { actor } => {
                let actions = systems::actions::available_actions(world, *actor);

                // Pick a random action
                if actions.is_empty() {
                    // TODO: End turn?
                    return None;
                }

                let action_id = actions.keys().choose(rng)?;
                let (contexts, _) = actions.get(action_id)?;
                let context = contexts.choose(rng)?;

                if let Some(action) = systems::actions::get_action(action_id) {
                    let targeting = (action.targeting())(world, *actor, context);

                    let mut targets = Vec::new();

                    for target_type in &targeting.valid_target_types {
                        match target_type {
                            TargetType::Entity { .. } => {
                                let possible_targets = encounter.participants(
                                    world,
                                    ParticipantsFilter::from(target_type.clone()),
                                );

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
                                    TargetingKind::Area { shape, origin } => todo!(),
                                }
                            }
                        }
                    }

                    Some(ActionDecision::Action {
                        action: ActionData {
                            actor: *actor,
                            action_id: action_id.clone(),
                            context: context.clone(),
                            targets,
                        },
                    })
                } else {
                    panic!("No action found for id {}", action_id);
                }
            }
            ActionPrompt::Reaction {
                reactor,
                action,
                options,
            } => todo!(),
        }
    }
}

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
        encounter::{Encounter, ParticipantsFilter},
        event::{ActionData, ActionDecision, ActionPrompt},
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

                let action_kind = systems::actions::get_action_clone(action_id)?.kind;
                let targeting =
                    systems::actions::targeting_context(world, *actor, action_id, context);
                let mut targets = Vec::new();

                for target_type in &targeting.valid_target_types {
                    match target_type {
                        TargetType::Entity { .. } => {
                            let possible_targets = encounter
                                .participants(world, ParticipantsFilter::from(target_type.clone()))
                                .into_iter()
                                .filter(|target| {
                                    let target_attitude =
                                        systems::factions::mutual_attitude(world, *actor, *target);
                                    target_attitude
                                        == systems::ai::recommeneded_target_attitude(
                                            world,
                                            *actor,
                                            &action_kind,
                                        )
                                })
                                .collect::<Vec<Entity>>();

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
            }
            ActionPrompt::Reaction {
                reactor,
                event,
                options,
            } => todo!("Implement reaction decision for RandomController AI"),
        }
    }
}

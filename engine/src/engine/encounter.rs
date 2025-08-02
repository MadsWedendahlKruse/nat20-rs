use std::collections::{HashSet, VecDeque};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::{ActionContext, ActionResult, ReactionKind},
        d20_check::D20CheckResult,
        id::{ActionId, EncounterId},
        resource::ResourceCostMap,
        skill::{Skill, SkillSet},
    },
    systems,
};

// TODO: struct name?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionData {
    pub actor: Entity,
    pub action_id: ActionId,
    pub context: ActionContext,
    pub targets: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionData {
    pub reaction_id: ActionId,
    pub contexts: Vec<ActionContext>,
    pub resource_cost: ResourceCostMap,
    pub kind: ReactionKind,
}

#[derive(Debug, Clone)]
pub enum ActionPrompt {
    Action {
        /// The entity that should perform the action
        actor: Entity,
    },
    Reaction {
        /// The entity that is reacting
        reactor: Entity,
        /// The action that triggered the reaction
        action: ActionData,
        /// The options available for the reaction. When responding to the prompt,
        /// the player must choose one of these options, or choose not to react.
        options: Vec<ReactionData>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionDecision {
    Action {
        action: ActionData,
    },
    Reaction {
        /// The entity that is reacting
        reactor: Entity,
        /// The action this is a reaction to
        action: ActionData,
        /// The choice made by the player in response to the reaction prompt.
        /// This will be `None` if the player chose not to react.
        choice: Option<ReactionData>,
    },
}

#[derive(Debug)]
pub enum ActionDecisionResult {
    /// The action was successfully performed, and the results are applied to the targets.
    ActionPerformed {
        action: ActionData,
        results: Vec<ActionResult>,
    },
    ReactionTriggered {
        reactor: Entity,
        action: ActionData,
    },
    ActionCancelled {
        reaction: ReactionData,
        action: ActionData,
    },
}

#[derive(Debug)]
pub enum ActionError {
    PromptDecisionMismatch {
        prompt: ActionPrompt,
        decision: ActionDecision,
    },
    FieldMismatch {
        field: &'static str,
        expected: String,
        actual: String,
        prompt: ActionPrompt,
        decision: ActionDecision,
    },
    MissingPrompt {
        decision: ActionDecision,
    },
}

macro_rules! ensure_equal {
    ($a:expr, $b:expr, $label:literal, $err_variant:ident, $self:expr, $decision:expr) => {
        if $a != $b {
            return Err(ActionError::$err_variant {
                field: $label,
                expected: format!("{:?}", $a),
                actual: format!("{:?}", $b),
                prompt: $self.clone(),
                decision: $decision.clone(),
            });
        }
    };
}

impl ActionPrompt {
    pub fn actor(&self) -> Entity {
        match self {
            ActionPrompt::Action { actor } => *actor,
            ActionPrompt::Reaction { action, .. } => action.actor,
        }
    }

    pub fn is_valid_decision(&self, decision: &ActionDecision) -> Result<(), ActionError> {
        match (self, decision) {
            (
                ActionPrompt::Action {
                    actor: prompt_actor,
                },
                ActionDecision::Action { action },
            ) => {
                ensure_equal!(
                    prompt_actor,
                    &action.actor,
                    "actor",
                    FieldMismatch,
                    self,
                    decision
                );
            }

            (
                ActionPrompt::Reaction {
                    reactor: prompt_reactor,
                    action: prompt_action,
                    options,
                },
                ActionDecision::Reaction {
                    reactor: decision_reactor,
                    action: decision_action,
                    choice,
                },
            ) => {
                ensure_equal!(
                    prompt_reactor,
                    decision_reactor,
                    "reactor",
                    FieldMismatch,
                    self,
                    decision
                );
                ensure_equal!(
                    prompt_action.actor,
                    decision_action.actor,
                    "actor",
                    FieldMismatch,
                    self,
                    decision
                );
                ensure_equal!(
                    prompt_action.action_id,
                    decision_action.action_id,
                    "action_id",
                    FieldMismatch,
                    self,
                    decision
                );
                ensure_equal!(
                    prompt_action.context,
                    decision_action.context,
                    "context",
                    FieldMismatch,
                    self,
                    decision
                );

                // Optional: check if reaction_decision is one of the prompt options
                if let Some(choice) = choice {
                    if !options.contains(choice) {
                        return Err(ActionError::FieldMismatch {
                            field: "reaction_decision",
                            expected: format!("one of {:?}", options),
                            actual: format!("{:?}", choice),
                            prompt: self.clone(),
                            decision: decision.clone(),
                        });
                    }
                }
            }

            _ => {
                return Err(ActionError::PromptDecisionMismatch {
                    prompt: self.clone(),
                    decision: decision.clone(),
                });
            }
        }
        Ok(())
    }
}

impl ActionDecision {
    pub fn actor(&self) -> Entity {
        match self {
            ActionDecision::Action { action, .. } => action.actor,
            ActionDecision::Reaction { action, .. } => action.actor,
        }
    }

    pub fn action_id(&self) -> &ActionId {
        match self {
            ActionDecision::Action { action, .. } => &action.action_id,
            ActionDecision::Reaction { action, .. } => &action.action_id,
        }
    }
}

pub struct Encounter {
    pub id: EncounterId,
    pub participants: HashSet<Entity>,
    pub round: usize,
    pub turn_index: usize,
    pub initiative_order: Vec<(Entity, D20CheckResult)>,
    pub pending_prompts: VecDeque<ActionPrompt>,
    /// In case a reaction is requested, this will hold the pending decisions
    /// until the reaction is resolved. The decision will then be processed once
    /// the reaction is resolved. In most cases this will be a single decision.
    pub pending_decisions: VecDeque<ActionDecision>,
}

impl Encounter {
    pub fn new(world: &mut World, participants: HashSet<Entity>, id: EncounterId) -> Self {
        let mut engine = Self {
            id,
            participants,
            round: 1,
            turn_index: 0,
            initiative_order: Vec::new(),
            pending_prompts: VecDeque::new(),
            pending_decisions: VecDeque::new(),
        };
        engine.roll_initiative(world);
        engine.start_turn(world);
        engine
    }

    fn roll_initiative(&mut self, world: &World) {
        let mut indexed_rolls: Vec<(Entity, D20CheckResult)> = self
            .participants
            .iter()
            .map(|entity| {
                let roll = systems::helpers::get_component::<SkillSet>(world, *entity).check(
                    Skill::Initiative,
                    world,
                    *entity,
                );
                (entity.clone(), roll)
            })
            .collect();

        indexed_rolls.sort_by_key(|(_, roll)| -(roll.total as i32));
        self.initiative_order = indexed_rolls
            .into_iter()
            .map(|(i, roll)| (i, roll))
            .collect();
    }

    pub fn initiative_order(&self) -> &Vec<(Entity, D20CheckResult)> {
        &self.initiative_order
    }

    pub fn current_entity(&self) -> Entity {
        let (idx, _) = self.initiative_order[self.turn_index];
        idx
    }

    pub fn participants(&self) -> &HashSet<Entity> {
        &self.participants
    }

    pub fn next_prompt(&self) -> Option<&ActionPrompt> {
        self.pending_prompts.front()
    }

    // TODO: Method name?
    pub fn process(
        &mut self,
        world: &mut World,
        decision: ActionDecision,
    ) -> Result<ActionDecisionResult, ActionError> {
        if self.pending_prompts.is_empty() {
            return Err(ActionError::MissingPrompt { decision });
        }

        let prompt = self.pending_prompts.front().unwrap();
        // Ensure the decision matches the current prompt
        prompt.is_valid_decision(&decision)?;

        match (prompt, &decision) {
            (ActionPrompt::Action { .. }, ActionDecision::Action { action }) => {
                // If the decision is currently pending it has already been
                // reacted to, so we can skip it
                if !self.pending_decisions.contains(&decision) {
                    // Check if anyone can react to this action
                    for reactor in &self.participants {
                        let reactions = systems::actions::available_reactions_to_action(
                            world,
                            *reactor,
                            action.actor,
                            &action.action_id,
                            &action.context,
                            &action.targets,
                        );
                        if !reactions.is_empty() {
                            // If available, prompt the reactor for a reaction
                            self.pending_prompts.push_back(ActionPrompt::Reaction {
                                reactor: *reactor,
                                action: action.clone(),
                                options: reactions
                                    .iter()
                                    .map(|(reaction_id, contexts, resource_cost, kind)| {
                                        ReactionData {
                                            reaction_id: reaction_id.clone(),
                                            contexts: contexts.clone(),
                                            resource_cost: resource_cost.clone(),
                                            kind: kind.clone(),
                                        }
                                    })
                                    .collect(),
                            });
                            // Add the decision to pending decisions and resolve it
                            // after the reaction is resolved
                            self.pending_decisions.push_back(decision.clone());
                            return Ok(ActionDecisionResult::ReactionTriggered {
                                reactor: *reactor,
                                action: action.clone(),
                            });
                        }
                    }
                }

                // Process the action decision
                let snapshots = systems::actions::perform_action(
                    world,
                    action.actor,
                    &action.action_id,
                    &action.context,
                    action.targets.len(),
                );
                // TODO: When the action is performed it's still the same entity's turn,
                // so we can either pop the prompt and then push a new one, or just
                // keep the current prompt in the queue. Functionally it's the same
                // if we just leave the prompt as is.
                // self.pending_prompts.pop_front();

                let results = systems::actions::apply_to_targets(world, snapshots, &action.targets);
                return Ok(ActionDecisionResult::ActionPerformed {
                    action: action.clone(),
                    results,
                });
            }

            (
                ActionPrompt::Reaction { .. },
                ActionDecision::Reaction {
                    reactor,
                    action,
                    choice,
                },
            ) => {
                if choice.is_none() {
                    // If no reaction was chosen just process the pending decision
                    self.pending_prompts.pop_front();
                    let pending_decision = self.pending_decisions.pop_front();
                    return self.process(world, pending_decision.unwrap());
                }

                let reaction = choice.as_ref().unwrap();
                match &reaction.kind {
                    ReactionKind::ModifyAction {
                        action,
                        // modification,
                    } => {
                        todo!("Handle modify action reaction");
                    }

                    ReactionKind::NewAction {
                        action,
                        context,
                        targets,
                    } => {
                        todo!("Handle new action from reaction");
                    }

                    ReactionKind::CancelAction {
                        action: action_id,
                        context,
                        targets,
                    } => {
                        // TODO: Not sure how much validation we need here?
                        self.pending_prompts.pop_front();
                        self.pending_decisions.pop_front().unwrap();
                        return Ok(ActionDecisionResult::ActionCancelled {
                            reaction: reaction.clone(),
                            action: ActionData {
                                actor: action.actor,
                                action_id: action_id.clone(),
                                context: context.clone(),
                                targets: targets.to_vec(),
                            },
                        });
                    }
                }
            }

            _ => {
                return Err(ActionError::PromptDecisionMismatch {
                    prompt: prompt.clone(),
                    decision,
                });
            }
        }
    }

    pub fn end_turn(&mut self, world: &mut World, entity: Entity) {
        if entity != self.current_entity() {
            panic!("Cannot end turn for entity that is not the current entity");
        }

        for prompt in self.pending_prompts.iter() {
            if prompt.actor() != entity {
                panic!("Cannot end turn while there are pending prompts for other entities");
            }
        }

        self.pending_prompts.clear();

        self.turn_index = (self.turn_index + 1) % self.participants.len();
        if self.turn_index == 0 {
            self.round += 1;
        }

        self.start_turn(world);
    }

    fn start_turn(&mut self, world: &mut World) {
        systems::turns::on_turn_start(world, self.current_entity());

        self.pending_prompts.push_back(ActionPrompt::Action {
            actor: self.current_entity(),
        });
    }

    pub fn round(&self) -> usize {
        self.round
    }
}

// impl ActionProvider for Encounter<'_> {
//     fn all_actions(&self) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
//         self.current_character().all_actions()
//     }

//     fn available_actions(
//         &self,
//     ) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
//         self.current_character().available_actions()
//     }
// }

use std::collections::{HashSet, VecDeque};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::{action::ReactionKind, targeting::TargetType},
        d20::{D20CheckDC, D20CheckResult},
        health::life_state::{DEATH_SAVING_THROW_DC, LifeState},
        id::{ActionId, EncounterId},
        modifier::{ModifierSet, ModifierSource},
        resource::RechargeRule,
        saving_throw::SavingThrowKind,
        skill::{Skill, SkillSet},
    },
    engine::game_state::{ActionData, EventLog, GameEvent, ReactionData},
    entities::{character::CharacterTag, monster::MonsterTag},
    systems,
};

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

pub enum ParticipantsFilter {
    All,
    Characters,
    Monsters,
    Specific(HashSet<Entity>),
    LifeStates(HashSet<LifeState>),
    NotLifeStates(HashSet<LifeState>),
}

impl From<TargetType> for ParticipantsFilter {
    fn from(value: TargetType) -> Self {
        match value {
            TargetType::Entity {
                allowed_states,
                invert,
            } => {
                if invert {
                    ParticipantsFilter::NotLifeStates(allowed_states)
                } else {
                    ParticipantsFilter::LifeStates(allowed_states)
                }
            }
        }
    }
}

pub struct Encounter {
    id: EncounterId,
    participants: HashSet<Entity>,
    round: usize,
    turn_index: usize,
    initiative_order: Vec<(Entity, D20CheckResult)>,
    pending_prompts: VecDeque<ActionPrompt>,
    /// In case a reaction is requested, this will hold the pending decisions
    /// until the reaction is resolved. The decision will then be processed once
    /// the reaction is resolved. In most cases this will be a single decision.
    pending_decisions: VecDeque<ActionDecision>,
    event_log: EventLog,
}

impl Encounter {
    pub fn new(world: &mut World, participants: HashSet<Entity>, id: EncounterId) -> Self {
        let mut encounter = Self {
            id,
            participants,
            round: 1,
            turn_index: 0,
            initiative_order: Vec::new(),
            pending_prompts: VecDeque::new(),
            pending_decisions: VecDeque::new(),
            event_log: Vec::new(),
        };
        encounter.roll_initiative(world);
        encounter.start_turn(world);
        encounter
            .event_log
            .push(GameEvent::NewRound(encounter.id.clone(), encounter.round()));
        encounter
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

    pub fn participants(&self, world: &World, filter: ParticipantsFilter) -> Vec<Entity> {
        match filter {
            ParticipantsFilter::All => self.participants.iter().cloned().collect(),

            ParticipantsFilter::Characters => world
                .query::<&CharacterTag>()
                .iter()
                .map(|(e, _)| e)
                .collect(),

            ParticipantsFilter::Monsters => world
                .query::<&MonsterTag>()
                .iter()
                .map(|(e, _)| e)
                .collect(),

            ParticipantsFilter::Specific(entities) => {
                self.participants.intersection(&entities).cloned().collect()
            }

            ParticipantsFilter::LifeStates(life_states) => world
                .query::<&LifeState>()
                .iter()
                .filter_map(|(e, ls)| {
                    if life_states.contains(ls) {
                        Some(e)
                    } else {
                        None
                    }
                })
                .collect(),

            ParticipantsFilter::NotLifeStates(life_states) => world
                .query::<&LifeState>()
                .iter()
                .filter_map(|(e, ls)| {
                    if !life_states.contains(ls) {
                        Some(e)
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    pub fn next_prompt(&self) -> Option<&ActionPrompt> {
        self.pending_prompts.front()
    }

    // TODO: Method name?
    pub fn process(
        &mut self,
        world: &mut World,
        decision: ActionDecision,
    ) -> Result<GameEvent, ActionError> {
        if self.pending_prompts.is_empty() {
            return Err(ActionError::MissingPrompt { decision });
        }

        let prompt = self.pending_prompts.front().unwrap();

        let result = self.resolve_decision(world, &prompt.clone(), decision);

        self.log(&result);

        result
    }

    fn resolve_decision(
        &mut self,
        world: &mut World,
        prompt: &ActionPrompt,
        decision: ActionDecision,
    ) -> Result<GameEvent, ActionError> {
        // Ensure the decision matches the current prompt
        prompt.is_valid_decision(&decision)?;

        match (prompt, &decision) {
            (ActionPrompt::Action { .. }, ActionDecision::Action { action }) => {
                // If the decision is currently pending it has already been
                // reacted to, so we don't have to check for reactions
                if !self.pending_decisions.contains(&decision) {
                    // Check if anyone can react to this action
                    for reactor in &self.participants {
                        println!(
                            "Checking for reactions for reactor: {:?} to action: {:?}",
                            reactor, action
                        );
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
                            self.pending_prompts.push_front(ActionPrompt::Reaction {
                                reactor: *reactor,
                                action: action.clone(),
                                options: reactions
                                    .iter()
                                    .flat_map(|(reaction_id, contexts, resource_cost, kind)| {
                                        contexts.iter().map(move |context| ReactionData {
                                            reaction_id: reaction_id.clone(),
                                            context: context.clone(),
                                            resource_cost: resource_cost.clone(),
                                            kind: kind.clone(),
                                        })
                                    })
                                    .collect(),
                            });
                            // Add the decision to pending decisions and resolve it
                            // after the reaction is resolved
                            self.pending_decisions.push_back(decision.clone());
                            return Ok(GameEvent::ReactionTriggered {
                                reactor: *reactor,
                                action: action.clone(),
                            });
                        }
                    }
                } else {
                    // Remove the decision from pending decisions and process it
                    self.pending_decisions.pop_front();
                }

                // Process the action decision
                let results = systems::actions::perform_action(
                    world,
                    action.actor,
                    &action.action_id,
                    &action.context,
                    &action.targets,
                );

                // Notice that we are not modifying the prompt queue here since
                // after performing the action it's still the same entity's turn.
                // The new prompt is then identically the same as the old one
                // (both are prompts for the current entity to take an action).

                return Ok(GameEvent::ActionPerformed {
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

                    let result = self.resolve_decision(
                        world,
                        &self.pending_prompts.front().unwrap().clone(),
                        self.pending_decisions.front().unwrap().clone(),
                    );
                    self.log(&Ok(GameEvent::NoReactionTaken {
                        reactor: *reactor,
                        action: action.clone(),
                    }));
                    return result;
                }

                let reaction = choice.as_ref().unwrap();

                // Perform the reaction to consume resources / apply cooldowns
                // TODO: Do we need to do anything with the results?
                let _ = systems::actions::perform_action(
                    world,
                    *reactor,
                    &reaction.reaction_id,
                    &reaction.context,
                    // TODO: How many snapshots to take?
                    &[],
                );

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
                        consume_resources,
                    } => {
                        // TODO: Not sure how much validation we need here?
                        self.pending_prompts.pop_front();
                        self.pending_decisions.pop_front().unwrap();

                        if *consume_resources {
                            // Easiest way to consume resources is to just perform
                            // the action with the given context (and 0 targets)
                            systems::actions::perform_action(
                                world,
                                action.actor,
                                action_id,
                                context,
                                &[],
                            );
                        }

                        return Ok(GameEvent::ActionCancelled {
                            reactor: *reactor,
                            reaction: reaction.clone(),
                            action: action.clone(),
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

    fn log(&mut self, result: &Result<GameEvent, ActionError>) {
        if let Ok(action_result) = result {
            self.event_log.push(action_result.clone());
        } else if let Err(err) = result {
            eprintln!("Error processing action: {:?}", err);
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
            self.event_log
                .push(GameEvent::NewRound(self.id.clone(), self.round));
        }

        self.start_turn(world);
    }

    // TODO: Some of this feels like it should belong somewhere else?
    fn should_skip_turn(&mut self, world: &mut World) -> bool {
        let current_entity = self.current_entity();

        // Borrow checker forces potential death saving throws to be handled in
        // a specific order

        // 1) Peek life state without taking a mutable borrow
        let is_unconscious = matches!(
            *systems::helpers::get_component::<LifeState>(world, current_entity),
            LifeState::Unconscious(_)
        );

        if is_unconscious {
            // 2) Do the d20 roll (needs only &World)
            let check_dc = D20CheckDC {
                dc: ModifierSet::from_iter([(
                    ModifierSource::Custom("Death Saving Throw".to_string()),
                    DEATH_SAVING_THROW_DC as i32,
                )]),
                key: SavingThrowKind::Death,
            };
            let check_result = systems::d20::saving_throw_dc(world, current_entity, &check_dc);
            self.event_log.push(GameEvent::SavingThrow(
                current_entity,
                check_result.clone(),
                check_dc.clone(),
            ));

            // 3) Now take the mutable borrow and update
            let mut life_state =
                systems::helpers::get_component_mut::<LifeState>(world, current_entity);
            if let LifeState::Unconscious(ref mut death_saving_throws) = *life_state {
                death_saving_throws.update(check_result);

                let next_state = death_saving_throws.next_state();

                if next_state != *life_state {
                    *life_state = next_state.clone();

                    self.event_log.push(GameEvent::LifeStateChanged {
                        entity: current_entity,
                        new_state: next_state,
                        actor: None,
                    });
                }
            }

            return true;
        } else {
            // Normal / other states => decide if they can act
            return !matches!(
                *systems::helpers::get_component::<LifeState>(world, current_entity),
                LifeState::Normal
            );
        };
    }

    fn start_turn(&mut self, world: &mut World) {
        systems::time::pass_time(world, self.current_entity(), &RechargeRule::OnTurn);

        if self.should_skip_turn(world) {
            self.end_turn(world, self.current_entity());
            return;
        }

        self.pending_prompts.push_back(ActionPrompt::Action {
            actor: self.current_entity(),
        });
    }

    pub fn round(&self) -> usize {
        self.round
    }

    pub fn combat_log(&self) -> &EventLog {
        &self.event_log
    }

    pub fn combat_log_move(&mut self) -> EventLog {
        std::mem::take(&mut self.event_log)
    }
}

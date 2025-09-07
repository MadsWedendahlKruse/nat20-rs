use std::{collections::VecDeque, f32::consts::E, ops::Deref, sync::Arc};

use hecs::{Entity, World};
use uuid::Uuid;

use crate::{
    components::{
        actions::action::{ActionContext, ActionResult, ReactionResult},
        d20::{D20CheckDC, D20CheckResult},
        damage::DamageRollResult,
        health::life_state::LifeState,
        id::ActionId,
        resource::ResourceCostMap,
        saving_throw::SavingThrowKind,
        skill::Skill,
    },
    engine::{
        encounter::{Encounter, EncounterId},
        game_state::GameState,
    },
    systems::{
        self,
        d20::{D20CheckDCKind, D20ResultKind},
    },
};

pub type EventId = Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: EventId,
    pub kind: EventKind,
    pub response_to: Option<EventId>,
}

impl Event {
    pub fn new(kind: EventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            response_to: None,
        }
    }

    pub fn as_response_to(mut self, event_id: EventId) -> Self {
        self.response_to = Some(event_id);
        self
    }

    pub fn encounter_event(encounter_event: EncounterEvent) -> Self {
        Self::new(EventKind::Encounter(encounter_event))
    }

    pub fn actor(&self) -> Option<Entity> {
        match &self.kind {
            EventKind::ActionRequested { action } => Some(action.actor),
            EventKind::ActionPerformed { action, .. } => Some(action.actor),
            EventKind::ReactionTriggered { reactor, .. } => Some(*reactor),
            EventKind::ReactionRequested { reactor, .. } => Some(*reactor),
            EventKind::ReactionPerformed { reactor, .. } => Some(*reactor),
            EventKind::LifeStateChanged { actor, .. } => *actor,
            EventKind::D20CheckPerformed(entity, _, _) => Some(*entity),
            EventKind::D20CheckResolved(entity, _, _) => Some(*entity),
            EventKind::DamageRollPerformed(entity, _) => Some(*entity),
            EventKind::DamageRollResolved(entity, _) => Some(*entity),
            _ => None,
        }
    }

    // TODO: I guess this is where the event actually "does" something?
    pub fn advance_event(&self, game_state: &mut GameState) {
        match &self.kind {
            // --- ACTION EVENTS ---
            EventKind::ActionRequested { action } => {
                // TODO: Where do we validate the action can be performed?
                systems::actions::perform_action(game_state, action);
            }

            // TODO: Probably don't have to do anything here?
            // EventKind::ActionPerformed { action, results } => todo!(),

            // --- REACTION EVENTS ---
            EventKind::ReactionRequested {
                reactor,
                reaction,
                event,
            } => match reaction.deref() {
                ReactionResult::NewEvent { event } => todo!(),
                ReactionResult::ModifyEvent { event } => todo!(),
                ReactionResult::CancelEvent { event_id } => todo!(),
                ReactionResult::NoEffect => todo!(),
            },

            _ => {} // No follow-up event

            EventKind::D20CheckPerformed(entity, kind, dc_kind) => {
                game_state.process_event(
                    Event::new(EventKind::D20CheckResolved(
                        *entity,
                        kind.clone(),
                        dc_kind.clone(),
                    ))
                    .as_response_to(self.id),
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Encounter(EncounterEvent),
    /// An entity has declared they want to take an action. The engine can then
    /// validate that the entity can perform the action and either approve or
    /// deny it. Other entities might also react to the request, e.g. if someone
    /// is casting a spell, another entity might use their reaction to Counterspell
    /// the action.
    ActionRequested {
        action: ActionData,
    },
    /// The action was successfully performed, and the results are applied to the targets.
    ActionPerformed {
        action: ActionData,
        results: Vec<ActionResult>,
    },
    ReactionTriggered {
        reactor: Entity,
        /// The event that triggered the reaction, e.g. an ActionRequested event
        /// might trigger a Counterspell reaction.
        trigger_event: Arc<Event>,
    },
    ReactionRequested {
        reactor: Entity,
        reaction: Arc<ReactionResult>,
        event: Arc<Event>,
    },
    ReactionPerformed {
        reactor: Entity,
        reaction: Arc<ReactionData>,
        event: Arc<Event>,
    },
    LifeStateChanged {
        entity: Entity,
        new_state: LifeState,
        /// The entity that caused the change, if any
        actor: Option<Entity>,
    },
    /// The initial D20 roll which can be reacted to, e.g. with the Lucky feat.
    D20CheckPerformed(Entity, D20ResultKind, Option<D20CheckDCKind>),
    /// The final result of a D20 check after reactions have been applied.
    D20CheckResolved(Entity, D20ResultKind, Option<D20CheckDCKind>),
    DamageRollPerformed(Entity, DamageRollResult),
    DamageRollResolved(Entity, DamageRollResult),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncounterEvent {
    EncounterStarted(EncounterId),
    EncounterEnded(EncounterId, EventLog),
    NewRound(EncounterId, usize),
}

pub type EventLog = Vec<Event>;
pub type EventQueue = VecDeque<Event>;

// TODO: struct name?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionData {
    pub actor: Entity,
    pub action_id: ActionId,
    pub context: ActionContext,
    pub targets: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReactionData {
    pub reaction_id: ActionId,
    pub context: ActionContext,
    // pub resource_cost: ResourceCostMap,
    pub kind: ReactionResult,
}

#[derive(Clone)]
pub struct EventListener {
    pub trigger_id: EventId,
    // pub filter: EventFilter,
    pub callback: EventCallback,
}

// type EventFilter = Arc<dyn Fn(&Event) -> bool + Send + Sync + 'static>;
type EventCallback = Arc<dyn Fn(&mut GameState, &Event) -> EventOrListener + Send + Sync + 'static>;

pub enum EventOrListener {
    Event(Event),
    Listener(EventListener),
}

impl EventListener {
    pub fn matches(&self, event: &Event) -> bool {
        if let Some(id) = event.response_to {
            if id == self.trigger_id {
                return true;
            }
        }
        return false;
        // (self.filter)(event)
    }

    pub fn callback(&self, game_state: &mut GameState, event: &Event) {
        let result = (self.callback)(game_state, event);
        match result {
            EventOrListener::Event(event) => {
                game_state.process_event(event);
            }
            EventOrListener::Listener(listener) => {
                game_state.add_event_listener(listener);
            }
        }
    }
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
        /// The event that triggered the reaction
        event: Event,
        /// The options available for the reaction. When responding to the prompt,
        /// the player must choose one of these options, or choose not to react.
        options: Vec<ReactionResult>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionDecision {
    Action {
        action: ActionData,
    },
    Reaction {
        /// The entity that is reacting
        reactor: Entity,
        /// The action this is a reaction to
        event: Event,
        /// The choice made by the player in response to the reaction prompt.
        /// This will be `None` if the player chose not to react.
        choice: Option<ReactionResult>,
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
    NotYourTurn {
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
            ActionPrompt::Reaction { event, .. } => event.actor().unwrap(),
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
                    event: prompt_event,
                    options,
                },
                ActionDecision::Reaction {
                    reactor: decision_reactor,
                    event: decision_event,
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
                    prompt_event.id,
                    decision_event.id,
                    "event_id",
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
            ActionDecision::Reaction { event, .. } => event.actor().unwrap(),
        }
    }

    // pub fn action_id(&self) -> &ActionId {
    //     match self {
    //         ActionDecision::Action { action, .. } => &action.action_id,
    //         ActionDecision::Reaction { action, .. } => &action.action_id,
    //     }
    // }
}

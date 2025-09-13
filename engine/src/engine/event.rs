use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use hecs::Entity;
use uuid::Uuid;

use crate::{
    components::{
        actions::action::{ActionContext, ActionResult},
        damage::DamageRollResult,
        health::life_state::LifeState,
        id::ActionId,
        resource::ResourceCostMap,
    },
    engine::{encounter::EncounterId, game_state::GameState},
    systems::d20::{D20CheckDCKind, D20ResultKind},
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
            EventKind::LifeStateChanged { actor, .. } => *actor,
            EventKind::D20CheckPerformed(entity, _, _) => Some(*entity),
            EventKind::D20CheckResolved(entity, _, _) => Some(*entity),
            EventKind::DamageRollPerformed(entity, _) => Some(*entity),
            EventKind::DamageRollResolved(entity, _) => Some(*entity),
            _ => None,
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
    // ReactionPerformed {
    //     reactor: Entity,
    //     reaction: Arc<ReactionData>,
    //     event: Arc<Event>,
    // },
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventLog {
    pub events: Vec<Event>,
    /// Track which entities have reacted to which events in this log. This is
    /// used to prevent an entity from reacting to the same event multiple times.
    /// TODO: Not sure if this is the best solution.
    pub reactors: HashMap<EventId, HashSet<Entity>>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            reactors: HashMap::new(),
        }
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn record_reaction(&mut self, event_id: EventId, reactor: Entity) {
        self.reactors
            .entry(event_id)
            .or_insert_with(HashSet::new)
            .insert(reactor);
    }

    pub fn has_reacted(&self, event_id: &EventId, reactor: &Entity) -> bool {
        if let Some(reactors) = self.reactors.get(event_id) {
            return reactors.contains(reactor);
        }
        false
    }
}

pub type EventQueue = VecDeque<Event>;

// TODO: struct name?
#[derive(Debug, Clone, PartialEq)]
pub struct ActionData {
    pub actor: Entity,
    pub action_id: ActionId,
    pub context: ActionContext,
    pub targets: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReactionData {
    pub reactor: Entity,
    // The event that triggered this reaction
    pub event: Arc<Event>,
    pub reaction_id: ActionId,
    pub context: ActionContext,
    pub resource_cost: ResourceCostMap,
}

impl From<&ReactionData> for ActionData {
    fn from(value: &ReactionData) -> Self {
        ActionData {
            actor: value.reactor,
            action_id: value.reaction_id.clone(),
            context: value.context.clone(),
            targets: vec![value.event.actor().unwrap()], // TODO: What if no actor?
        }
    }
}

#[derive(Clone)]
pub struct EventListener {
    trigger_id: EventId,
    callback: EventCallback,
}

type EventCallback = Arc<dyn Fn(&mut GameState, &Event) -> CallbackResult + Send + Sync + 'static>;

pub enum CallbackResult {
    Event(Event),
    EventWithCallback(Event, EventCallback),
}

impl EventListener {
    pub fn new(trigger_id: EventId, callback: EventCallback) -> Self {
        Self {
            trigger_id,
            callback,
        }
    }

    pub fn trigger_id(&self) -> EventId {
        self.trigger_id
    }

    pub fn matches(&self, event: &Event) -> bool {
        if let Some(id) = event.response_to {
            if id == self.trigger_id {
                return true;
            }
        }
        return false;
    }

    pub fn callback(&self, game_state: &mut GameState, event: &Event) {
        let result = (self.callback)(game_state, event);
        match result {
            CallbackResult::Event(event) => {
                let _ = game_state.process_event(event);
            }
            CallbackResult::EventWithCallback(event, callback) => {
                let event_id = event.id;
                game_state
                    .process_event_with_listener(event, EventListener::new(event_id, callback));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionPrompt {
    /// Prompt an entity to perform an action
    Action {
        /// The entity that should perform the action
        actor: Entity,
    },
    /// Prompt all entities that can react to an event to make a reaction decision.
    /// While actions are prompted one at a time, an action can trigger multiple
    /// reactions, and we need to give everyone a fair chance to react before we
    /// can proceede
    Reactions {
        /// The event that triggered the reactions
        event: Event,
        /// The options available for those reacting. The key is the entity which
        /// is reaction, and the value is their options
        options: HashMap<Entity, Vec<ReactionData>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionDecisionPartial {
    /// For actions this is the full decision
    Action { action: ActionData },
    /// For reactions this is just the choice made by a single reactor
    Reaction {
        reactor: Entity,
        event: Event,
        choice: Option<ReactionData>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionDecision {
    Action {
        action: ActionData,
    },
    Reactions {
        /// The event that triggered the reaction
        event: Event,
        /// The choices made by each entity which could react to the event. If the
        /// entity chose not to react, this will be 'None'
        choices: HashMap<Entity, Option<ReactionData>>,
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
        prompts: Vec<ActionPrompt>,
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
    pub fn actors(&self) -> Vec<Entity> {
        match self {
            ActionPrompt::Action { actor } => vec![*actor],
            ActionPrompt::Reactions { options, .. } => options.keys().cloned().collect(),
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
                ActionPrompt::Reactions {
                    event: prompt_event,
                    options,
                },
                ActionDecision::Reactions {
                    event: decision_event,
                    choices,
                },
            ) => {
                ensure_equal!(
                    options.keys().collect::<HashSet<_>>(),
                    choices.keys().collect::<HashSet<_>>(),
                    "reactors",
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

                for reactor in options.keys() {
                    if let Some(choice) = choices.get(reactor).unwrap() {
                        if !options.get(reactor).unwrap().contains(&choice) {
                            return Err(ActionError::FieldMismatch {
                                field: "choices",
                                expected: format!("one of {:?}", options),
                                actual: format!("{:?}", choice),
                                prompt: self.clone(),
                                decision: decision.clone(),
                            });
                        }
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
    pub fn actors(&self) -> Vec<Entity> {
        match self {
            ActionDecision::Action { action, .. } => vec![action.actor],
            ActionDecision::Reactions { choices, .. } => choices.keys().cloned().collect(),
        }
    }
}

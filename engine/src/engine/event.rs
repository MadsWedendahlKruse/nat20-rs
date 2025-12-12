use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use hecs::Entity;
use uuid::Uuid;

use crate::{
    components::{
        actions::{
            action::{ActionContext, ActionKindResult, ActionResult},
            targeting::TargetInstance,
        },
        damage::DamageRollResult,
        health::life_state::LifeState,
        id::ActionId,
        resource::{ResourceAmountMap, ResourceError},
    },
    engine::{encounter::EncounterId, game_state::GameState},
    systems::{
        actions::ActionUsabilityError,
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
            // TODO: What to do here? Multiple reactors?
            EventKind::ReactionTriggered { reactors, .. } => Some(*reactors.iter().next()?),
            EventKind::ReactionRequested { reaction } => Some(reaction.reactor),
            EventKind::LifeStateChanged { entity, actor, .. } => {
                if let Some(actor) = actor {
                    Some(*actor)
                } else {
                    Some(*entity)
                }
            }
            EventKind::D20CheckPerformed(entity, _, _) => Some(*entity),
            EventKind::D20CheckResolved(entity, _, _) => Some(*entity),
            EventKind::DamageRollPerformed(entity, _) => Some(*entity),
            EventKind::DamageRollResolved(entity, _) => Some(*entity),
            EventKind::Encounter(_) => None,
        }
    }

    pub fn target(&self) -> Option<Entity> {
        match &self.kind {
            EventKind::ActionRequested { action } => {
                if let Some(TargetInstance::Entity(target)) = action.targets.first() {
                    Some(*target)
                } else {
                    None
                }
            }
            EventKind::ActionPerformed { action, .. } => {
                if let Some(TargetInstance::Entity(target)) = action.targets.first() {
                    Some(*target)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn action_performed_event(
        game_state: &GameState,
        performer: Entity,
        action_id: &ActionId,
        context: &ActionContext,
        resource_cost: &ResourceAmountMap,
        target: Entity,
        result: ActionKindResult,
    ) -> Event {
        Event::new(EventKind::ActionPerformed {
            action: ActionData {
                actor: performer,
                action_id: action_id.clone(),
                context: context.clone(),
                resource_cost: resource_cost.clone(),
                targets: vec![TargetInstance::Entity(target)],
            },
            results: vec![ActionResult::new(
                &game_state.world,
                performer,
                target,
                result,
            )],
        })
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
        /// The event that triggered the reaction, e.g. an ActionRequested event
        /// might trigger a Counterspell reaction.
        trigger_event: Arc<Event>,
        reactors: HashSet<Entity>,
    },
    ReactionRequested {
        reaction: ReactionData,
    },
    LifeStateChanged {
        entity: Entity,
        new_state: LifeState,
        /// The entity that caused the change, if any
        actor: Option<Entity>,
    },
    /// The initial D20 roll which can be reacted to, e.g. with the Lucky feat.
    D20CheckPerformed(Entity, D20ResultKind, D20CheckDCKind),
    /// The final result of a D20 check after reactions have been applied.
    D20CheckResolved(Entity, D20ResultKind, D20CheckDCKind),
    DamageRollPerformed(Entity, DamageRollResult),
    DamageRollResolved(Entity, DamageRollResult),
}

impl EventKind {
    pub fn name(&self) -> &'static str {
        match self {
            EventKind::Encounter(_) => "Encounter",
            EventKind::ActionRequested { .. } => "ActionRequested",
            EventKind::ActionPerformed { .. } => "ActionPerformed",
            EventKind::ReactionTriggered { .. } => "ReactionTriggered",
            EventKind::ReactionRequested { .. } => "ReactionRequested",
            EventKind::LifeStateChanged { .. } => "LifeStateChanged",
            EventKind::D20CheckPerformed(_, _, _) => "D20CheckPerformed",
            EventKind::D20CheckResolved(_, _, _) => "D20CheckResolved",
            EventKind::DamageRollPerformed(_, _) => "DamageRollPerformed",
            EventKind::DamageRollResolved(_, _) => "DamageRollResolved",
        }
    }
}

// TODO: Do we need this?
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
    pub resource_cost: ResourceAmountMap,
    pub targets: Vec<TargetInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReactionData {
    pub reactor: Entity,
    // The event that triggered this reaction
    pub event: Arc<Event>,
    pub reaction_id: ActionId,
    pub context: ActionContext,
    pub resource_cost: ResourceAmountMap,
}

impl From<&ReactionData> for ActionData {
    fn from(reaction: &ReactionData) -> Self {
        ActionData {
            actor: reaction.reactor,
            action_id: reaction.reaction_id.clone(),
            context: reaction.context.clone(),
            resource_cost: reaction.resource_cost.clone(),
            targets: vec![TargetInstance::Entity(reaction.event.actor().unwrap())], // TODO: What if no actor?
        }
    }
}

#[derive(Clone)]
pub struct EventListener {
    trigger_id: EventId,
    callback: EventCallback,
}

pub type EventCallback =
    Arc<dyn Fn(&mut GameState, &Event) -> CallbackResult + Send + Sync + 'static>;

pub enum CallbackResult {
    Event(Event),
    EventWithCallback(Event, EventCallback),
    None,
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
                game_state.process_event(event);
            }
            CallbackResult::EventWithCallback(event, callback) => {
                game_state.process_event_with_callback(event, callback);
            }
            CallbackResult::None => {}
        }
    }
}

pub type ActionPromptId = Uuid;

#[derive(Debug, Clone)]
pub enum ActionPromptKind {
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

#[derive(Debug, Clone)]
pub struct ActionPrompt {
    pub id: ActionPromptId,
    pub kind: ActionPromptKind,
}

impl ActionPrompt {
    pub fn new(kind: ActionPromptKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionDecisionKind {
    Action {
        action: ActionData,
    },
    Reaction {
        /// The event that triggered the reaction
        event: Event,
        reactor: Entity,
        /// The chosen reaction. None if the entity chooses not to react
        choice: Option<ReactionData>,
    },
}

#[derive(Debug, Clone)]
pub struct ActionDecision {
    pub response_to: ActionPromptId,
    pub kind: ActionDecisionKind,
}

#[derive(Debug, Clone)]
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
    Usability(ActionUsabilityError),
    Resource(ResourceError),
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
        match &self.kind {
            ActionPromptKind::Action { actor, .. } => vec![*actor],
            ActionPromptKind::Reactions { options, .. } => options.keys().cloned().collect(),
        }
    }

    pub fn is_valid_decision(&self, decision: &ActionDecision) -> Result<(), ActionError> {
        ensure_equal!(
            self.id,
            decision.response_to,
            "response_to",
            FieldMismatch,
            self,
            decision
        );

        match (&self.kind, &decision.kind) {
            (
                ActionPromptKind::Action {
                    actor: prompt_actor,
                },
                ActionDecisionKind::Action { action },
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
                ActionPromptKind::Reactions {
                    event: prompt_event,
                    options,
                },
                ActionDecisionKind::Reaction {
                    event: decision_event,
                    reactor,
                    choice,
                },
            ) => {
                ensure_equal!(
                    prompt_event.id,
                    decision_event.id,
                    "event_id",
                    FieldMismatch,
                    self,
                    decision
                );

                if let Some(options) = options.get(&reactor) {
                    if let Some(choice) = choice
                        && !options.contains(&choice)
                    {
                        return Err(ActionError::FieldMismatch {
                            field: "choices",
                            expected: format!("one of {:?}", options),
                            actual: format!("{:?}", choice),
                            prompt: self.clone(),
                            decision: decision.clone(),
                        });
                    }
                } else {
                    return Err(ActionError::FieldMismatch {
                        field: "reactor",
                        expected: format!("one of {:?}", options.keys()),
                        actual: format!("{:?}", reactor),
                        prompt: self.clone(),
                        decision: decision.clone(),
                    });
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

impl ActionDecisionKind {
    pub fn actor(&self) -> Entity {
        match self {
            ActionDecisionKind::Action { action, .. } => action.actor,
            ActionDecisionKind::Reaction { reactor, .. } => *reactor,
        }
    }
}

impl ActionDecision {
    pub fn without_response_to(kind: ActionDecisionKind) -> Self {
        Self {
            response_to: Uuid::nil(),
            kind,
        }
    }

    pub fn actor(&self) -> Entity {
        self.kind.actor()
    }
}

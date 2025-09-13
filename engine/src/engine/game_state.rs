use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use hecs::{Entity, World};

use crate::{
    components::actions::{
        action::{ActionKindResult, ReactionResult},
        targeting::TargetType,
    },
    engine::{
        encounter::{Encounter, EncounterId, ParticipantsFilter},
        event::{
            ActionData, ActionDecision, ActionError, ActionPrompt, EncounterEvent, Event, EventId,
            EventKind, EventListener, EventLog, EventQueue,
        },
    },
    systems,
};

// TODO: WorldState instead?
pub struct GameState {
    pub world: World,
    pub encounters: HashMap<EncounterId, Encounter>,
    pub in_combat: HashMap<Entity, EncounterId>,
    /// Pending prompts for entities that aren't in combat.
    /// Probably empty most of the time?
    pub pending_prompts: VecDeque<ActionPrompt>,

    pub event_log: EventLog,
    pending_events: EventQueue,
    event_listeners: HashMap<EventId, EventListener>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            encounters: HashMap::new(),
            in_combat: HashMap::new(),
            pending_prompts: VecDeque::new(),
            event_log: EventLog::new(),
            pending_events: EventQueue::new(),
            event_listeners: HashMap::new(),
        }
    }

    pub fn start_encounter_with_id(
        &mut self,
        participants: HashSet<Entity>,
        encounter_id: EncounterId,
    ) -> EncounterId {
        for entity in &participants {
            self.in_combat.insert(*entity, encounter_id.clone());
        }

        self.event_log
            .push(Event::encounter_event(EncounterEvent::EncounterStarted(
                encounter_id.clone(),
            )));

        let encounter = Encounter::new(self, participants, encounter_id.clone());

        self.encounters.insert(encounter_id.clone(), encounter);
        encounter_id
    }

    pub fn start_encounter(&mut self, participants: HashSet<Entity>) -> EncounterId {
        self.start_encounter_with_id(participants, EncounterId::new_v4())
    }

    pub fn encounter(&self, encounter_id: &EncounterId) -> Option<&Encounter> {
        self.encounters.get(encounter_id)
    }

    pub fn encounter_mut(&mut self, encounter_id: &EncounterId) -> Option<&mut Encounter> {
        self.encounters.get_mut(encounter_id)
    }

    pub fn encounter_for_entity(&self, entity: &Entity) -> Option<&EncounterId> {
        self.in_combat.get(entity)
    }

    pub fn end_encounter(&mut self, encounter_id: &EncounterId) {
        if let Some(mut encounter) = self.encounters.remove(encounter_id) {
            for entity in encounter.participants(&self.world, ParticipantsFilter::All) {
                self.in_combat.remove(&entity);
            }
            self.event_log
                .push(Event::encounter_event(EncounterEvent::EncounterEnded(
                    encounter_id.clone(),
                    encounter.combat_log_move(),
                )));
        }
    }

    pub fn next_prompt(&self) -> Option<&ActionPrompt> {
        self.pending_prompts.front()
    }

    pub fn submit_decision(&mut self, decision: ActionDecision) -> Result<(), ActionError> {
        // Check that all of the actors involved in the decision have a are all
        // either in combat or all out of combat
        let mut encounter_id = None;
        for actor in decision.actors() {
            let actor_encounter_id = self.in_combat.get(&actor);
            if let Some(id) = encounter_id {
                if actor_encounter_id != Some(id) {
                    // TODO: If this ever triggers then it'd be nice with a bit
                    // more context
                    panic!(
                        "All actors in a decision must be either in combat or out of combat together"
                    );
                }
            } else {
                encounter_id = actor_encounter_id.as_ref().cloned();
            }
        }

        // If the entities are in combat, then check that there is a pending
        // prompt for them, and that the decision is valid for that prompt
        if let Some(encounter_id) = encounter_id {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                if encounter.pending_prompts().is_empty() {
                    return Err(ActionError::MissingPrompt {
                        decision,
                        prompts: encounter.pending_prompts().iter().cloned().collect(),
                    });
                }

                let prompt = encounter.pop_prompt().unwrap();
                prompt.is_valid_decision(&decision)?;
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        }

        self.process_decision(decision)?;

        Ok(())
    }

    fn process_decision(&mut self, decision: ActionDecision) -> Result<(), ActionError> {
        // Convert the decision into the appropriate event to process
        match &decision {
            ActionDecision::Action { action } => {
                self.process_event(Event::new(EventKind::ActionRequested {
                    action: action.clone(),
                }))
            }

            ActionDecision::Reactions { event, choices } => {
                for (_, choice) in choices {
                    // Process the chosen reactions, if any
                    if let Some(reaction) = choice {
                        self.process_event(Event::new(EventKind::ActionRequested {
                            action: ActionData::from(reaction),
                        }))?;
                    }
                }
                // After processing all reactions, continue with the pending event
                // (if any). Otherwise, if this happened inside an encounter, then
                // prompt the current actor for their next action
                if let Some(pending_event) = self.pending_events.pop_front() {
                    self.process_event(pending_event)?;
                } else if let Some(encounter_id) = self.in_combat.get(&event.actor().unwrap()) {
                    if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                        encounter.queue_prompt(
                            ActionPrompt::Action {
                                actor: encounter.current_entity(),
                            },
                            true,
                        );
                    } else {
                        panic!("Inconsistent state: entity is in combat but encounter not found");
                    }
                }

                Ok(())
            }
        }
    }

    /// pub(crate) to avoid injecting arbitrary events from outside the engine
    pub(crate) fn process_event(&mut self, event: Event) -> Result<(), ActionError> {
        self.log_event(event.clone());

        // 1. Check if the event is awaited (a response to a previous event)
        let mut awaited_event = None;
        if let Some(event_id) = event.response_to {
            if let Some(event_listener) = self.event_listeners.get(&event_id) {
                if event_listener.matches(&event) {
                    awaited_event = Some(&event);
                }
            }
        }
        if let Some(event) = awaited_event {
            let event_listener = self
                .event_listeners
                .remove(&event.response_to.unwrap())
                .unwrap();
            event_listener.callback(self, event);
        }

        // TODO: What about reactions outside of combat?
        // 2. If the actor is in combat, check if anyone can react to the event
        if let Some(actor) = event.actor() {
            if let Some(encounter_id) = self.in_combat.get(&actor) {
                if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                    let mut reaction_options = None;

                    for reactor in &encounter.participants(
                        &self.world,
                        ParticipantsFilter::from(TargetType::entity_not_dead()),
                    ) {
                        if self.event_log.has_reacted(&event.id, reactor) {
                            println!(
                                "Entity {:?} has already reacted to event {:?}",
                                reactor, event.id
                            );
                            continue;
                        }

                        let reactions = systems::actions::available_reactions_to_event(
                            &self.world,
                            *reactor,
                            &event,
                        );

                        println!(
                            "Entity {:?} has {} available reactions to event {:?}",
                            reactor,
                            reactions.len(),
                            event.id
                        );

                        if !reactions.is_empty() {
                            // Record the available reactions for this reactor
                            reaction_options
                                .get_or_insert_with(HashMap::new)
                                .insert(*reactor, reactions);

                            encounter.log_event(
                                Event::new(EventKind::ReactionTriggered {
                                    reactor: *reactor,
                                    trigger_event: event.clone().into(),
                                })
                                .as_response_to(event.id),
                            );

                            self.event_log.record_reaction(event.id, *reactor);
                        }
                    }

                    if let Some(options) = reaction_options {
                        // Prompt all reactors for their reaction
                        encounter.queue_prompt(
                            ActionPrompt::Reactions {
                                event: event.clone(),
                                options,
                            },
                            false,
                        );

                        // Save the event being reacted to for later processing
                        self.pending_events.push_back(event);
                        return Ok(());
                    }
                } else {
                    panic!("Inconsistent state: entity is in combat but encounter not found");
                }
            }
        }

        // 3. Advance the event
        self.advance_event(event);

        Ok(())
    }

    // TODO: I guess this is where the event actually "does" something? New name?
    pub fn advance_event(&mut self, event: Event) {
        match &event.kind {
            EventKind::ActionRequested { action } => {
                // TODO: Where do we validate the action can be performed?
                systems::actions::perform_action(self, action);
            }

            EventKind::ActionPerformed { results, .. } => {
                for action_result in results {
                    match &action_result.kind {
                        ActionKindResult::Reaction {
                            result: reaction_result,
                        } => {
                            match reaction_result {
                                ReactionResult::CancelEvent {
                                    event_id,
                                    resources_refunded,
                                } => {
                                    // TODO: Do something with the resources
                                    self.pending_events.retain(|e| &e.id != event_id);
                                }

                                // TODO: How to handle this properly?
                                ReactionResult::ModifyEvent { event } => {}

                                ReactionResult::NoEffect => { /* Do nothing */ }
                            }
                        }

                        _ => {}
                    }
                }
            }

            EventKind::D20CheckPerformed(entity, kind, dc_kind) => {
                let _ = self.process_event(
                    Event::new(EventKind::D20CheckResolved(
                        *entity,
                        kind.clone(),
                        dc_kind.clone(),
                    ))
                    .as_response_to(event.id),
                );
            }

            EventKind::DamageRollPerformed(entity, damage) => {
                let _ = self.process_event(
                    Event::new(EventKind::DamageRollResolved(*entity, damage.clone()))
                        .as_response_to(event.id),
                );
            }

            _ => {} // No follow-up event
        }
    }

    fn log_event(&mut self, event: Event) {
        // If the actor is in combat log it in the encounter log, otherwise in
        // the global log
        if let Some(actor) = event.actor() {
            if let Some(encounter_id) = self.in_combat.get(&actor) {
                if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                    encounter.log_event(event);
                } else {
                    panic!("Inconsistent state: entity is in combat but encounter not found");
                }
            } else {
                self.event_log.push(event);
            }
        } else {
            self.event_log.push(event);
        }
    }

    pub fn add_event_listener(&mut self, event_listener: EventListener) {
        self.event_listeners
            .insert(event_listener.trigger_id(), event_listener);
    }

    pub fn process_event_with_listener(
        &mut self,
        event: Event,
        event_listener: EventListener,
    ) -> Result<(), ActionError> {
        self.add_event_listener(event_listener);
        self.process_event(event)
    }
}

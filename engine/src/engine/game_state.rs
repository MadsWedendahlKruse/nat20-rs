use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use hecs::{Entity, World};

use crate::{
    components::actions::targeting::TargetType,
    engine::{
        encounter::{Encounter, EncounterId, ParticipantsFilter},
        event::{
            self, ActionDecision, ActionError, ActionPrompt, EncounterEvent, Event, EventId,
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
        let entity = decision.actor();

        // If the entity is in combat, then they're only allowed to submit
        // decisions if it's their turn and they have a pending prompt
        if let Some(encounter_id) = self.in_combat.get(&entity) {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                if self.pending_prompts.is_empty() {
                    return Err(ActionError::MissingPrompt { decision });
                }

                if encounter.current_entity() != entity {
                    return Err(ActionError::NotYourTurn { decision });
                }

                let prompt = encounter.next_prompt().unwrap();
                prompt.is_valid_decision(&decision)?
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        }

        let event = self.next_event(decision.clone())?;

        self.process_event(event)?;

        Ok(())
    }

    fn next_event(&mut self, decision: ActionDecision) -> Result<Event, ActionError> {
        // Convert the decision into the appropriate event to process
        match &decision {
            ActionDecision::Action { action } => Ok(Event::new(EventKind::ActionRequested {
                action: action.clone(),
            })),

            ActionDecision::Reaction {
                reactor,
                event,
                choice,
            } => {
                if let Some(reaction) = choice {
                    Ok(Event::new(EventKind::ReactionRequested {
                        reactor: *reactor,
                        reaction: reaction.clone().into(),
                        event: event.clone().into(),
                    }))
                } else {
                    // No reaction chosen, simply continue with the pending event
                    Ok(self.pending_events.pop_front().unwrap())
                }
            }
        }
    }

    /// pub(crate) to avoid injecting arbitrary events from outside the engine
    pub(crate) fn process_event(&mut self, event: Event) -> Result<(), ActionError> {
        self.log_event(event.clone());

        // 1. Check if the event is awaited (a response to a previous event)
        let mut listener_event = None;
        if let Some(event_id) = event.response_to {
            if let Some(event_listener) = self.event_listeners.get(&event_id) {
                if event_listener.matches(&event) {
                    listener_event = Some(&event);
                }
            }
        }
        if let Some(event) = listener_event {
            let event_listener = self.event_listeners.remove(&event.id).unwrap();
            (event_listener.callback)(self, event);
        }

        // 2. If the actor is in combat, check if anyone can react to the event
        if let Some(actor) = event.actor() {
            if let Some(encounter_id) = self.in_combat.get(&actor) {
                if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                    let mut reactions_triggered = false;
                    for reactor in &encounter.participants(
                        &self.world,
                        ParticipantsFilter::from(TargetType::entity_not_dead()),
                    ) {
                        let reactions = systems::actions::available_reactions_to_event(
                            &self.world,
                            *reactor,
                            &event,
                        );
                        if !reactions.is_empty() {
                            reactions_triggered = true;

                            // If available, prompt the reactor for a reaction
                            self.pending_prompts.push_front(ActionPrompt::Reaction {
                                reactor: *reactor,
                                event: event.clone(),
                                options: reactions,
                            });

                            self.event_log
                                .push(Event::new(EventKind::ReactionTriggered {
                                    reactor: *reactor,
                                    trigger_event: event.clone().into(),
                                }));
                            // Save the event for later processing
                            self.pending_events.push_back(event.clone());
                        }
                    }
                    if reactions_triggered {
                        return Ok(());
                    }
                } else {
                    panic!("Inconsistent state: entity is in combat but encounter not found");
                }
            }
        }

        // 3. Advance the event
        event.advance_event(self);

        Ok(())
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
            .insert(event_listener.trigger_id, event_listener);
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

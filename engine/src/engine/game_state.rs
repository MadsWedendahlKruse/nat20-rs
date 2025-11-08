use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::BufReader,
    path::Path,
};

use hecs::{Entity, World};
use obj::Obj;
use parry3d::na::Point3;

use crate::{
    components::actions::{
        action::{ActionKindResult, ReactionResult},
        targeting::EntityFilter,
    },
    engine::{
        encounter::{Encounter, EncounterId},
        event::{
            ActionData, ActionDecision, ActionDecisionKind, ActionError, ActionPrompt,
            ActionPromptId, ActionPromptKind, EncounterEvent, Event, EventCallback, EventId,
            EventKind, EventListener, EventLog, EventQueue,
        },
        geometry::WorldGeometry,
    },
    systems::{
        self,
        movement::{MovementError, PathResult},
    },
};

// TODO: WorldState instead?
pub struct GameState {
    pub world: World,
    pub geometry: WorldGeometry,

    pub encounters: HashMap<EncounterId, Encounter>,
    pub in_combat: HashMap<Entity, EncounterId>,
    /// Pending prompts for entities that aren't in combat.
    /// Probably empty most of the time?
    pub pending_prompts: VecDeque<ActionPrompt>,
    /// Decisions already submitted for prompts (out of combat). Is this needed?
    pub action_decisions: HashMap<ActionPromptId, HashMap<Entity, ActionDecision>>,
    pub event_log: EventLog,
    pending_events: EventQueue,
    event_listeners: HashMap<EventId, EventListener>,
}

impl GameState {
    pub fn new<P: AsRef<Path>>(world_geometry_path: P, navmesh_config: &rerecast::Config) -> Self {
        let obj: Obj = obj::load_obj(BufReader::new(File::open(world_geometry_path).unwrap()))
            .expect("Failed to load world geometry");

        Self {
            world: World::new(),
            encounters: HashMap::new(),
            in_combat: HashMap::new(),
            pending_prompts: VecDeque::new(),
            action_decisions: HashMap::new(),
            event_log: EventLog::new(),
            pending_events: EventQueue::new(),
            event_listeners: HashMap::new(),
            geometry: WorldGeometry::from_obj(obj, navmesh_config),
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
            for entity in encounter.participants(&self.world, EntityFilter::All) {
                self.in_combat.remove(&entity);
            }
            self.event_log
                .push(Event::encounter_event(EncounterEvent::EncounterEnded(
                    encounter_id.clone(),
                    encounter.combat_log_move(),
                )));
        }
    }

    pub fn end_turn(&mut self, entity: Entity) {
        let encounter = if let Some(encounter_id) = self.in_combat.get(&entity) {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                unsafe { Some(&mut *(encounter as *mut Encounter)) }
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        } else {
            None
        };

        if let Some(encounter) = encounter {
            encounter.end_turn(self, entity);

            // TODO: Handle next entity being an NPC with AI controller
        }
    }

    pub fn submit_movement(
        &mut self,
        entity: Entity,
        goal: Point3<f32>,
    ) -> Result<PathResult, MovementError> {
        if let Some(encounter_id) = self.in_combat.get(&entity) {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                if encounter.current_entity() != entity {
                    return Err(MovementError::NotYourTurn);
                }
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        }
        systems::movement::path(
            self,
            entity,
            &goal,
            true,
            true,
            self.in_combat.get(&entity).is_some(),
        )
    }

    pub fn next_prompt(&self) -> Option<&ActionPrompt> {
        self.pending_prompts.front()
    }

    pub fn submit_decision(&mut self, mut decision: ActionDecision) -> Result<(), ActionError> {
        println!("[GameState] Received decision: {:#?}", decision);
        let mut response_to = decision.response_to;
        // If the entities are in combat, then check that there is a pending
        // prompt for them, and that the decision is valid for that prompt
        let encounter_id = self.in_combat.get(&decision.actor()).cloned();
        if let Some(encounter_id) = encounter_id {
            if let Some(encounter) = self.encounters.get_mut(&encounter_id) {
                if encounter.pending_prompts().is_empty() {
                    return Err(ActionError::MissingPrompt {
                        decision,
                        // TODO: Combination of the if-statement and the error seems to
                        // mismatch. If there's no pending prompts, then why do we bother
                        // cloning all of them into a vector?
                        prompts: encounter.pending_prompts().iter().cloned().collect(),
                    });
                }

                let prompt = encounter.next_pending_prompt().unwrap();
                prompt.is_valid_decision(&decision)?;
                encounter.add_decision(decision);
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        } else {
            // TODO: Very unhappy with this solution
            // Out of combat prompts
            // Prompt order is irrelevant out of combat? Just find matching prompt
            // TODO: Is it better that we out of combat just always have a prompt
            // waiting for the entity?
            let prompt = {
                let found_prompt = self.pending_prompts.iter().find(|p| p.id == response_to);
                if found_prompt.is_none() {
                    self.pending_prompts
                        .push_back(ActionPrompt::new(ActionPromptKind::Action {
                            actor: decision.actor(),
                        }));
                    let new_prompt = self.pending_prompts.back().unwrap();
                    decision.response_to = new_prompt.id;
                    response_to = new_prompt.id;
                    new_prompt
                } else {
                    found_prompt.unwrap()
                }
            };

            prompt.is_valid_decision(&decision)?;

            self.action_decisions
                .entry(prompt.id)
                .or_insert_with(HashMap::new)
                .insert(decision.actor(), decision);
        }

        println!(
            "[GameState::submit_decision] Response to prompt ID: {:?} in encounter {:?}",
            response_to, encounter_id
        );

        self.process_decisions(response_to, encounter_id)?;

        Ok(())
    }

    fn process_decisions(
        &mut self,
        decision_response_to: ActionPromptId,
        encounter_id: Option<EncounterId>,
    ) -> Result<(), ActionError> {
        println!(
            "[GameState::process_decisions] Processing decisions for prompt ID: {:?} in encounter {:?}",
            decision_response_to, encounter_id
        );

        // Check that all involved entities have submitted their decisions
        let decisions = {
            let (prompt, decisions) = if let Some(encounter_id) = encounter_id {
                let encounter = self.encounters.get(&encounter_id).unwrap();
                let prompt = encounter
                .pending_prompts()
                .iter()
                .find(|p| p.id == decision_response_to)
                .expect(
                    format!("Inconsistent state: decision submitted, but prompt {:?} not found in encounter {:?} prompts: {:#?}",
                    decision_response_to,
                    encounter_id,
                    encounter.pending_prompts()).as_str(),
                );
                let decisions = encounter
                .decisions_for_prompt(&decision_response_to)
                .expect(
                    format!("Inconsistent state: decision submitted, but not found in encounter {:?} decisions: {:#?}",
                        encounter_id, encounter.decisions()).as_str(),
                );
                println!(
                    "[GameState::process_decisions] Retrieved prompt and decisions from encounter {:?}",
                    encounter_id
                );
                (prompt, decisions)
            } else {
                let prompt = self
                .pending_prompts
                .iter()
                .find(|p| p.id == decision_response_to)
                .expect("Inconsistent state: decision submitted, but prompt not found in pending_prompts");
                let decisions = self.action_decisions.get(&decision_response_to).expect(
                    "Inconsistent state: decision submitted, but not found in action_decisions",
                );
                println!(
                    "[GameState::process_decisions] Retrieved prompt and decisions from out-of-combat prompts"
                );
                (prompt, decisions)
            };

            if !prompt
                .actors()
                .iter()
                .all(|actor| decisions.contains_key(actor))
            {
                // Not all decisions have been submitted yet
                return Ok(());
            }

            // Avoid borrowing issues. Performance impact should be negligible here
            decisions.clone()
        };

        // All decisions have been submitted; pop the prompt
        if encounter_id.is_none() {
            self.pending_prompts
                .retain(|p| p.id != decision_response_to);
            self.action_decisions.remove(&decision_response_to);
        } else if let Some(encounter_id) = &encounter_id {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                encounter.pop_prompt_and_validate_next(&self.world, &self.geometry);
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        }

        for (_, decision) in decisions {
            // Convert the decision into the appropriate event to process
            let action = match &decision.kind {
                ActionDecisionKind::Action { action } => Some(action.clone()),

                ActionDecisionKind::Reaction { choice, .. } => {
                    if let Some(reaction) = choice {
                        Some(ActionData::from(reaction))
                    } else {
                        None
                    }
                }
            };

            if let Some(action) = action {
                self.validate_action(&action, true)?;
                self.process_event(Event::new(EventKind::ActionRequested { action }))?;
            }
        }

        // Check if any reactions are still pending, or if we can resume processing
        // the pending events
        let mut resume_pending_events = false;
        if let Some(encounter_id) = &encounter_id {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                if let Some(prompt) = encounter.next_pending_prompt() {
                    if let ActionPromptKind::Reactions { .. } = prompt.kind {
                        // There are more reactions to process, so just validate the
                        // prompt and return
                        encounter.validate_next_prompt(&self.world, &self.geometry);
                        return Ok(());
                    }
                    // No more reactions, continue with the pending events
                    // (if any)
                    resume_pending_events = true;
                }
            } else {
                panic!("Inconsistent state: entity is in combat but encounter not found");
            }
        }
        if resume_pending_events {
            while let Some(event) = self.pending_events.pop_front() {
                self.advance_event(event);
            }
        }
        Ok(())
    }

    pub fn validate_action(
        &mut self,
        action: &ActionData,
        // TODO: Could also be called simulate?
        spend_resources: bool,
    ) -> Result<(), ActionError> {
        let ActionData {
            actor,
            action_id,
            context: action_context,
            resource_cost,
            targets,
        } = action;

        systems::actions::action_usable_on_targets(
            &self.world,
            &self.geometry,
            *actor,
            action_id,
            action_context,
            resource_cost,
            targets,
        )
        .map_err(|error| ActionError::Usability(error))?;

        if spend_resources {
            systems::resources::spend(&mut self.world, *actor, resource_cost)
                .map_err(|error| ActionError::Resource(error))?;
        }

        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> Result<(), ActionError> {
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

                    for reactor in &encounter.participants(&self.world, EntityFilter::not_dead()) {
                        if self.event_log.has_reacted(&event.id, reactor) {
                            continue;
                        }

                        let reactions = systems::actions::available_reactions_to_event(
                            &self.world,
                            &self.geometry,
                            *reactor,
                            &event,
                        );

                        if !reactions.is_empty() {
                            // Record the available reactions for this reactor
                            reaction_options
                                .get_or_insert_with(HashMap::new)
                                .insert(*reactor, reactions);

                            self.event_log.record_reaction(event.id, *reactor);
                        }
                    }

                    if let Some(options) = reaction_options {
                        encounter.log_event(
                            Event::new(EventKind::ReactionTriggered {
                                trigger_event: event.clone().into(),
                                reactors: options.keys().cloned().collect(),
                            })
                            .as_response_to(event.id),
                        );

                        // Prompt all reactors for their reaction
                        encounter.queue_prompt(
                            ActionPrompt::new(ActionPromptKind::Reactions {
                                event: event.clone(),
                                options,
                            }),
                            true,
                        );

                        // Save the event being reacted to for later processing
                        self.pending_events.push_front(event);
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
    fn advance_event(&mut self, event: Event) {
        match &event.kind {
            EventKind::ActionRequested { action } => {
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
                                    event,
                                    resources_refunded,
                                } => {
                                    let actor = self.pending_events.iter().find_map(|e| {
                                        if e.id == event.id { e.actor() } else { None }
                                    });
                                    systems::resources::restore(
                                        &mut self.world,
                                        actor.unwrap(),
                                        resources_refunded,
                                    );
                                    self.pending_events.retain(|e| e.id != event.id);
                                }

                                // TODO: How to handle this properly?
                                ReactionResult::ModifyEvent { modification } => {
                                    (modification)(&mut self.pending_events.front_mut().unwrap())
                                }

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

    pub fn process_event_with_callback(
        &mut self,
        event: Event,
        callback: EventCallback,
    ) -> Result<(), ActionError> {
        self.add_event_listener(EventListener::new(event.id, callback));
        self.process_event(event)
    }
}

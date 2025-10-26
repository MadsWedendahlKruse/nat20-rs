use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::BufReader,
    path::Path,
    sync::Arc,
};

use hecs::{Entity, World};
use obj::Obj;
use parry3d::na::Point3;

use crate::{
    components::{
        actions::{
            action::{ActionKindResult, ReactionResult},
            targeting::{TargetType, TargetTypeInstance, TargetingError},
        },
        resource,
    },
    engine::{
        encounter::{Encounter, EncounterId, ParticipantsFilter},
        event::{
            ActionData, ActionDecision, ActionError, ActionPrompt, EncounterEvent, Event,
            EventCallback, EventId, EventKind, EventListener, EventLog, EventQueue,
        },
        geometry::{WorldGeometry, WorldPath},
    },
    systems::{
        self,
        actions::ActionUsabilityError,
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

    pub fn submit_decision(&mut self, decision: ActionDecision) -> Result<(), ActionError> {
        // Check that all of the actors involved in the decision are either:
        // 1. All in combat
        // 2. All out of combat
        let mut encounter_id = None;
        for actor in decision.actors() {
            let actor_encounter_id = self.in_combat.get(&actor);
            if let Some(id) = encounter_id {
                if actor_encounter_id != Some(id) {
                    panic!(
                        "All actors in a decision must be either in combat or out of combat together.\n
                        Decision actors: {:#?}\n
                        Actor encounter IDs: {:#?}",
                        decision.actors(),
                        decision.actors().iter().map(|a| self.in_combat.get(a)).collect::<Vec<_>>()
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
                self.validate_action(action, true)?;
                self.process_event(Event::new(EventKind::ActionRequested {
                    action: action.clone(),
                }))
            }

            ActionDecision::Reactions { event, choices } => {
                for (_, choice) in choices {
                    // Process the chosen reactions, if any
                    if let Some(reaction) = choice {
                        let action = ActionData::from(reaction);
                        self.validate_action(&action, true)?;
                        self.process_event(Event::new(EventKind::ActionRequested { action }))?;
                    }
                }
                // Check if the reaction triggered more reactions
                let mut resume_pending_events = false;
                if let Some(encounter_id) = self.in_combat.get(&event.actor().unwrap()) {
                    if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                        if let Some(prompt) = encounter.next_pending_prompt() {
                            if let ActionPrompt::Reactions { .. } = prompt {
                                // There are more reactions to process, so just return
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
        }
    }

    pub fn validate_action(
        &mut self,
        action: &ActionData,
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

        // ActionUsability::TargetingError(ref targeting_error) => match targeting_error {
        //     TargetingError::OutOfRange {
        //         target,
        //         distance,
        //         max_range,
        //     } => {
        //         // TODO: If out of range, attempt to find a path to get in range
        //         println!(
        //             "Target {:?} is out of range: distance {:?}, max range {:?}",
        //             target, distance, max_range
        //         );

        //         let target_position = match target {
        //             TargetTypeInstance::Entity(entity) => {
        //                 systems::geometry::get_position(&self.world, *entity).unwrap()
        //             }
        //             TargetTypeInstance::Point(point) => *point,
        //         };

        //         match systems::movement::path_in_range_of_point(
        //             self,
        //             *actor,
        //             target_position,
        //             *max_range,
        //             true,
        //             true,
        //             true,
        //             true,
        //         ) {
        //             Ok(path_result) => {
        //                 println!("Found path to get in range: {:?}", path_result.full_path);
        //                 // Validate the action again now that we're (potentially) in range
        //                 self.validate_action(action, spend_resources)
        //             }
        //             Err(movement_error) => {
        //                 println!("Failed to find path to get in range: {:?}", movement_error);
        //                 Err(ActionError::TargetingError(targeting_error.clone()))
        //             }
        //         }
        //     }

        //     // TODO: Proper error handling
        //     _ => panic!(
        //         "Action {:?} is not usable by entity {:?}: {:?}",
        //         action_id, actor, targeting_error
        //     ),
        // },
        // }
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

                    for reactor in &encounter.participants(
                        &self.world,
                        ParticipantsFilter::from(TargetType::entity_not_dead()),
                    ) {
                        if self.event_log.has_reacted(&event.id, reactor) {
                            continue;
                        }

                        let reactions = systems::actions::available_reactions_to_event(
                            &self.world,
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
                            ActionPrompt::Reactions {
                                event: event.clone(),
                                options,
                            },
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

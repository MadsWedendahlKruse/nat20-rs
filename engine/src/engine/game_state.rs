use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::BufReader,
    path::Path,
};

use hecs::{Entity, World};
use obj::Obj;
use parry3d::{na::Point3, shape::Ball};
use uom::si::f32::Length;

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
            EventKind, EventListener, EventLog, ReactionData,
        },
        geometry::WorldGeometry,
        interaction::{InteractionEngine, InteractionScopeId, InteractionSession},
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
    pub interaction_engine: InteractionEngine,
    pub event_log: EventLog,
    event_listeners: HashMap<EventId, EventListener>,
}

impl GameState {
    pub fn new(geometry: WorldGeometry) -> Self {
        Self {
            world: World::new(),
            encounters: HashMap::new(),
            in_combat: HashMap::new(),
            interaction_engine: InteractionEngine::default(),
            event_log: EventLog::new(),
            event_listeners: HashMap::new(),
            geometry,
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

        if let Some(session) = self.session_for_entity(entity) {
            if !session.pending_events().is_empty() {
                println!(
                    "Warning: ending turn for {:?} while there are pending events: {:#?}",
                    entity,
                    session.pending_events()
                );
            }
        }

        if let Some(encounter) = encounter {
            encounter.end_turn(self, entity);
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

    fn scope_for_entity(&self, entity: Entity) -> InteractionScopeId {
        if let Some(id) = self.in_combat.get(&entity) {
            InteractionScopeId::Encounter(*id)
        } else {
            InteractionScopeId::Global
        }
    }

    pub fn session_for_entity(&self, entity: Entity) -> Option<&InteractionSession> {
        let scope = self.scope_for_entity(entity);
        self.interaction_engine.session(scope)
    }

    pub fn next_prompt(&self, scope: InteractionScopeId) -> Option<&ActionPrompt> {
        self.interaction_engine
            .session(scope)
            .and_then(|s| s.next_prompt())
    }

    pub fn next_promt_encounter(&self, encounter_id: &EncounterId) -> Option<&ActionPrompt> {
        self.next_prompt(InteractionScopeId::Encounter(*encounter_id))
    }

    pub fn next_prompt_entity(&self, entity: Entity) -> Option<&ActionPrompt> {
        let scope = self.scope_for_entity(entity);
        self.next_prompt(scope)
    }

    pub fn submit_decision(&mut self, mut decision: ActionDecision) -> Result<(), ActionError> {
        let scope = match decision.kind {
            ActionDecisionKind::Action { ref action } => self.scope_for_entity(action.actor),
            ActionDecisionKind::Reaction { reactor, .. } => self.scope_for_entity(reactor),
        };

        // Avoid double mutable borrow
        let prompt_id = {
            let session = self.interaction_engine.session_mut(scope);

            // Ensure there is a prompt to respond to; lazily create one for Global.
            if session
                .pending_prompts()
                .iter()
                .all(|p| p.id != decision.response_to)
            {
                if matches!(scope, InteractionScopeId::Global) {
                    // “Open world” behavior: allow ad-hoc Action prompts.
                    session.queue_prompt(
                        ActionPrompt::new(ActionPromptKind::Action {
                            actor: decision.actor(),
                        }),
                        false,
                    );
                    decision.response_to = session.pending_prompts().back().unwrap().id;
                } else {
                    // In encounter scope, a missing prompt is a hard error.
                    return Err(ActionError::MissingPrompt {
                        decision: decision.clone(),
                        prompts: session.pending_prompts().iter().cloned().collect(),
                    });
                }
            }

            // Validate against the found prompt.
            let prompt = session
                .pending_prompts()
                .iter()
                .find(|p| p.id == decision.response_to)
                .expect("Prompt must exist at this point");
            prompt.is_valid_decision(&decision)?;
            let id = prompt.id;

            session.record_decision(decision);

            id
        };

        self.try_process_prompt(scope, prompt_id)
    }

    fn try_process_prompt(
        &mut self,
        scope: InteractionScopeId,
        prompt_id: ActionPromptId,
    ) -> Result<(), ActionError> {
        let (all_decisions_ready, prompt, decisions) = {
            let session = self.interaction_engine.session(scope);
            let prompt = session
                .and_then(|s| s.pending_prompts().iter().find(|p| p.id == prompt_id))
                .cloned()
                // .ok_or_else(|| anyhow::anyhow!("Prompt disappeared")) // or custom error
                .ok_or_else(|| panic!("Prompt disappeared"))
                .unwrap();

            let decisions_map = self
                .interaction_engine
                .session(scope)
                .unwrap()
                .decisions_for_prompt(&prompt_id)
                .cloned()
                .unwrap_or_default();

            let all_actors_submitted = prompt
                .actors()
                .iter()
                .all(|a| decisions_map.contains_key(a));
            (all_actors_submitted, prompt, decisions_map)
        };

        if !all_decisions_ready {
            return Ok(());
        }

        // Pop prompt & clear decisions
        {
            let session = self.interaction_engine.session_mut(scope);
            session.pop_prompt_by_id(&prompt_id);
        }

        // Convert decisions → events and validate/queue
        for (_actor, decision) in decisions {
            let maybe_action = match &decision.kind {
                ActionDecisionKind::Action { action } => Some(action.clone()),
                ActionDecisionKind::Reaction { choice, .. } => {
                    choice.as_ref().map(ActionData::from)
                }
            };

            if let Some(action) = maybe_action {
                self.validate_action(&action, true)?;
                self.process_event_scoped(
                    scope,
                    Event::new(EventKind::ActionRequested { action }),
                )?;
            }
        }

        // If we are in encounter scope, validate the next prompt (reactions may prune options)
        self.validate_or_refill_prompt_queue(scope);

        // If no reactions are pending, resume paused events.
        self.resume_pending_events_if_ready(scope);

        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> Result<(), ActionError> {
        if let Some(actor) = event.actor() {
            self.process_event_scoped(self.scope_for_entity(actor), event)
        } else {
            panic!("Cannot process event without actor: {:#?}", event);
        }
    }

    fn process_event_scoped(
        &mut self,
        scope: InteractionScopeId,
        event: Event,
    ) -> Result<(), ActionError> {
        self.log_event(event.clone());

        if let Some(event_id) = event.response_to {
            if let Some(listener) = self.event_listeners.get(&event_id) {
                if listener.matches(&event) {
                    let listener = self.event_listeners.remove(&event_id).unwrap();
                    listener.callback(self, &event);
                }
            }
        }

        // Reaction window
        if let Some(actor) = event.actor() {
            if let Some(reaction_options) = self.collect_reactions(actor, &event) {
                // Announce and prompt
                let session = self.interaction_engine.session_mut(scope);
                session.queue_prompt(
                    ActionPrompt::new(ActionPromptKind::Reactions {
                        event: event.clone(),
                        options: reaction_options,
                    }),
                    true,
                );
                session.queue_event(event, true);
                return Ok(());
            }
        }

        // No reaction window → advance now (same as your `advance_event`)
        self.advance_event(event);
        Ok(())
    }

    fn validate_or_refill_prompt_queue(&mut self, scope: InteractionScopeId) {
        let session = self.interaction_engine.session_mut(scope);

        if let Some(front) = session.next_prompt_mut() {
            let mut invalid = false;

            match &mut front.kind {
                ActionPromptKind::Reactions { event, options } => {
                    let mut new_options = HashMap::new();
                    for reactor in options.keys() {
                        let reactions = systems::actions::available_reactions_to_event(
                            &self.world,
                            &self.geometry,
                            *reactor,
                            event,
                        );
                        if !reactions.is_empty() {
                            new_options.insert(*reactor, reactions);
                        }
                    }
                    if new_options.is_empty() {
                        invalid = true;
                    }
                    *options = new_options;
                }
                ActionPromptKind::Action { .. } => { /* nothing */ }
            }

            if invalid {
                session.pop_prompt();
            }
        }

        match scope {
            InteractionScopeId::Global => { /* don't auto-refill */ }
            InteractionScopeId::Encounter(encounter_id) => {
                let encounter = self
                    .encounters
                    .get_mut(&encounter_id)
                    .expect("Inconsistent state: encounter not found");

                if session.pending_prompts().is_empty() {
                    session.queue_prompt(
                        ActionPrompt::new(ActionPromptKind::Action {
                            actor: encounter.current_entity(),
                        }),
                        false,
                    );
                }
            }
        }
    }

    fn resume_pending_events_if_ready(&mut self, scope: InteractionScopeId) {
        let session = self.interaction_engine.session_mut(scope);

        let ready_to_resume = {
            if session.pending_prompts().is_empty() {
                // Not sure this ever actually happens
                println!("[GameState] No pending prompts; ready to resume pending events.");
                true
            } else if let Some(front) = session.next_prompt()
                && !matches!(front.kind, ActionPromptKind::Reactions { .. })
            {
                println!(
                    "[GameState] Next prompt is not a reaction; ready to resume pending events."
                );
                true
            } else {
                false
            }
        };

        // TODO: Avoid clone
        let mut pending_events = session.pending_events().clone();
        if ready_to_resume {
            while let Some(event) = pending_events.pop_front() {
                self.advance_event(event);
            }
        }
    }

    fn collect_reactions(
        &self,
        actor: Entity,
        event: &Event,
    ) -> Option<HashMap<Entity, Vec<ReactionData>>> {
        // If in combat, only consider participants. Otherwise, consider all entities
        // that are nearby
        let reactors = if let Some(encounter_id) = self.in_combat.get(&actor)
            && let Some(encounter) = self.encounters.get(encounter_id)
        {
            encounter.participants(&self.world, EntityFilter::not_dead())
        } else if let Some((_, shape_pose)) = systems::geometry::get_shape(&self.world, actor) {
            systems::geometry::entities_in_shape(
                &self.world,
                // TODO: Not entirely sure what the right shape is here
                Box::new(Ball { radius: 100.0 }),
                &shape_pose,
            )
        } else {
            return None;
        };

        println!(
            "[GameState] Collecting reactions to event {:?} from reactors: {:?}",
            event.id, reactors
        );

        let mut reaction_options = HashMap::new();

        for reactor in &reactors {
            if self.event_log.has_reacted(&event.id, reactor) {
                continue;
            }

            let reactions = systems::actions::available_reactions_to_event(
                &self.world,
                &self.geometry,
                *reactor,
                event,
            );

            if !reactions.is_empty() {
                reaction_options.insert(*reactor, reactions);
            }
        }

        if reaction_options.is_empty() {
            println!("\tNo reaction options available for event {:?}", event.id);
            None
        } else {
            println!(
                "\tFound reactors for event {:?}: {:?}",
                event.id,
                reaction_options.keys()
            );
            Some(reaction_options)
        }
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
                            let session = self
                                .interaction_engine
                                .session_mut(self.scope_for_entity(action_result.performer.id()));

                            match reaction_result {
                                ReactionResult::CancelEvent {
                                    event,
                                    resources_refunded,
                                } => {
                                    if let Some(actor) =
                                        session.pending_events().iter().find_map(|e| {
                                            if e.id == event.id { e.actor() } else { None }
                                        })
                                    {
                                        systems::resources::restore(
                                            &mut self.world,
                                            actor,
                                            resources_refunded,
                                        );
                                        session.pending_events_mut().retain(|e| e.id != event.id);
                                    } else {
                                        panic!(
                                            "Attempted to cancel event which is not pending: {:#?}",
                                            event
                                        );
                                    }
                                }

                                // TODO: How to handle this properly?
                                ReactionResult::ModifyEvent { modification } => (modification)(
                                    &mut session.pending_events_mut().front_mut().unwrap(),
                                ),

                                ReactionResult::NoEffect => { /* Do nothing */ }
                            }
                        }

                        _ => {}
                    }
                }
            }

            EventKind::D20CheckPerformed(entity, kind, dc_kind) => {
                let _ = self.process_event_scoped(
                    self.scope_for_entity(*entity),
                    Event::new(EventKind::D20CheckResolved(
                        *entity,
                        kind.clone(),
                        dc_kind.clone(),
                    ))
                    .as_response_to(event.id),
                );
            }

            EventKind::DamageRollPerformed(entity, damage) => {
                let _ = self.process_event_scoped(
                    self.scope_for_entity(*entity),
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
        if let Some(actor) = event.actor() {
            self.add_event_listener(EventListener::new(event.id, callback));
            self.process_event_scoped(self.scope_for_entity(actor), event)
        } else {
            panic!(
                "Cannot process event with callback for event without actor: {:#?}",
                event
            );
        }
    }
}

use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::{ActionContext, ActionResult, ReactionKind},
        d20::{D20CheckDC, D20CheckResult},
        health::life_state::LifeState,
        id::{ActionId, EncounterId},
        resource::ResourceCostMap,
        saving_throw::SavingThrowKind,
        skill::Skill,
    },
    engine::encounter::{ActionDecision, ActionError, Encounter, ParticipantsFilter},
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
    pub context: ActionContext,
    pub resource_cost: ResourceCostMap,
    pub kind: ReactionKind,
}

#[derive(Debug, Clone)]
// TODO: Not 100% sure this is the best solution
pub enum GameEvent {
    EncounterStarted(EncounterId),
    EncounterEnded(EncounterId, EventLog),
    NewRound(EncounterId, usize),
    /// The action was successfully performed, and the results are applied to the targets.
    ActionPerformed {
        action: ActionData,
        results: Vec<ActionResult>,
    },
    ReactionTriggered {
        reactor: Entity,
        action: ActionData,
    },
    NoReactionTaken {
        reactor: Entity,
        action: ActionData,
    },
    ActionCancelled {
        reactor: Entity,
        reaction: ReactionData,
        action: ActionData,
    },
    SavingThrow(Entity, D20CheckResult, D20CheckDC<SavingThrowKind>),
    SkillCheck(Entity, D20CheckResult, D20CheckDC<Skill>),
    LifeStateChanged {
        entity: Entity,
        new_state: LifeState,
        /// The entity that caused the change, if any
        actor: Option<Entity>,
    },
}

pub type EventLog = Vec<GameEvent>;

// TODO: WorldState instead?
pub struct GameState {
    pub world: World,
    pub encounters: HashMap<EncounterId, Encounter>,
    pub in_combat: HashMap<Entity, EncounterId>,
    pub event_log: EventLog,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            encounters: HashMap::new(),
            in_combat: HashMap::new(),
            event_log: Vec::new(),
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
        let encounter = Encounter::new(&mut self.world, participants, encounter_id.clone());
        self.encounters.insert(encounter_id.clone(), encounter);
        self.event_log
            .push(GameEvent::EncounterStarted(encounter_id.clone()));
        encounter_id
    }

    pub fn start_encounter(&mut self, participants: HashSet<Entity>) -> EncounterId {
        let encounter_id = EncounterId::new_v4();
        self.start_encounter_with_id(participants, encounter_id.clone());
        encounter_id
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
            self.event_log.push(GameEvent::EncounterEnded(
                encounter_id.clone(),
                encounter.combat_log_move(),
            ));
        }
    }

    pub fn process(&mut self, decision: ActionDecision) -> Result<GameEvent, ActionError> {
        let entity = decision.actor();
        if let Some(encounter_id) = self.in_combat.get(&entity) {
            if let Some(encounter) = self.encounters.get_mut(encounter_id) {
                return encounter.process(&mut self.world, decision);
            }
            todo!("Handle missing encounter");
        } else {
            todo!("Handle entity not in combat");
        }
    }

    // TODO: Not sure if it should be possible to log events from outside of the game state
    pub fn log_event(&mut self, event: GameEvent) {
        self.event_log.push(event);
    }
}

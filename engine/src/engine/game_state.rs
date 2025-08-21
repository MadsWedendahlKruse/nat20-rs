use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};

use crate::{
    components::{
        ability::Ability,
        d20_check::{D20CheckDC, D20CheckResult},
        id::EncounterId,
        skill::Skill,
    },
    engine::encounter::{ActionDecision, ActionError, CombatEvents, CombatLog, Encounter},
};

// TODO: Not 100% sure this is the best solution
pub enum GameEvent {
    EncounterStarted(EncounterId),
    EncounterEnded(EncounterId, CombatLog),
    SavingThrow(Entity, D20CheckResult, D20CheckDC<Ability>),
    SkillCheck(Entity, D20CheckResult, D20CheckDC<Skill>),
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
        if let Some(encounter) = self.encounters.remove(encounter_id) {
            for entity in encounter.participants() {
                self.in_combat.remove(&entity);
            }
            self.event_log.push(GameEvent::EncounterEnded(
                encounter_id.clone(),
                encounter.combat_log,
            ));
        }
    }

    pub fn process(&mut self, decision: ActionDecision) -> Result<CombatEvents, ActionError> {
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

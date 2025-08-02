use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::ActionContext,
        id::{ActionId, EncounterId},
    },
    engine::encounter::{ActionDecision, ActionDecisionResult, ActionError, Encounter},
};

// pub enum GameEvent {
//     ActionRequested {
//         /// The entity that is requesting the action
//         entity: Entity,
//         /// The ID of the action being requested
//         action_id: ActionId,
//         /// The context for the action, e.g. spell level
//         context: ActionContext,
//     },
//     ReactionRequested(ReactionRequest),   // A reaction might be triggered
//     ActionCancelled(ActionId),            // Something (like counterspell) cancelled an action
//     ActionExecuted(ResolvedAction),       // The action is actually performed
//     // Maybe also: ResourceSpent, DamageDealt, etc.
// }

pub struct GameState {
    pub world: World,
    pub encounters: HashMap<EncounterId, Encounter>,
    pub in_combat: HashMap<Entity, EncounterId>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            encounters: HashMap::new(),
            in_combat: HashMap::new(),
        }
    }

    pub fn start_encounter_with_id(
        &mut self,
        participants: HashSet<Entity>,
        encounter_id: EncounterId,
    ) -> EncounterId {
        let encounter = Encounter::new(&mut self.world, participants, encounter_id.clone());
        self.encounters.insert(encounter_id.clone(), encounter);
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

    pub fn process(
        &mut self,
        decision: ActionDecision,
    ) -> Result<ActionDecisionResult, ActionError> {
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
}

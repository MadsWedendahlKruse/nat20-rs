use std::collections::HashSet;

use hecs::{Entity, World};

use crate::{
    components::{
        actions::action::{ActionContext, ActionResult},
        d20_check::D20CheckResult,
        id::ActionId,
        skill::{Skill, SkillSet},
    },
    systems,
};

#[derive(Debug, PartialEq, Eq)]
pub enum EncounterState {
    AwaitingAction,
    ResolvingAction,
    TurnTransition,
    CombatEnded,
}

pub struct Encounter {
    pub participants: HashSet<Entity>,
    pub round: usize,
    pub turn_index: usize,
    pub initiative_order: Vec<(Entity, D20CheckResult)>,
    pub state: EncounterState,
}

impl Encounter {
    pub fn new(world: &mut World, participants: &[Entity]) -> Self {
        let mut engine = Self {
            participants: participants.iter().cloned().collect(),
            round: 1,
            turn_index: 0,
            initiative_order: Vec::new(),
            state: EncounterState::TurnTransition,
        };
        engine.roll_initiative(world);
        engine.start_turn(world);
        engine
    }

    fn roll_initiative(&mut self, world: &World) {
        let mut indexed_rolls: Vec<(Entity, D20CheckResult)> = self
            .participants
            .iter()
            .map(|entity| {
                let roll = systems::helpers::get_component::<SkillSet>(world, *entity).check(
                    Skill::Initiative,
                    world,
                    *entity,
                );
                (entity.clone(), roll)
            })
            .collect();

        indexed_rolls.sort_by_key(|(_, roll)| -(roll.total as i32));
        self.initiative_order = indexed_rolls
            .into_iter()
            .map(|(i, roll)| (i, roll))
            .collect();
    }

    pub fn initiative_order(&self) -> &Vec<(Entity, D20CheckResult)> {
        &self.initiative_order
    }

    pub fn current_entity(&self) -> Entity {
        let (idx, _) = self.initiative_order[self.turn_index];
        idx
    }

    pub fn participants(&self) -> &HashSet<Entity> {
        &self.participants
    }

    // pub fn submit_action(
    //     &mut self,
    //     action_id: &ActionId,
    //     action_context: &ActionContext,
    //     // TODO: Targets have to determined before submitting the action
    //     // e.g. for Fireball, the targets are determined by the asking the engine
    //     // which characters are within the area of effect
    //     targets: Vec<Entity>,
    // ) -> Result<Vec<ActionResult>, String> {
    //     if self.state != EncounterState::AwaitingAction {
    //         return Err("Engine is not ready for an action submission".into());
    //     }

    //     self.state = EncounterState::ResolvingAction;

    //     // TODO: validate actions, e.g. for a melee weapon attack, the character must be adjacent to the target
    //     // TODO: validate that character has enough resources to perform the action
    //     // TEMP: Assume action is valid (unwrap)

    //     let snapshots =
    //         self.current_character_mut()
    //             .perform_action(action_id, action_context, targets.len());

    //     let results: Vec<_> = targets
    //         .into_iter()
    //         .zip(snapshots)
    //         .map(|(target_id, action_snapshot)| {
    //             let target = self
    //                 .participants
    //                 .get_mut(&target_id)
    //                 .expect("Target character not found in participants");
    //             action_snapshot.apply_to_character(target)
    //         })
    //         .collect();

    //     // For now we just assume the action is resolved
    //     self.state = EncounterState::AwaitingAction;
    //     Ok(results)
    // }

    pub fn end_turn(&mut self, world: &mut World) {
        if self.state != EncounterState::AwaitingAction {
            return;
        }

        self.turn_index = (self.turn_index + 1) % self.participants.len();
        if self.turn_index == 0 {
            self.round += 1;
        }

        self.state = EncounterState::TurnTransition;
        self.start_turn(world);
    }

    fn start_turn(&mut self, world: &mut World) {
        systems::turns::on_turn_start(world, self.current_entity());
        self.state = EncounterState::AwaitingAction;
    }

    pub fn round(&self) -> usize {
        self.round
    }
}

// impl ActionProvider for Encounter<'_> {
//     fn all_actions(&self) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
//         self.current_character().all_actions()
//     }

//     fn available_actions(
//         &self,
//     ) -> HashMap<ActionId, (Vec<ActionContext>, HashMap<ResourceId, u8>)> {
//         self.current_character().available_actions()
//     }
// }

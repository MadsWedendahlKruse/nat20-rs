use std::collections::HashMap;

use crate::actions::action::{Action, ActionContext, ActionProvider, ActionResult};
use crate::creature::character::Character;
use crate::engine;
use crate::resources::resources::ResourceError;
use crate::stats::d20_check::D20CheckResult;
use crate::stats::skill::Skill;
use crate::utils::id::CharacterId;

#[derive(Debug, PartialEq, Eq)]
pub enum CombatState {
    AwaitingAction,
    ResolvingAction,
    TurnTransition,
    CombatEnded,
}

pub struct CombatEngine<'c> {
    pub participants: HashMap<CharacterId, &'c mut Character>,
    pub round: usize,
    pub turn_index: usize,
    pub initiative_order: Vec<(CharacterId, D20CheckResult)>,
    pub state: CombatState,
}

impl<'c> CombatEngine<'c> {
    pub fn new(participants: Vec<&'c mut Character>) -> Self {
        let mut engine = Self {
            participants: participants.into_iter().map(|p| (p.id(), p)).collect(),
            round: 1,
            turn_index: 0,
            initiative_order: Vec::new(),
            state: CombatState::TurnTransition,
        };
        engine.roll_initiative();
        engine.start_turn();
        engine
    }

    fn roll_initiative(&mut self) {
        let mut indexed_rolls: Vec<(CharacterId, D20CheckResult)> = self
            .participants
            .iter_mut()
            .map(|(uuid, character)| {
                let roll = character.skill_check(Skill::Initiative);
                (uuid.clone(), roll)
            })
            .collect();

        indexed_rolls.sort_by_key(|(_, roll)| -(roll.total as i32));
        self.initiative_order = indexed_rolls
            .into_iter()
            .map(|(i, roll)| (i, roll))
            .collect();
    }

    pub fn initiative_order(&self) -> &Vec<(CharacterId, D20CheckResult)> {
        &self.initiative_order
    }

    pub fn current_character_id(&self) -> CharacterId {
        let (idx, _) = self.initiative_order[self.turn_index];
        idx
    }

    pub fn current_character(&self) -> &Character {
        self.participants.get(&self.current_character_id()).unwrap()
    }

    pub fn current_character_mut(&mut self) -> &mut Character {
        self.participants
            .get_mut(&self.current_character_id())
            .unwrap()
    }

    pub fn available_actions(&self) -> Vec<(&Action, ActionContext)> {
        self.current_character().available_actions()
    }

    pub fn participant(&self, id: &CharacterId) -> Option<&Character> {
        self.participants.get(id).map(|c| &**c)
    }

    pub fn participants(&self) -> Vec<&Character> {
        self.participants.values().map(|c| &**c).collect()
    }

    pub fn submit_action(
        &mut self,
        action: &Action,
        action_context: &ActionContext,
        // TODO: Targets have to determined before submitting the action
        // e.g. for Fireball, the targets are determined by the asking the engine
        // which characters are within the area of effect
        targets: Vec<CharacterId>,
    ) -> Result<Vec<ActionResult>, String> {
        if self.state != CombatState::AwaitingAction {
            return Err("Engine is not ready for an action submission".into());
        }

        self.state = CombatState::ResolvingAction;

        // TODO: validate actions, e.g. for a melee weapon attack, the character must be adjacent to the target

        // TODO: Assume action is valid
        // TODO: Resrouce might error? Should probably check resources as part of the validation
        // Spend resources, e.g. action points, spell slots, etc.
        let snapshots = action.perform(self.current_character_mut(), action_context, targets.len());

        let results: Vec<_> = targets
            .into_iter()
            .zip(snapshots)
            .map(|(target_id, action_snapshot)| {
                let target = self
                    .participants
                    .get_mut(&target_id)
                    .expect("Target character not found in participants");
                action_snapshot.apply_to_character(target)
            })
            .collect();

        // For now we just assume the action is resolved
        self.state = CombatState::AwaitingAction;
        Ok(results)
    }

    fn resolve_action(
        &mut self,
        action: &Action,
        action_context: &ActionContext,
        target_id: CharacterId,
    ) -> ActionResult {
        let character = self.current_character();
        let action_snapshot = action.snapshot(character, action_context);
        let target = self
            .participants
            .get_mut(&target_id)
            .expect("Target character not found in participants");
        action_snapshot.apply_to_character(target)
    }

    pub fn end_turn(&mut self) {
        if self.state != CombatState::AwaitingAction {
            return;
        }

        self.turn_index = (self.turn_index + 1) % self.participants.len();
        if self.turn_index == 0 {
            self.round += 1;
        }

        self.state = CombatState::TurnTransition;
        self.start_turn();
    }

    fn start_turn(&mut self) {
        self.state = CombatState::AwaitingAction;
        // TODO: run turn start effects
    }
}

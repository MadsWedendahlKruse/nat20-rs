use std::collections::HashMap;

use crate::combat::action::{CombatAction, CombatActionRequest, CombatActionResult};
use crate::creature::character::Character;
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
                let roll = character.skills().check(Skill::Initiative, character);
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

    pub fn available_actions(&self) -> Vec<CombatAction> {
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
        action: CombatActionRequest,
    ) -> Result<CombatActionResult, String> {
        if self.state != CombatState::AwaitingAction {
            return Err("Engine is not ready for an action submission".into());
        }

        self.state = CombatState::ResolvingAction;
        // TODO: validate and resolve action
        // This is where you'd match on the action type and apply its logic
        let result = self.resolve_action(action);

        // For now we just assume the action is resolved
        self.state = CombatState::AwaitingAction;
        Ok(result)
    }

    fn resolve_action(&mut self, action: CombatActionRequest) -> CombatActionResult {
        match action {
            CombatActionRequest::WeaponAttack {
                weapon_type,
                hand,
                target,
            } => {
                // Resolve the weapon attack action
                let attacker = self.current_character();
                let attack_roll_result =
                    attacker
                        .loadout()
                        .attack_roll(&attacker, &weapon_type, hand);
                let weapon = attacker
                    .loadout()
                    .weapon_in_hand(&weapon_type, hand)
                    .unwrap();
                let damage_roll_result = weapon
                    .damage_roll(&attacker, hand)
                    // TODO: What if the target can't be critically hit?
                    .roll_crit(attack_roll_result.is_crit);

                let target_character = self.participants.get_mut(&target).unwrap();
                let armor_class = target_character.loadout().armor_class(&target_character);

                let attack_hit = target_character
                    .loadout()
                    .does_attack_hit(&target_character, &attack_roll_result);

                let damage_result = if attack_hit {
                    Some(target_character.take_damage(&damage_roll_result))
                } else {
                    None
                };

                CombatActionResult::WeaponAttack {
                    target: target_character.id(),
                    target_armor_class: armor_class,
                    attack_roll_result,
                    damage_roll_result,
                    damage_result,
                }
            }

            CombatActionRequest::UseItem { item_name, target } => todo!(),
            CombatActionRequest::Dodge => todo!(),
            CombatActionRequest::Disengage => todo!(),
            CombatActionRequest::Help { target } => todo!(),
            CombatActionRequest::EndTurn => todo!(),
        }
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

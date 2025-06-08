use std::collections::HashMap;

use crate::combat::action::{
    CombatAction, CombatActionProvider, CombatActionRequest, CombatActionResult,
};
use crate::combat::damage::DamageEventResult;
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

        // TODO: validate actions, e.g. for a melee weapon attack, the character must be adjacent to the target

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
                let attacker = self.current_character();
                let attack_roll_result = attacker.attack_roll(&weapon_type, hand);
                let damage_roll_result = attacker
                    .damage_roll(&weapon_type, hand)
                    // TODO: What if the target can't be critically hit?
                    .roll_crit_damage(attack_roll_result.roll_result.is_crit);

                let target_character = self.participants.get_mut(&target).unwrap();
                let armor_class = target_character.armor_class();

                let damage_source = DamageEventResult::WeaponAttack(
                    attack_roll_result.clone(),
                    damage_roll_result.clone(),
                );
                let damage_result = target_character.take_damage(&damage_source);

                CombatActionResult::WeaponAttack {
                    target: target_character.id(),
                    target_armor_class: armor_class,
                    attack_roll_result,
                    damage_roll_result,
                    damage_result,
                }
            }

            CombatActionRequest::CastSpell {
                spell_id,
                level,
                targets,
            } => {
                let caster = self.current_character();
                // TODO: What if the caster doesn't have the spell?
                let spell_snapshot = caster.spell_snapshot(&spell_id, level).unwrap().unwrap();

                let mut spell_results = Vec::new();
                for target in targets {
                    let target_character = self.participants.get_mut(&target).unwrap();
                    let spell_result = spell_snapshot.cast(target_character);
                    spell_results.push(spell_result);
                }

                // TODO: When/where do we consume the spell slot?

                CombatActionResult::CastSpell {
                    result: spell_results,
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

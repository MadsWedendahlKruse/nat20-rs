use crate::{
    item::equipment::{equipment::HandSlot, weapon::WeaponType},
    stats::d20_check::D20CheckResult,
    utils::id::CharacterId,
};

use super::damage::DamageMitigationResult;

#[derive(Debug, Clone, PartialEq)]
pub enum CombatAction {
    // TODO: Unarmed attack
    WeaponAttack {
        weapon_type: WeaponType,
        hand: HandSlot,
    },
    UseItem {
        item_name: String,
    },
    Dodge,
    Disengage,
    Help,
    CastSpell {
        spell_name: String,
    },
    EndTurn,
}

pub enum TargetType {
    SelfTarget,
    Single,
    Multiple(usize),
}

impl TargetType {
    pub fn target_count(&self) -> usize {
        match self {
            TargetType::SelfTarget => 1,
            TargetType::Single => 1,
            TargetType::Multiple(count) => *count,
        }
    }
}

impl CombatAction {
    pub fn request_with_targets(&self, targets: Vec<CharacterId>) -> Option<CombatActionRequest> {
        match self {
            CombatAction::WeaponAttack { weapon_type, hand } if targets.len() == 1 => {
                Some(CombatActionRequest::WeaponAttack {
                    weapon_type: weapon_type.clone(),
                    hand: *hand,
                    target: targets[0],
                })
            }
            CombatAction::Help if targets.len() == 1 => {
                Some(CombatActionRequest::Help { target: targets[0] })
            }
            CombatAction::UseItem { item_name } => Some(CombatActionRequest::UseItem {
                item_name: item_name.clone(),
                target: targets.get(0).copied(),
            }),
            CombatAction::Dodge => Some(CombatActionRequest::Dodge),
            CombatAction::Disengage => Some(CombatActionRequest::Disengage),
            CombatAction::EndTurn => Some(CombatActionRequest::EndTurn),
            _ => None, // fallback: not enough or too many targets
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatActionRequest {
    WeaponAttack {
        weapon_type: WeaponType,
        hand: HandSlot,
        target: CharacterId,
    },
    // CastSpell {
    //     spell: Spell,
    //     targets: Vec<CharacterId>,
    // },
    UseItem {
        item_name: String,
        target: Option<CharacterId>,
    },
    Dodge,
    Disengage,
    Help {
        target: CharacterId,
    },
    EndTurn,
}

#[derive(Debug)]
pub enum CombatActionResult {
    WeaponAttack {
        target: CharacterId,
        attack_roll_result: D20CheckResult,
        damage_result: Option<DamageMitigationResult>,
    },
    UseItem {
        target: Option<CharacterId>,
        effect: String,
    },
    Help {
        assisted: CharacterId,
    },
    Dodge,
    Disengage,
    EndTurn,
}

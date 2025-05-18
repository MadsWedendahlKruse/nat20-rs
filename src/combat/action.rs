use std::fmt;

use crate::{
    item::equipment::{equipment::HandSlot, weapon::WeaponType},
    stats::{d20_check::D20CheckResult, modifier::ModifierSet},
    utils::id::CharacterId,
};

use super::damage::{DamageMitigationResult, DamageRollResult};

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
        target_armor_class: ModifierSet,
        attack_roll_result: D20CheckResult,
        damage_roll_result: DamageRollResult,
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

impl fmt::Display for CombatActionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CombatActionResult::WeaponAttack {
                target,
                target_armor_class,
                attack_roll_result,
                damage_roll_result,
                damage_result,
            } => write!(
                f,
                "Target: {}\nTarget Armor Class: {} = {}\nAttack Roll: {}\nDamage Roll: {}\nDamage Result: {}",
                target,
                target_armor_class,
                target_armor_class.total(),
                attack_roll_result,
                damage_roll_result,
                if let Some(result) = damage_result {
                    format!("{}", result)
                } else {
                    "Miss".to_string()
                }
            ),
            CombatActionResult::UseItem { target, effect } => {
                write!(f, "Use Item on {:?}: Effect: {}", target, effect)
            }
            CombatActionResult::Help { assisted } => {
                write!(f, "Help action on {}", assisted)
            }
            CombatActionResult::Dodge => write!(f, "Dodge action"),
            CombatActionResult::Disengage => write!(f, "Disengage action"),
            CombatActionResult::EndTurn => write!(f, "End Turn action"),
        }
    }
}

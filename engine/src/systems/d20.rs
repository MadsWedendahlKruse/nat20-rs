use std::sync::Arc;

use hecs::Entity;

use crate::{
    components::{
        d20::{D20CheckDC, D20CheckResult},
        damage::AttackRollResult,
        items::equipment::{armor::ArmorClass, slots::EquipmentSlot},
        saving_throw::{SavingThrowKind, SavingThrowSet},
        skill::{Skill, SkillSet},
    },
    engine::{
        event::{Event, EventId, EventKind},
        game_state::GameState,
    },
    systems,
};

// TODO: Do we even need it without the DC? Does that make sense?
#[derive(Debug, Clone)]
pub enum D20CheckKind {
    SavingThrow(SavingThrowKind),
    Skill(Skill),
    AttackRoll(EquipmentSlot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum D20CheckDCKind {
    SavingThrow(D20CheckDC<SavingThrowKind>),
    Skill(D20CheckDC<Skill>),
    AttackRoll(EquipmentSlot, Entity, ArmorClass),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum D20ResultKind {
    SavingThrow {
        kind: SavingThrowKind,
        result: D20CheckResult,
    },
    Skill {
        skill: Skill,
        result: D20CheckResult,
    },
    AttackRoll {
        result: AttackRollResult,
    },
}

impl D20ResultKind {
    pub fn is_success(&self, dc: &D20CheckDCKind) -> bool {
        match (self, dc) {
            (D20ResultKind::SavingThrow { result, .. }, D20CheckDCKind::SavingThrow(dc)) => {
                result.is_success(dc)
            }
            (D20ResultKind::Skill { result, .. }, D20CheckDCKind::Skill(dc)) => {
                result.is_success(dc)
            }
            (
                D20ResultKind::AttackRoll { result },
                D20CheckDCKind::AttackRoll(_, _, armor_class),
            ) => {
                let result = &result.roll_result;
                !result.is_crit_fail
                    && (result.is_crit || result.total >= armor_class.total() as u32)
            }
            _ => false,
        }
    }

    pub fn d20_result(&self) -> &D20CheckResult {
        match self {
            D20ResultKind::SavingThrow { result, .. } => result,
            D20ResultKind::Skill { result, .. } => result,
            D20ResultKind::AttackRoll { result } => &result.roll_result,
        }
    }
}

#[must_use]
pub fn check(game_state: &mut GameState, entity: Entity, dc: &D20CheckDCKind) -> EventId {
    let world = &game_state.world;
    let result = match dc {
        D20CheckDCKind::SavingThrow(dc) => D20ResultKind::SavingThrow {
            kind: dc.key,
            result: systems::helpers::get_component::<SavingThrowSet>(world, entity)
                .check_dc(dc, world, entity),
        },
        D20CheckDCKind::Skill(dc) => D20ResultKind::Skill {
            skill: dc.key,
            result: systems::helpers::get_component::<SkillSet>(world, entity)
                .check_dc(dc, world, entity),
        },
        // D20CheckDCKind::AttackRoll(slot, target, armor_class) => D20ResultKind::AttackRoll {
        //     result: systems::combat::attack_roll_against_target(world, entity, slot, target),
        // },
        D20CheckDCKind::AttackRoll(_, _, _) => {
            todo!("systems::d20 attack roll checks are not yet implemented");
        }
    };
    process_event(game_state, entity, result, Some(dc.clone()))
}

fn process_event(
    game_state: &mut GameState,
    entity: Entity,
    result: D20ResultKind,
    dc: Option<D20CheckDCKind>,
) -> EventId {
    let event = Event::new(EventKind::D20CheckPerformed(entity, result.clone(), dc));
    let event_id = event.id;
    game_state.process_event(event);
    event_id
}

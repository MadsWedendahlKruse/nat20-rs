use std::{fmt::Debug, sync::Arc};

use crate::{
    combat::damage::{AttackRoll, AttackRollResult, DamageRoll, DamageRollResult},
    creature::character::Character,
    effects::hooks::{
        ArmorClassHook, AttackRollHook, AttackRollResultHook, DamageRollHook, DamageRollResultHook,
    },
    stats::modifier::{ModifierSet, ModifierSource},
    utils::id::EffectId,
};

use super::hooks::{EffectHook, SavingThrowHook, SkillCheckHook};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDuration {
    Instant,
    Temporary(usize),
    Persistent,
}

#[derive(Clone)]
pub struct Effect {
    pub id: EffectId,
    source: ModifierSource,
    duration: EffectDuration,
    // TODO: description?
    pub on_apply: EffectHook,
    // on_turn_start: EffectHook,
    // TODO: Do we need to differentiate between when an effect explicitly expires and when
    // the effect is removed from the character?
    // pub on_expire: EffectHook,
    pub on_unapply: EffectHook,
    // These use Option because they need a key for the skill or saving throw, which
    // we don't have when constructing the effect.
    pub on_skill_check: Option<SkillCheckHook>,
    pub on_saving_throw: Option<SavingThrowHook>,
    pub pre_attack_roll: AttackRollHook,
    pub post_attack_roll: AttackRollResultHook,
    pub on_armor_class: ArmorClassHook,
    pub pre_damage_roll: DamageRollHook,
    pub post_damage_roll: DamageRollResultHook,
}

impl Effect {
    pub fn new(id: EffectId, source: ModifierSource, duration: EffectDuration) -> Self {
        let noop = Arc::new(|_: &mut Character| {}) as EffectHook;

        Self {
            id,
            source,
            duration,
            on_apply: noop.clone(),
            on_unapply: noop.clone(),
            on_skill_check: None,
            on_saving_throw: None,
            pre_attack_roll: Arc::new(|_: &Character, _: &mut AttackRoll| {}) as AttackRollHook,
            post_attack_roll: Arc::new(|_: &Character, _: &mut AttackRollResult| {})
                as AttackRollResultHook,
            on_armor_class: Arc::new(|_: &Character, _: &mut ModifierSet| {}) as ArmorClassHook,
            pre_damage_roll: Arc::new(|_: &Character, _: &mut DamageRoll| {}) as DamageRollHook,
            post_damage_roll: Arc::new(|_: &Character, _: &mut DamageRollResult| {})
                as DamageRollResultHook,
        }
    }

    pub fn is_expired(&self, turn: usize) -> bool {
        match self.duration {
            EffectDuration::Instant => true,
            EffectDuration::Temporary(duration) => turn >= duration,
            EffectDuration::Persistent => false,
        }
    }
}

impl Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effect")
            .field("id", &self.id)
            .field("source", &self.source)
            .field("duration", &self.duration)
            .finish()
    }
}

impl PartialEq for Effect {
    // TODO: Might have to implement a more complex way to identify effects
    // What if an item has multiple effects?
    // Compare memory addresses of functions? Don't know if that's possible
    // or even a good idea.
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.duration == other.duration
    }
}

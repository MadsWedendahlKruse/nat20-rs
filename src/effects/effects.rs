use std::{fmt::Debug, sync::Arc};

use crate::{
    creature::character::Character,
    stats::{
        d20_check::{D20Check, D20CheckResult},
        modifier::ModifierSource,
    },
};

use super::hooks::{D20CheckHook, D20CheckResultHook, EffectHook, SavingThrowHook, SkillCheckHook};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDuration {
    Instant,
    Temporary(usize),
    Persistent,
}

#[derive(Clone)]
pub struct Effect {
    source: ModifierSource,
    duration: EffectDuration,
    // TODO: description?
    pub on_apply: EffectHook,
    // on_turn_start: EffectHook,
    // TODO: Do we need to differentiate between when an effect explicitly expires and when
    // the effect is removed from the character?
    // pub on_expire: EffectHook,
    pub on_unapply: EffectHook,
    pub skill_check_hook: Option<SkillCheckHook>,
    pub saving_throw_hook: Option<SavingThrowHook>,
    pub pre_attack_roll: D20CheckHook,
    pub post_attack_roll: D20CheckResultHook,
}

impl Effect {
    pub fn new(source: ModifierSource, duration: EffectDuration) -> Self {
        let noop = Arc::new(|_: &mut Character| {}) as EffectHook;
        let noop_d20 = Arc::new(|_: &Character, _: &mut D20Check| {}) as D20CheckHook;
        let noop_d20_result =
            Arc::new(|_: &Character, _: &mut D20CheckResult| {}) as D20CheckResultHook;

        Self {
            source,
            duration,
            on_apply: noop.clone(),
            on_unapply: noop.clone(),
            skill_check_hook: None,
            saving_throw_hook: None,
            pre_attack_roll: noop_d20.clone(),
            post_attack_roll: noop_d20_result.clone(),
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
        write!(
            f,
            "Effect {{ source: {:?}, duration: {:?} }}",
            self.source, self.duration
        )
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

use std::{fmt::Debug, sync::Arc};

use crate::{creature::character::Character, stats::modifier::ModifierSource};

use super::hooks::{AttackPostHook, AttackPreHook, EffectHook};

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
    on_apply: EffectHook,
    pub pre_attack_roll: AttackPreHook,
    pub post_attack_roll: AttackPostHook,
    on_turn_start: EffectHook,
    on_expire: EffectHook,
}

impl Effect {
    pub fn new(source: ModifierSource, duration: EffectDuration) -> Self {
        let noop = Arc::new(|_: &mut Character| {}) as EffectHook;

        Self {
            source,
            duration,
            on_apply: noop.clone(),
            pre_attack_roll: Arc::new(|_, _| {}),
            post_attack_roll: Arc::new(|_, _| {}),
            on_turn_start: noop.clone(),
            on_expire: noop,
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

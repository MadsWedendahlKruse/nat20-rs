use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

use hecs::{Entity, World};

use crate::components::{
    actions::action::{Action, ActionContext},
    damage::{AttackRoll, AttackRollResult, DamageRoll, DamageRollResult},
    effects::hooks::{
        ActionHook, ArmorClassHook, AttackRollHook, AttackRollResultHook, D20CheckHooks,
        DamageRollHook, DamageRollResultHook, ResourceCostHook,
    },
    id::EffectId,
    items::equipment::armor::ArmorClass,
    modifier::ModifierSource,
    resource::ResourceAmountMap,
    saving_throw::SavingThrowKind,
    skill::Skill,
};

use super::hooks::EffectHook;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectDuration {
    Instant,
    Temporary {
        /// Number of turns the effect lasts
        duration: u8,
        /// Number of turns that have passed since the effect was applied
        turns_elapsed: u8,
    },
    Conditional,
    Permanent,
}

impl EffectDuration {
    pub fn temporary(duration: u8) -> Self {
        Self::Temporary {
            duration,
            turns_elapsed: 0,
        }
    }
}

impl Display for EffectDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectDuration::Instant => write!(f, "Instant"),
            EffectDuration::Permanent => write!(f, "Persistent"),
            EffectDuration::Temporary {
                duration,
                turns_elapsed,
            } => {
                write!(
                    f,
                    "Temporary ({} turns, {} elapsed)",
                    duration, turns_elapsed
                )
            }
            EffectDuration::Conditional => write!(f, "Conditional"),
        }
    }
}

#[derive(Clone)]
pub struct Effect {
    id: EffectId,
    source: ModifierSource,
    duration: EffectDuration,
    // TODO: description?
    pub on_apply: EffectHook,
    // on_turn_start: EffectHook,
    // TODO: Do we need to differentiate between when an effect explicitly expires and when
    // the effect is removed from the character?
    // pub on_expire: EffectHook,
    pub on_unapply: EffectHook,
    pub on_skill_check: HashMap<Skill, D20CheckHooks>,
    pub on_saving_throw: HashMap<SavingThrowKind, D20CheckHooks>,
    pub pre_attack_roll: AttackRollHook,
    pub post_attack_roll: AttackRollResultHook,
    pub on_armor_class: ArmorClassHook,
    pub pre_damage_roll: DamageRollHook,
    pub post_damage_roll: DamageRollResultHook,
    pub on_action: ActionHook,
    pub on_resource_cost: ResourceCostHook,
    pub replaces: Option<EffectId>,
}

impl Effect {
    pub fn new(id: EffectId, source: ModifierSource, duration: EffectDuration) -> Self {
        let noop = Arc::new(|_: &mut World, _: Entity| {}) as EffectHook;

        Self {
            id,
            source,
            duration,
            on_apply: noop.clone(),
            on_unapply: noop.clone(),
            on_skill_check: HashMap::new(),
            on_saving_throw: HashMap::new(),
            pre_attack_roll: Arc::new(|_: &World, _: Entity, _: &mut AttackRoll| {})
                as AttackRollHook,
            post_attack_roll: Arc::new(|_: &World, _: Entity, _: &mut AttackRollResult| {})
                as AttackRollResultHook,
            on_armor_class: Arc::new(|_: &World, _: Entity, _: &mut ArmorClass| {})
                as ArmorClassHook,
            pre_damage_roll: Arc::new(|_: &World, _: Entity, _: &mut DamageRoll| {})
                as DamageRollHook,
            post_damage_roll: Arc::new(|_: &World, _: Entity, _: &mut DamageRollResult| {})
                as DamageRollResultHook,
            on_action: Arc::new(
                |_: &mut World, _: Entity, _: &Action, _: &ActionContext, _: &ResourceAmountMap| {},
            ) as ActionHook,
            on_resource_cost: Arc::new(
                |_: &World, _: Entity, _: &ActionContext, _: &mut ResourceAmountMap| {},
            ) as ResourceCostHook,
            replaces: None,
        }
    }

    pub fn increment_turns(&mut self) {
        if let EffectDuration::Temporary {
            duration: _,
            ref mut turns_elapsed,
        } = self.duration
        {
            *turns_elapsed += 1;
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.duration {
            EffectDuration::Instant => true,
            EffectDuration::Permanent | EffectDuration::Conditional => false,
            EffectDuration::Temporary {
                duration,
                turns_elapsed,
            } => turns_elapsed >= duration,
        }
    }

    pub fn id(&self) -> &EffectId {
        &self.id
    }

    pub fn source(&self) -> &ModifierSource {
        &self.source
    }

    pub fn duration(&self) -> &EffectDuration {
        &self.duration
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

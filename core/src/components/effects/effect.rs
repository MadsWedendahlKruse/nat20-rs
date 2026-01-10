use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::{
    chain_hook_field,
    components::{
        actions::action::ActionContext,
        damage::{
            AttackRoll, AttackRollResult, DamageMitigationResult, DamageRoll, DamageRollResult,
        },
        effects::hooks::{
            ActionHook, ApplyEffectHook, ArmorClassHook, AttackRollHook, AttackRollResultHook,
            D20CheckHooks, DamageRollHook, DamageRollResultHook, DeathHook,
            PostDamageMitigationHook, PreDamageMitigationHook, ResourceCostHook, UnapplyEffectHook,
        },
        id::{ActionId, EffectId, IdProvider},
        items::equipment::armor::ArmorClass,
        modifier::ModifierSource,
        resource::ResourceAmountMap,
        saving_throw::SavingThrowKind,
        skill::Skill,
        time::{TimeDuration, TimeStep, TurnBoundary},
    },
    engine::event::ActionData,
    registry::{registry::EffectsRegistry, serialize::effect::EffectDefinition},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectLifetime {
    Permanent,

    /// Expire at Start/End of `entity`'s turn, after `remaining` boundaries.
    /// - remaining = 1 => expire at the next matching boundary
    AtTurnBoundary {
        entity: Entity,
        boundary: TurnBoundary,
        duration: TimeDuration,
        remaining: TimeDuration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectLifetimeEntiy {
    Applier,
    Target,
}

/// Effect lifetimes are unique in the sense that they can refer to different entities,
/// but those entities are only known at runtime. Therefore, we need a template
/// that can be instantiated into a concrete `EffectLifetime` when the effect is applied.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectLifetimeTemplate {
    Permanent,
    AtTurnBoundary {
        entity: EffectLifetimeEntiy,
        boundary: TurnBoundary,
        duration: TimeDuration,
    },
}

impl EffectLifetimeTemplate {
    pub fn instantiate(&self, applier: Entity, target: Entity) -> EffectLifetime {
        match self {
            EffectLifetimeTemplate::Permanent => EffectLifetime::Permanent,

            EffectLifetimeTemplate::AtTurnBoundary {
                entity,
                boundary,
                duration,
            } => {
                let entity = match entity {
                    EffectLifetimeEntiy::Applier => applier,
                    EffectLifetimeEntiy::Target => target,
                };
                EffectLifetime::AtTurnBoundary {
                    entity,
                    boundary: *boundary,
                    duration: *duration,
                    remaining: *duration,
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Buff,
    Debuff,
}

#[derive(Clone, Deserialize)]
#[serde(from = "EffectDefinition")]
pub struct Effect {
    pub id: EffectId,
    pub kind: EffectKind,
    pub description: String,
    pub replaces: Option<EffectId>,

    // on_turn_start: EffectHook,
    // TODO: Do we need to differentiate between when an effect explicitly expires and when
    // the effect is removed from the character?
    // pub on_expire: EffectHook,
    pub on_apply: ApplyEffectHook,
    pub on_unapply: UnapplyEffectHook,
    pub on_skill_check: HashMap<Skill, D20CheckHooks>,
    pub on_saving_throw: HashMap<SavingThrowKind, D20CheckHooks>,
    pub pre_attack_roll: AttackRollHook,
    pub post_attack_roll: AttackRollResultHook,
    pub on_armor_class: ArmorClassHook,
    pub pre_damage_roll: DamageRollHook,
    pub post_damage_roll: DamageRollResultHook,
    pub on_action: ActionHook,
    pub on_resource_cost: ResourceCostHook,
    pub pre_damage_mitigation: PreDamageMitigationHook,
    pub post_damage_mitigation: PostDamageMitigationHook,
    pub on_death: DeathHook,
}

impl Effect {
    pub fn new(id: EffectId, kind: EffectKind, description: String) -> Self {
        Self {
            id,
            kind,
            description,
            on_apply: Arc::new(|_: &mut World, _: Entity, _: Option<&ActionContext>| {})
                as ApplyEffectHook,
            on_unapply: Arc::new(|_: &mut World, _: Entity| {}) as UnapplyEffectHook,
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
            on_action: Arc::new(|_: &mut World, _: &ActionData| {}) as ActionHook,
            on_resource_cost: Arc::new(
                |_: &World,
                 _: Entity,
                 _: &ActionId,
                 _: &ActionContext,
                 _: &mut ResourceAmountMap| {},
            ) as ResourceCostHook,
            pre_damage_mitigation: Arc::new(
                |_: &World, _: Entity, _: &EffectInstance, _: &mut DamageRollResult| {},
            ) as PreDamageMitigationHook,
            post_damage_mitigation: Arc::new(
                |_: &World, _: Entity, _: &mut DamageMitigationResult| {},
            ) as PostDamageMitigationHook,
            on_death: Arc::new(
                |_: &mut World,
                 _victim: Entity,
                 _killer: Option<Entity>,
                 _applier: Option<Entity>| {},
            ) as DeathHook,
            replaces: None,
        }
    }

    pub fn id(&self) -> &EffectId {
        &self.id
    }

    /// TODO: This is definitely not the most elegant way to combine hooks. It could
    /// be argued that the hooks should perhaps be stored in a vector instead, or some
    /// other similar structure, which allows storing multiple hooks of the same type.
    /// However, it's also worth considering if *any* event would actually need to have
    /// multiple hooks for the same type? Would it ever make sense to define something
    /// like an effect which adds +2 and +1 to a skill? That's a question for another
    /// day - right now this works :^)
    pub fn combine_hooks(&mut self, other: &Effect) {
        chain_hook_field!(
            self,
            other,
            on_apply,
            |world: &mut World, entity: Entity, action_context: Option<&ActionContext>|,
            (world, entity, action_context)
        );
        chain_hook_field!(
            self,
            other,
            on_unapply,
            |world: &mut World, entity: Entity|,
            (world, entity)
        );
        chain_hook_field!(
            self,
            other,
            pre_attack_roll,
            |world: &World, entity: Entity, attack_roll: &mut AttackRoll|,
            (world, entity, attack_roll)
        );
        chain_hook_field!(
            self,
            other,
            post_attack_roll,
            |world: &World, entity: Entity, attack_roll_result: &mut AttackRollResult|,
            (world, entity, attack_roll_result)
        );
        chain_hook_field!(
            self,
            other,
            on_armor_class,
            |world: &World, entity: Entity, armor_class: &mut ArmorClass|,
            (world, entity, armor_class)
        );
        chain_hook_field!(
            self,
            other,
            pre_damage_roll,
            |world: &World, entity: Entity, damage_roll: &mut DamageRoll|,
            (world, entity, damage_roll)
        );
        chain_hook_field!(
            self,
            other,
            post_damage_roll,
            |world: &World, entity: Entity, damage_roll_result: &mut DamageRollResult|,
            (world, entity, damage_roll_result)
        );
        chain_hook_field!(
            self,
            other,
            on_action,
            |world: &mut World, action_data: &ActionData|,
            (world, action_data)
        );
        chain_hook_field!(
            self,
            other,
            on_resource_cost,
            |world: &World,
             entity: Entity,
             action_id: &ActionId,
             action_context: &ActionContext,
             resource_costs: &mut ResourceAmountMap|,
            (world, entity, action_id, action_context, resource_costs)
        );
        chain_hook_field!(
            self,
            other,
            pre_damage_mitigation,
            |world: &World,
             entity: Entity,
             effect_instance: &EffectInstance,
             damage_roll_result: &mut DamageRollResult|,
            (world, entity, effect_instance, damage_roll_result)
        );
        chain_hook_field!(
            self,
            other,
            post_damage_mitigation,
            |world: &World, entity: Entity, mitigation_result: &mut DamageMitigationResult|,
            (world, entity, mitigation_result)
        );
        chain_hook_field!(
            self,
            other,
            on_death,
            |world: &mut World,
             victim: Entity,
             killer: Option<Entity>,
             applier: Option<Entity>|,
            (world, victim, killer, applier)
        );
        for (skill, hooks) in &other.on_skill_check {
            self.on_skill_check
                .entry(*skill)
                .and_modify(|existing_hooks| {
                    existing_hooks.combine_hooks(hooks);
                })
                .or_insert_with(|| hooks.clone());
        }
        for (saving_throw, hooks) in &other.on_saving_throw {
            self.on_saving_throw
                .entry(*saving_throw)
                .and_modify(|existing_hooks| {
                    existing_hooks.combine_hooks(hooks);
                })
                .or_insert_with(|| hooks.clone());
        }
    }
}

impl IdProvider for Effect {
    type Id = EffectId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct EffectInstance {
    pub effect_id: EffectId,
    pub source: ModifierSource,
    pub applier: Option<Entity>,
    pub lifetime: EffectLifetime,
}

impl EffectInstance {
    pub fn new(effect_id: EffectId, source: ModifierSource, lifetime: EffectLifetime) -> Self {
        Self {
            effect_id,
            source,
            lifetime,
            applier: None,
        }
    }

    pub fn permanent(effect_id: EffectId, source: ModifierSource) -> Self {
        Self::new(effect_id, source, EffectLifetime::Permanent)
    }

    pub fn effect(&self) -> &Effect {
        EffectsRegistry::get(&self.effect_id)
            .expect(format!("Effect definition not found for ID `{}`", self.effect_id).as_str())
    }

    pub fn advance_time(&mut self, time_step: TimeStep) {
        match self.lifetime {
            EffectLifetime::Permanent => { /* Do nothing */ }

            EffectLifetime::AtTurnBoundary {
                entity: life_time_entity,
                boundary: lifetime_boundary,
                ref mut remaining,
                ..
            } => {
                match time_step {
                    TimeStep::TurnBoundary {
                        entity: time_step_entity,
                        boundary: time_step_boundary,
                    } => {
                        if !(time_step_entity == life_time_entity
                            && time_step_boundary == lifetime_boundary)
                        {
                            return;
                        }
                    }
                    _ => { /* Do nothing */ }
                }
                remaining.decrement(&time_step);
            }
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.lifetime {
            EffectLifetime::Permanent => false,

            EffectLifetime::AtTurnBoundary { ref remaining, .. } => remaining.as_turns() == 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectInstanceTemplate {
    pub effect_id: EffectId,
    pub lifetime: EffectLifetimeTemplate,
}

impl EffectInstanceTemplate {
    pub fn instantiate(
        &self,
        applier: Entity,
        target: Entity,
        source: ModifierSource,
    ) -> EffectInstance {
        EffectInstance {
            effect_id: self.effect_id.clone(),
            source,
            lifetime: self.lifetime.instantiate(applier, target),
            applier: Some(applier),
        }
    }

    pub fn effect(&self) -> &Effect {
        EffectsRegistry::get(&self.effect_id)
            .expect(format!("Effect definition not found for ID `{}`", self.effect_id).as_str())
    }
}

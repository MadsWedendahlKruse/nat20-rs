use hecs::{Entity, World};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::sync::Arc;

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::ActionContext,
        d20::{D20CheckKey, D20CheckSet},
        damage::{
            DamageMitigationEffect, DamageMitigationResult, DamageResistances, DamageRollResult,
        },
        effects::{
            effects::{Effect, EffectDuration, EffectKind},
            hooks::{
                ActionHook, ArmorClassHook, AttackRollHook, DamageRollResultHook, DamageTakenHook,
                ResourceCostHook,
            },
        },
        health::hit_points::{HitPoints, TemporaryHitPoints},
        id::{ActionId, EffectId, ResourceId, ScriptId},
        items::equipment::armor::ArmorClass,
        modifier::{KeyedModifiable, Modifiable, ModifierSource},
        resource::{ResourceAmount, ResourceAmountMap, ResourceMap},
        saving_throw::SavingThrowSet,
        skill::SkillSet,
        speed::Speed,
    },
    engine::event::ActionData,
    registry::{
        registry_validation::{ReferenceCollector, RegistryReference, RegistryReferenceCollector},
        serialize::{
            dice::HealEquation,
            modifier::{
                AbilityModifierProvider, ArmorClassModifierProvider, AttackRollModifier,
                AttackRollModifierProvider, D20CheckModifierProvider, DamageResistanceProvider,
                SavingThrowModifierProvider, SkillModifierProvider, SpeedModifier,
                SpeedModifierProvider,
            },
        },
    },
    scripts::{
        script::ScriptFunction,
        script_api::{
            ScriptActionView, ScriptDamageMitigationResult, ScriptDamageRollResult,
            ScriptEntityView, ScriptResourceCost,
        },
    },
    systems,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDefinition {
    pub id: EffectId,
    pub kind: EffectKind,
    pub description: String,
    pub duration: EffectDurationDefinition,

    /// If present, this effect replaces another effect with the given id
    #[serde(default)]
    pub replaces: Option<EffectId>,

    /// Simple effect modifiers like:
    /// - Ability score changes
    /// - Skill modifiers
    /// - Saving throw modifiers
    /// - Damage resistances
    /// - Resource changes
    #[serde(default)]
    pub modifiers: Vec<EffectModifier>,

    /// Other hooks can be either pattern-based or script-based
    #[serde(default)]
    pub pre_attack_roll: Vec<AttackRollHookDefinition>,
    // #[serde(default)]
    // pub post_attack_roll: Vec<AttackRollResultHookDef>,
    #[serde(default)]
    pub on_armor_class: Vec<ArmorClassHookDefinition>,

    // #[serde(default)]
    // pub pre_damage_roll: Vec<DamageRollHookDef>,
    #[serde(default)]
    pub post_damage_roll: Vec<DamageRollResultHookDefinition>,
    #[serde(default)]
    pub on_damage_taken: Vec<DamageTakenHookDefinition>,
    /// “Big” custom logic lives here
    #[serde(default)]
    pub on_action: Vec<ActionHookDefinition>,
    #[serde(default)]
    pub on_resource_cost: Vec<ResourceCostHookDefinition>,
}

impl From<EffectDefinition> for Effect {
    fn from(definition: EffectDefinition) -> Self {
        let effect_id = definition.id.clone();

        let mut effect = Effect::new(
            effect_id.clone(),
            definition.kind,
            definition.description,
            definition.duration.into(),
        );

        // 1. Simple persistent modifiers
        // Build on_apply from all modifiers
        {
            let effect_id = effect_id.clone();
            let modifiers = definition.modifiers.clone();
            effect.on_apply = Arc::new(
                move |world: &mut World, entity: Entity, context: Option<&ActionContext>| {
                    for modifier in &modifiers {
                        modifier.evaluate(world, entity, &effect_id, EffectPhase::Apply, context);
                    }
                },
            );
        }

        // Build on_unapply from the *same* modifiers, but different phase
        {
            let effect_id = effect_id.clone();
            let modifiers_for_unapply = definition.modifiers;
            effect.on_unapply = Arc::new(move |world: &mut World, entity: Entity| {
                for modifier in &modifiers_for_unapply {
                    modifier.evaluate(world, entity, &effect_id, EffectPhase::Unapply, None);
                }
            });
        }

        // 2. Hook-based modifiers
        // Build pre_attack_roll hooks
        {
            let hooks = collect_effect_hooks(&definition.pre_attack_roll, &effect_id);
            effect.pre_attack_roll = AttackRollHookDefinition::combine_hooks(hooks);
        }

        // Build post_damage_roll hooks
        {
            let hooks = collect_effect_hooks(&definition.post_damage_roll, &effect_id);
            effect.post_damage_roll = DamageRollResultHookDefinition::combine_hooks(hooks);
        }

        // Build damage_taken hooks
        {
            let hooks = collect_effect_hooks(&definition.on_damage_taken, &effect_id);
            effect.damage_taken = DamageTakenHookDefinition::combine_hooks(hooks);
        }

        // Build armor class hooks
        {
            let hooks = collect_effect_hooks(&definition.on_armor_class, &effect_id);
            effect.on_armor_class = ArmorClassHookDefinition::combine_hooks(hooks);
        }

        // Build resource cost hooks
        {
            let hooks = collect_effect_hooks(&definition.on_resource_cost, &effect_id);
            effect.on_resource_cost = ResourceCostHookDefinition::combine_hooks(hooks);
        }

        // Build on_action hooks
        {
            let hooks = collect_effect_hooks(&definition.on_action, &effect_id);
            effect.on_action = ActionHookDefinition::combine_hooks(hooks);
        }

        effect
    }
}

impl RegistryReferenceCollector for EffectDefinition {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        if let Some(replaces) = &self.replaces {
            collector.add(RegistryReference::Effect(replaces.clone()));
        }
        for modifier in &self.modifiers {
            match modifier {
                EffectModifier::Resource { resource, .. } => {
                    collector.add(RegistryReference::Resource(resource.clone()));
                }
                _ => { /* No references to collect */ }
            }
        }
        for hook in &self.pre_attack_roll {
            match hook {
                AttackRollHookDefinition::Script { script } => {
                    collector.add(RegistryReference::Script(
                        script.clone(),
                        ScriptFunction::AttackRollHook,
                    ));
                }
                _ => { /* No references to collect */ }
            }
        }
        for hook in &self.post_damage_roll {
            match hook {
                DamageRollResultHookDefinition::Script { script } => {
                    collector.add(RegistryReference::Script(
                        script.clone(),
                        ScriptFunction::DamageRollResultHook,
                    ));
                }
            }
        }
        for hook in &self.on_armor_class {
            match hook {
                ArmorClassHookDefinition::Modifier { .. } => { /* No references to collect */ }
                ArmorClassHookDefinition::Script { script } => {
                    collector.add(RegistryReference::Script(
                        script.clone(),
                        ScriptFunction::ArmorClassHook,
                    ));
                }
            }
        }
        for hook in &self.on_action {
            match hook {
                ActionHookDefinition::Script { script } => {
                    collector.add(RegistryReference::Script(
                        script.clone(),
                        ScriptFunction::ActionHook,
                    ));
                }
            }
        }
        for hook in &self.on_resource_cost {
            match hook {
                ResourceCostHookDefinition::Script { script } => {
                    collector.add(RegistryReference::Script(
                        script.clone(),
                        ScriptFunction::ResourceCostHook,
                    ));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String", rename_all = "snake_case")]
pub enum EffectDurationDefinition {
    Instant,
    Temporary { duration: u32 },
    Conditional,
    Permanent,
}

impl Into<EffectDuration> for EffectDurationDefinition {
    fn into(self) -> EffectDuration {
        match self {
            EffectDurationDefinition::Instant => EffectDuration::Instant,
            EffectDurationDefinition::Conditional => EffectDuration::Conditional,
            EffectDurationDefinition::Permanent => EffectDuration::Permanent,
            EffectDurationDefinition::Temporary { duration } => EffectDuration::Temporary {
                duration,
                turns_elapsed: 0,
            },
        }
    }
}

impl Into<String> for EffectDurationDefinition {
    fn into(self) -> String {
        match self {
            EffectDurationDefinition::Instant => "instant".to_string(),
            EffectDurationDefinition::Conditional => "conditional".to_string(),
            EffectDurationDefinition::Permanent => "permanent".to_string(),
            EffectDurationDefinition::Temporary { duration } => {
                format!("temporary({})", duration)
            }
        }
    }
}

impl TryFrom<String> for EffectDurationDefinition {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "instant" => Ok(EffectDurationDefinition::Instant),
            "conditional" => Ok(EffectDurationDefinition::Conditional),
            "permanent" => Ok(EffectDurationDefinition::Permanent),
            _ => {
                if value.starts_with("temporary(") && value.ends_with(')') {
                    let inner = &value["temporary(".len()..value.len() - 1];
                    let duration = inner
                        .parse::<u32>()
                        .map_err(|e| format!("Failed to parse duration in temporary(): {}", e))?;
                    Ok(EffectDurationDefinition::Temporary { duration })
                } else {
                    Err(format!("Unknown effect duration: {}", value))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EffectModifier {
    Ability {
        ability: AbilityModifierProvider,
    },
    Skill {
        skill: SkillModifierProvider,
    },
    SavingThrow {
        saving_throw: SavingThrowModifierProvider,
    },
    DamageResistance {
        resistance: DamageResistanceProvider,
    },
    Resource {
        resource: ResourceId,
        amount: ResourceAmount,
    },
    Speed {
        speed: SpeedModifierProvider,
    },
    TemporaryHitPoints {
        temporary_hit_points: HealEquation,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum EffectPhase {
    Apply,
    Unapply,
}

impl EffectModifier {
    pub fn evaluate(
        &self,
        world: &mut World,
        entity: Entity,
        effect_id: &EffectId,
        phase: EffectPhase,
        context: Option<&ActionContext>,
    ) {
        let source = ModifierSource::Effect(effect_id.clone());
        match self {
            EffectModifier::Ability { ability: modifier } => {
                let mut abilities =
                    systems::helpers::get_component_mut::<AbilityScoreMap>(world, entity);
                match phase {
                    EffectPhase::Apply => {
                        abilities.add_modifier(modifier.ability, source, modifier.delta);
                    }
                    EffectPhase::Unapply => {
                        abilities.remove_modifier(modifier.ability, &source);
                    }
                }
            }

            EffectModifier::Skill { skill: modifier } => {
                let mut skills = systems::helpers::get_component_mut::<SkillSet>(world, entity);
                Self::apply_d20_check_modifier(&mut *skills, modifier, source, phase);
            }

            EffectModifier::SavingThrow {
                saving_throw: modifier,
            } => {
                let mut saves =
                    systems::helpers::get_component_mut::<SavingThrowSet>(world, entity);
                Self::apply_d20_check_modifier(&mut *saves, modifier, source, phase);
            }

            EffectModifier::DamageResistance {
                resistance: modifier,
            } => {
                let mut res =
                    systems::helpers::get_component_mut::<DamageResistances>(world, entity);
                let mitigation_effect = DamageMitigationEffect {
                    source: source.clone(),
                    operation: modifier.operation.clone(),
                };
                match phase {
                    EffectPhase::Apply => {
                        res.add_effect(modifier.damage_type, mitigation_effect);
                    }
                    EffectPhase::Unapply => {
                        res.remove_effect(modifier.damage_type, &mitigation_effect);
                    }
                }
            }

            EffectModifier::Resource { resource, amount } => {
                let mut resources =
                    systems::helpers::get_component_mut::<ResourceMap>(world, entity);
                match phase {
                    EffectPhase::Apply => {
                        resources.add_uses(resource, amount);
                    }
                    EffectPhase::Unapply => {
                        resources.remove_uses(resource, amount);
                    }
                }
            }

            EffectModifier::Speed { speed: modifier } => {
                let mut speed = systems::helpers::get_component_mut::<Speed>(world, entity);
                match phase {
                    EffectPhase::Apply => match &modifier.modifier {
                        SpeedModifier::Flat(bonus) => {
                            speed.add_flat_modifier(
                                source,
                                bonus.evaluate_without_variables().unwrap().value,
                            );
                        }
                        SpeedModifier::Multiplier(multiplier) => {
                            speed.add_multiplier(source, *multiplier);
                        }
                    },
                    EffectPhase::Unapply => match modifier.modifier {
                        SpeedModifier::Flat(_) => {
                            speed.remove_flat_modifier(&source);
                        }
                        SpeedModifier::Multiplier(_) => {
                            speed.remove_multiplier(&source);
                        }
                    },
                }
            }

            EffectModifier::TemporaryHitPoints {
                temporary_hit_points,
            } => {
                if let Some(context) = context {
                    let amount = (temporary_hit_points.function)(world, entity, context)
                        .roll()
                        .subtotal as u32;
                    let mut hit_points =
                        systems::helpers::get_component_mut::<HitPoints>(world, entity);
                    let source = ModifierSource::Effect(effect_id.clone());
                    match phase {
                        EffectPhase::Apply => {
                            hit_points.set_temp(TemporaryHitPoints::new(amount, &source));
                        }
                        EffectPhase::Unapply => {
                            hit_points.clear_temp(&source);
                        }
                    }
                }
            }
        }
    }

    fn apply_d20_check_modifier<K>(
        modifiable: &mut D20CheckSet<K>,
        modifier: &D20CheckModifierProvider<K>,
        source: ModifierSource,
        phase: EffectPhase,
    ) where
        K: D20CheckKey + DeserializeOwned,
    {
        match phase {
            EffectPhase::Apply => {
                if let Some(delta) = modifier.delta {
                    modifiable.add_modifier(modifier.kind, source.clone(), delta);
                }
                if let Some(advantage_type) = modifier.advantage {
                    modifiable.add_advantage(modifier.kind, advantage_type, source);
                }
            }
            EffectPhase::Unapply => {
                modifiable.remove_modifier(modifier.kind, &source);
                modifiable.remove_advantage(modifier.kind, &source);
            }
        }
    }
}

/// Trait for effects that rely on hooks rather than simple modifiers
pub trait HookEffect<HookFn> {
    fn build_hook(&self, effect_id: &EffectId) -> HookFn;

    fn combine_hooks(hooks: Vec<HookFn>) -> HookFn;
}

fn collect_effect_hooks<HookFn, HookDefinition>(
    definitions: &Vec<HookDefinition>,
    effect_id: &EffectId,
) -> Vec<HookFn>
where
    HookDefinition: HookEffect<HookFn>,
{
    definitions
        .iter()
        .map(|def| def.build_hook(effect_id))
        .collect::<Vec<HookFn>>()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttackRollHookDefinition {
    Modifier {
        modifier: AttackRollModifierProvider,
    },
    Script {
        script: ScriptId,
    },
}

impl HookEffect<AttackRollHook> for AttackRollHookDefinition {
    fn build_hook(&self, effect: &EffectId) -> AttackRollHook {
        match self {
            AttackRollHookDefinition::Modifier { modifier } => {
                let modifier_source = ModifierSource::Effect(effect.clone());
                Arc::new({
                    let modifier = modifier.clone();
                    move |_world, _entity, attack_roll| {
                        if let Some(damage_source) = &modifier.source
                            && *damage_source != attack_roll.source
                        {
                            // Only apply if the damage source matches
                            return;
                        }

                        if let Some(attack_modifier) = &modifier.modifier {
                            match attack_modifier {
                                AttackRollModifier::FlatBonus(bonus) => {
                                    attack_roll
                                        .d20_check
                                        .add_modifier(modifier_source.clone(), *bonus);
                                }
                                AttackRollModifier::Advantage(advantage) => {
                                    attack_roll
                                        .d20_check
                                        .advantage_tracker_mut()
                                        .add(*advantage, modifier_source.clone());
                                }
                                AttackRollModifier::CritThreshold(threshold) => {
                                    attack_roll.reduce_crit_threshold(*threshold);
                                }
                            }
                        }
                    }
                })
            }

            AttackRollHookDefinition::Script { script } => {
                todo!("Implement script-based AttackRollHook")
            }
        }
    }

    fn combine_hooks(hooks: Vec<AttackRollHook>) -> AttackRollHook {
        Arc::new(move |world, entity, attack_roll| {
            for hook in &hooks {
                hook(world, entity, attack_roll);
            }
        })
    }
}

pub enum DamageRollHookDefinition {
    // …
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageRollResultHookDefinition {
    Script { script: ScriptId },
}

impl HookEffect<DamageRollResultHook> for DamageRollResultHookDefinition {
    fn build_hook(&self, _effect: &EffectId) -> DamageRollResultHook {
        match self {
            DamageRollResultHookDefinition::Script { script } => {
                let script_id = script.clone();

                Arc::new(
                    move |world: &World,
                          entity: Entity,
                          damage_roll_result: &mut DamageRollResult| {
                        let entity_view = ScriptEntityView::new_from_world(world, entity);
                        let script_damage_roll_result =
                            ScriptDamageRollResult::take_from(damage_roll_result);

                        systems::scripts::evaluate_damage_roll_result_hook(
                            &script_id,
                            &entity_view,
                            &script_damage_roll_result,
                        );

                        *damage_roll_result = script_damage_roll_result.into_inner();
                    },
                )
            }
        }
    }

    fn combine_hooks(hooks: Vec<DamageRollResultHook>) -> DamageRollResultHook {
        Arc::new(
            move |world: &World, entity: Entity, damage_roll_result: &mut DamageRollResult| {
                for hook in &hooks {
                    hook(world, entity, damage_roll_result);
                }
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmorClassHookDefinition {
    Modifier {
        modifier: ArmorClassModifierProvider,
    },
    Script {
        script: ScriptId,
    },
}

impl HookEffect<ArmorClassHook> for ArmorClassHookDefinition {
    fn build_hook(&self, effect: &EffectId) -> ArmorClassHook {
        match self {
            ArmorClassHookDefinition::Modifier { modifier } => Arc::new({
                let modifier = modifier.clone();
                let effect = effect.clone();
                move |_world, _entity, armor_class| {
                    armor_class
                        .add_modifier(ModifierSource::Effect(effect.clone()), modifier.delta);
                }
            }),

            ArmorClassHookDefinition::Script { script } => {
                let effect_id = effect.clone();
                let script_id = script.clone();
                Arc::new(
                    move |world: &World, entity: Entity, armor_class: &mut ArmorClass| {
                        let entity_view = ScriptEntityView::new_from_world(world, entity);

                        let modifier =
                            systems::scripts::evaluate_armor_class_hook(&script_id, &entity_view);
                        armor_class
                            .add_modifier(ModifierSource::Effect(effect_id.clone()), modifier);
                    },
                )
            }
        }
    }

    fn combine_hooks(hooks: Vec<ArmorClassHook>) -> ArmorClassHook {
        Arc::new(move |world, entity, armor_class| {
            for hook in &hooks {
                hook(world, entity, armor_class);
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionHookDefinition {
    Script { script: ScriptId },
}

impl HookEffect<ActionHook> for ActionHookDefinition {
    fn build_hook(&self, _effect: &EffectId) -> ActionHook {
        match self {
            ActionHookDefinition::Script { script } => {
                let script_id = script.clone();
                Arc::new(move |world: &mut World, action_data: &ActionData| {
                    let action_view = ScriptActionView::from(action_data);

                    let entity_view = ScriptEntityView::take_from_world(world, action_data.actor);

                    systems::scripts::evalute_action_hook(&script_id, &action_view, &entity_view);

                    // Replace the entity in the world with the modified one
                    entity_view.replace_in_world(world);
                })
            }
        }
    }

    fn combine_hooks(hooks: Vec<ActionHook>) -> ActionHook {
        Arc::new(move |world: &mut World, action_data: &ActionData| {
            for hook in &hooks {
                hook(world, action_data);
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceCostHookDefinition {
    Script { script: ScriptId },
}

impl HookEffect<ResourceCostHook> for ResourceCostHookDefinition {
    fn build_hook(&self, _effect: &EffectId) -> ResourceCostHook {
        match self {
            ResourceCostHookDefinition::Script { script } => {
                let script_id = script.clone();
                Arc::new(
                    move |world: &World,
                          entity: Entity,
                          action: &ActionId,
                          context: &ActionContext,
                          resource_costs: &mut ResourceAmountMap| {
                        // Evaluate the script, which can modify the shared resource costs.
                        let action_view = ScriptActionView::new(
                            action,
                            entity,
                            context,
                            // Move out (no clone), leaving an empty map behind temporarily
                            ScriptResourceCost::take_from(resource_costs),
                            // TODO: Not sure what to do about targets here
                            Vec::new(),
                        );

                        let entity_view = ScriptEntityView::new_from_world(world, entity);

                        systems::scripts::evaluate_resource_cost_hook(
                            &script_id,
                            &action_view,
                            &entity_view,
                        );

                        // Move back out (no clone)
                        *resource_costs = action_view.resource_cost.into_inner();
                    },
                )
            }
        }
    }

    fn combine_hooks(hooks: Vec<ResourceCostHook>) -> ResourceCostHook {
        Arc::new(
            move |world: &World,
                  entity: Entity,
                  action: &ActionId,
                  context: &ActionContext,
                  resource_costs: &mut ResourceAmountMap| {
                for hook in &hooks {
                    hook(world, entity, action, context, resource_costs);
                }
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageTakenHookDefinition {
    Script { script: ScriptId },
}

impl HookEffect<DamageTakenHook> for DamageTakenHookDefinition {
    fn build_hook(&self, _effect: &EffectId) -> DamageTakenHook {
        match self {
            DamageTakenHookDefinition::Script { script } => {
                let script_id = script.clone();
                Arc::new(
                    move |world: &World,
                          entity: Entity,
                          damage_mitigation_result: &mut DamageMitigationResult| {
                        let entity_view = ScriptEntityView::new_from_world(world, entity);
                        let script_damage_mitigation_result =
                            ScriptDamageMitigationResult::take_from(damage_mitigation_result);

                        systems::scripts::evaluate_damage_taken_hook(
                            &script_id,
                            &entity_view,
                            &script_damage_mitigation_result,
                        );

                        *damage_mitigation_result = script_damage_mitigation_result.into_inner();
                    },
                )
            }
        }
    }

    fn combine_hooks(hooks: Vec<DamageTakenHook>) -> DamageTakenHook {
        Arc::new(
            move |world: &World,
                  entity: Entity,
                  damage_mitigation_result: &mut DamageMitigationResult| {
                for hook in &hooks {
                    hook(world, entity, damage_mitigation_result);
                }
            },
        )
    }
}

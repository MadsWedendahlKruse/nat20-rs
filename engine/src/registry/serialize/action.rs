use serde::{Deserialize, Serialize};

use crate::{
    components::{
        actions::action::{Action, ActionCondition, ActionKind, ActionPayload, DamageOnFailure},
        id::{ActionId, EffectId, ScriptId},
        resource::{RechargeRule, ResourceAmountMap},
    },
    registry::{
        registry_validation::{ReferenceCollector, RegistryReference, RegistryReferenceCollector},
        serialize::{
            d20::{AttackRollProvider, SavingThrowProvider},
            dice::{DamageEquation, HealEquation},
            targeting::TargetingDefinition,
        },
    },
    scripts::script::ScriptFunction,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageOnFailureDefinition {
    Half,
    Custom(DamageEquation),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionConditionDefinition {
    AttackRoll {
        attack_roll: AttackRollProvider,
        #[serde(default)]
        damage_on_miss: Option<DamageOnFailureDefinition>,
    },
    SavingThrow {
        saving_throw: SavingThrowProvider,
        #[serde(default)]
        damage_on_save: Option<DamageOnFailureDefinition>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ActionPayloadDefinition {
    #[serde(default)]
    pub damage: Option<DamageEquation>,
    #[serde(default)]
    pub healing: Option<HealEquation>,
    #[serde(default)]
    pub effect: Option<EffectId>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKindDefinition {
    Standard {
        #[serde(default)]
        condition: Option<ActionConditionDefinition>,
        payload: ActionPayloadDefinition,
    },

    // Utility {
    //     // most likely non-data / hand-written only,
    //     // you can keep this variant out of the serializable enum if you want
    // },
    Composite {
        actions: Vec<ActionKindDefinition>,
    },
    // Same story here: you probably will not serialize these directly
    // for now, so they can live only on the runtime enum.
    Reaction {
        script: ScriptId,
    }, // Custom(...)
}

impl From<ActionKindDefinition> for ActionKind {
    fn from(spec: ActionKindDefinition) -> Self {
        match spec {
            ActionKindDefinition::Standard { condition, payload } => ActionKind::Standard {
                condition: if let Some(condition) = condition {
                    match condition {
                        ActionConditionDefinition::AttackRoll {
                            attack_roll,
                            damage_on_miss,
                        } => ActionCondition::AttackRoll {
                            attack_roll: attack_roll.function,
                            damage_on_miss: damage_on_miss.map(|damage_on_failure| {
                                match damage_on_failure {
                                    DamageOnFailureDefinition::Half => DamageOnFailure::Half,
                                    DamageOnFailureDefinition::Custom(damage_equation) => {
                                        DamageOnFailure::Custom(damage_equation.function)
                                    }
                                }
                            }),
                        },
                        ActionConditionDefinition::SavingThrow {
                            saving_throw,
                            damage_on_save,
                        } => ActionCondition::SavingThrow {
                            saving_throw: saving_throw.function,
                            damage_on_save: damage_on_save.map(|damage_on_failure| {
                                match damage_on_failure {
                                    DamageOnFailureDefinition::Half => DamageOnFailure::Half,
                                    DamageOnFailureDefinition::Custom(damage_equation) => {
                                        DamageOnFailure::Custom(damage_equation.function)
                                    }
                                }
                            }),
                        },
                    }
                } else {
                    ActionCondition::None
                },
                payload: ActionPayload::new(
                    payload.damage.map(|eq| eq.function),
                    payload.effect,
                    payload.healing.map(|eq| eq.function),
                )
                .unwrap(),
            },

            ActionKindDefinition::Composite { actions } => ActionKind::Composite {
                actions: actions.into_iter().map(ActionKind::from).collect(),
            },

            ActionKindDefinition::Reaction { script } => ActionKind::Reaction { reaction: script },
        }
    }
}

impl RegistryReferenceCollector for ActionKindDefinition {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        match self {
            ActionKindDefinition::Standard { payload, .. } => {
                if let Some(effect_id) = &payload.effect {
                    collector.add(RegistryReference::Effect(effect_id.clone()));
                }
            }
            ActionKindDefinition::Composite { actions } => {
                for action in actions {
                    action.collect_registry_references(collector);
                }
            }
            ActionKindDefinition::Reaction { script } => {
                collector.add(RegistryReference::Script(
                    script.clone(),
                    ScriptFunction::ReactionBody,
                ));
            }
            _ => { /* No references to collect */ }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub id: ActionId,
    pub description: String,
    pub kind: ActionKindDefinition,
    pub targeting: TargetingDefinition,
    /// e.g. Action, Bonus Action, Reaction
    pub resource_cost: ResourceAmountMap,
    /// Optional cooldown for the action
    #[serde(default)]
    pub cooldown: Option<RechargeRule>,
    // TODO: How to handle reaction triggers in serialization?
    #[serde(default)]
    pub reaction_trigger: Option<ScriptId>,
}

impl RegistryReferenceCollector for ActionDefinition {
    fn collect_registry_references(&self, collector: &mut ReferenceCollector) {
        self.kind.collect_registry_references(collector);
        for resource in self.resource_cost.keys() {
            collector.add(RegistryReference::Resource(resource.clone()));
        }
        if let Some(reaction_trigger) = &self.reaction_trigger {
            collector.add(RegistryReference::Script(
                reaction_trigger.clone(),
                ScriptFunction::ReactionTrigger,
            ));
        }
    }
}

impl From<ActionDefinition> for Action {
    fn from(value: ActionDefinition) -> Self {
        Action {
            id: value.id,
            description: value.description,
            kind: value.kind.into(),
            resource_cost: value.resource_cost,
            targeting: value.targeting.function(),
            cooldown: value.cooldown,
            reaction_trigger: value.reaction_trigger,
        }
    }
}

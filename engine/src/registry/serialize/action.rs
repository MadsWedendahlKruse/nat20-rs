use serde::{Deserialize, Serialize};

use crate::{
    components::{
        actions::action::{Action, ActionKind},
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
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKindDefinition {
    UnconditionalDamage {
        damage: DamageEquation,
    },

    AttackRollDamage {
        attack_roll: AttackRollProvider,
        damage: DamageEquation,
        #[serde(default)]
        damage_on_miss: Option<DamageEquation>,
    },

    SavingThrowDamage {
        saving_throw: SavingThrowProvider,
        half_damage_on_save: bool,
        damage: DamageEquation,
    },

    UnconditionalEffect {
        effect: EffectId,
    },

    SavingThrowEffect {
        saving_throw: SavingThrowProvider,
        effect: EffectId,
    },

    BeneficialEffect {
        effect: EffectId,
    },

    Healing {
        heal: HealEquation,
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
            ActionKindDefinition::UnconditionalDamage { damage } => {
                ActionKind::UnconditionalDamage {
                    damage: damage.function,
                }
            }

            ActionKindDefinition::AttackRollDamage {
                attack_roll,
                damage,
                damage_on_miss,
            } => ActionKind::AttackRollDamage {
                attack_roll: attack_roll.function,
                damage: damage.function,
                damage_on_miss: damage_on_miss.map(|equation| equation.function),
            },

            ActionKindDefinition::SavingThrowDamage {
                saving_throw,
                half_damage_on_save,
                damage,
            } => ActionKind::SavingThrowDamage {
                saving_throw: saving_throw.function,
                half_damage_on_save,
                damage: damage.function,
            },

            ActionKindDefinition::UnconditionalEffect { effect } => {
                ActionKind::UnconditionalEffect { effect }
            }

            ActionKindDefinition::SavingThrowEffect {
                saving_throw,
                effect,
            } => ActionKind::SavingThrowEffect {
                saving_throw: saving_throw.function,
                effect,
            },

            ActionKindDefinition::BeneficialEffect { effect } => {
                ActionKind::BeneficialEffect { effect }
            }

            ActionKindDefinition::Healing { heal } => ActionKind::Healing {
                heal: heal.function,
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
            ActionKindDefinition::UnconditionalEffect { effect }
            | ActionKindDefinition::SavingThrowEffect { effect, .. }
            | ActionKindDefinition::BeneficialEffect { effect } => {
                collector.add(RegistryReference::Effect(effect.clone()));
            }
            ActionKindDefinition::Composite { actions } => {
                for action in actions {
                    action.collect_registry_references(collector);
                }
            }
            ActionKindDefinition::Reaction { script } => {
                collector.add(RegistryReference::Script(script.clone()));
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
            collector.add(RegistryReference::Script(reaction_trigger.clone()));
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

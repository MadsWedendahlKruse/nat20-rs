use serde::{Deserialize, Serialize};

use crate::{
    components::{
        actions::action::{Action, ActionKind},
        id::{ActionId, EffectId},
        resource::{RechargeRule, ResourceAmountMap},
    },
    registry::serialize::{
        attack_roll::AttackRollProvider,
        damage::DamageEquation,
        saving_throw::SavingThrowProvider,
        targeting::{TargetingContextDefinition, TargetingDefinition},
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

    // Healing {
    //     // You can define a HealingEquation similar to DamageEquation,
    //     // or re-use DiceExpression + variables and return DiceSetRoll instead.
    //     heal: HealingProvider,
    // },
    // Utility {
    //     // most likely non-data / hand-written only,
    //     // you can keep this variant out of the serializable enum if you want
    // },
    Composite {
        actions: Vec<ActionKindDefinition>,
    },
    // Same story here: you probably will not serialize these directly
    // for now, so they can live only on the runtime enum.
    // Reaction { ... }
    // Custom(...)
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

            // ActionKindSpec::Healing { heal } => ActionKind::Healing {
            //     heal: heal.function,
            // },
            ActionKindDefinition::Composite { actions } => ActionKind::Composite {
                actions: actions.into_iter().map(ActionKind::from).collect(),
            }, // Utility / Reaction / Custom are intentionally not in ActionKindSpec.
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
    // pub reaction_trigger: Option<Arc<dyn Fn(Entity, &Event) -> bool + Send + Sync>>,
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
            reaction_trigger: None,
        }
    }
}

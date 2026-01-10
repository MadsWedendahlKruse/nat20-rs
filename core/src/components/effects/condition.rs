use std::{collections::HashMap, sync::Arc};

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::components::{
    d20::{AdvantageType, D20Check, D20CheckResult},
    damage::AttackRoll,
    effects::{
        effect::{self, Effect},
        hooks::D20CheckHooks,
    },
    id::EffectId,
    modifier::ModifierSource,
    skill::Skill,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Exhaustion,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}

impl Condition {
    pub fn effect_id(&self) -> EffectId {
        EffectId::new("nat20_core", format!("condition.{}", self))
    }

    pub fn description(&self) -> String {
        // TODO: Replace with actual condition descriptions
        "Lorem ipsum".to_string()
    }

    pub fn kind(&self) -> effect::EffectKind {
        match self {
            Condition::Invisible => effect::EffectKind::Buff,
            _ => effect::EffectKind::Debuff,
        }
    }

    pub fn effect(&self) -> Effect {
        let id = self.effect_id();
        let kind = self.kind();
        let description = self.description();
        let mut effect = Effect::new(id.clone(), kind, description);
        let source = ModifierSource::Effect(id.clone());

        match self {
            Condition::Blinded => todo!(),
            Condition::Charmed => todo!(),
            Condition::Deafened => todo!(),
            Condition::Exhaustion => todo!(),
            Condition::Frightened => todo!(),
            Condition::Grappled => todo!(),
            Condition::Incapacitated => todo!(),
            Condition::Invisible => todo!(),
            Condition::Paralyzed => todo!(),
            Condition::Petrified => todo!(),

            Condition::Poisoned => {
                effect.on_skill_check = HashMap::from_iter(Skill::iter().map(|skill| {
                    (
                        skill,
                        D20CheckHooks {
                            check_hook: Arc::new({
                                let source = source.clone();
                                move |_world: &World, _entity: Entity, check: &mut D20Check| {
                                    check
                                        .advantage_tracker_mut()
                                        .add(AdvantageType::Disadvantage, source.clone())
                                }
                            }),
                            result_hook: Arc::new(
                                |_world: &World, _entity: Entity, _result: &mut D20CheckResult| {},
                            ),
                        },
                    )
                }));

                effect.pre_attack_roll = Arc::new({
                    let source = source.clone();
                    move |_world: &World, _attacker: Entity, attack_roll: &mut AttackRoll| {
                        attack_roll
                            .d20_check
                            .advantage_tracker_mut()
                            .add(AdvantageType::Disadvantage, source.clone())
                    }
                });
            }

            Condition::Prone => todo!(),
            Condition::Restrained => todo!(),
            Condition::Stunned => todo!(),
            Condition::Unconscious => todo!(),
        }

        effect
    }
}

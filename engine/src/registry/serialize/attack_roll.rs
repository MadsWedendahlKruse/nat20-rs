use std::{fmt::Display, str::FromStr, sync::Arc};

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        ability::AbilityScoreMap,
        actions::action::{ActionContext, AttackRollFunction},
        d20::D20Check,
        damage::{AttackRoll, DamageSource},
        id::SpellId,
        modifier::ModifierSource,
        proficiency::{Proficiency, ProficiencyLevel},
        spells::spellbook::Spellbook,
    },
    systems,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AttackRollProvider {
    pub raw: String,
    pub function: Arc<AttackRollFunction>,
}

impl Display for AttackRollProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl FromStr for AttackRollProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let function = match s {
            "weapon_attack_roll" => Arc::new(
                |world: &World, entity: Entity, action_context: &ActionContext| {
                    weapon_attack_roll(world, entity, action_context)
                },
            ) as Arc<AttackRollFunction>,
            "spell_attack_roll" => Arc::new({
                |world: &World, entity: Entity, action_context: &ActionContext| {
                    let spell_id = if let ActionContext::Spell { id, .. } = action_context {
                        id
                    } else {
                        panic!("Action context must be Spell for spell_attack_roll");
                    };
                    spell_attack_roll(world, entity, &spell_id)
                }
            }) as Arc<AttackRollFunction>,
            _ => {
                return Err(format!("Unknown AttackRollProvider: {}", s));
            }
        };

        Ok(Self {
            raw: s.to_string(),
            function,
        })
    }
}

impl TryFrom<String> for AttackRollProvider {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<AttackRollProvider> for String {
    fn from(equation: AttackRollProvider) -> Self {
        equation.raw
    }
}

fn weapon_attack_roll(world: &World, entity: Entity, action_context: &ActionContext) -> AttackRoll {
    if let ActionContext::Weapon { slot } = action_context {
        return systems::combat::attack_roll(world, entity, slot);
    }
    panic!("Action context must be Weapon");
}

fn spell_attack_roll(world: &World, caster: Entity, spell_id: &SpellId) -> AttackRoll {
    let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(world, caster);
    let spellcasting_ability = systems::helpers::get_component::<Spellbook>(world, caster)
        .spellcasting_ability(spell_id)
        .unwrap()
        .clone();
    let proficiency_bonus = systems::helpers::level(world, caster)
        .unwrap()
        .proficiency_bonus();

    let mut roll = D20Check::new(Proficiency::new(
        ProficiencyLevel::Proficient,
        ModifierSource::None,
    ));
    let spellcasting_modifier = ability_scores
        .ability_modifier(spellcasting_ability)
        .total();
    roll.add_modifier(
        ModifierSource::Ability(spellcasting_ability),
        spellcasting_modifier,
    );
    roll.add_modifier(
        ModifierSource::Proficiency(ProficiencyLevel::Proficient),
        proficiency_bonus as i32,
    );

    AttackRoll::new(roll, DamageSource::Spell)
}

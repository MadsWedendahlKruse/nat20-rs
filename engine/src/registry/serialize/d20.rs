use std::{fmt::Display, str::FromStr, sync::Arc};

use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        ability::{Ability, AbilityScoreMap},
        actions::action::{ActionContext, AttackRollFunction, SavingThrowFunction},
        d20::{D20Check, D20CheckDC},
        damage::{AttackRoll, DamageSource},
        id::SpellId,
        modifier::{ModifierSet, ModifierSource},
        proficiency::{Proficiency, ProficiencyLevel},
        saving_throw::{SavingThrowDC, SavingThrowKind},
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SavingThrowProvider {
    pub raw: String,
    pub function: Arc<SavingThrowFunction>,
}

impl Display for SavingThrowProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl FromStr for SavingThrowProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Example format: "spell_save_dc;dexterity"

        let parts: Vec<&str> = s.split(';').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid SavingThrowProvider format: {}", s));
        }

        let ability: Ability = serde_plain::from_str(parts[1]).unwrap();

        let function = match parts[0] {
            "weapon_save_dc" => Arc::new(
                |world: &World, entity: Entity, action_context: &ActionContext| {
                    weapon_save_dc(world, entity, action_context)
                },
            ) as Arc<SavingThrowFunction>,

            "spell_save_dc" => Arc::new({
                let ability = ability.clone();
                move |world: &World, entity: Entity, action_context: &ActionContext| {
                    if let ActionContext::Spell { id, .. } = action_context {
                        spell_save_dc(world, entity, id, ability)
                    } else {
                        panic!("Action context must be Spell for spell_save_dc");
                    }
                }
            }) as Arc<SavingThrowFunction>,
            _ => {
                return Err(format!("Unknown SavingThrowProvider: {}", s));
            }
        };

        Ok(Self {
            raw: s.to_string(),
            function,
        })
    }
}

impl TryFrom<String> for SavingThrowProvider {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<SavingThrowProvider> for String {
    fn from(equation: SavingThrowProvider) -> Self {
        equation.raw
    }
}

const BASE_SAVE_DC: i32 = 8;

fn weapon_save_dc(
    world: &World,
    entity: Entity,
    action_context: &ActionContext,
) -> D20CheckDC<SavingThrowKind> {
    todo!("Implement weapon_save_dc");
    // https://www.reddit.com/r/BaldursGate3/comments/16kynf6/how_is_the_save_dc_of_maneuvers_calculated/
    // https://bg3.wiki/wiki/Dice_rolls#Weapon_action_DC
    // if let ActionContext::Weapon { slot } = action_context {
    //     return systems::combat::weapon_saving_throw_dc(world, entity, slot);
    // }
    // panic!("Action context must be Weapon");
}

fn spell_save_dc(
    world: &World,
    caster: Entity,
    spell_id: &SpellId,
    saving_throw_ability: Ability,
) -> SavingThrowDC {
    let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(world, caster);
    let spellcasting_ability = systems::helpers::get_component::<Spellbook>(world, caster)
        .spellcasting_ability(spell_id)
        .unwrap()
        .clone();
    let proficiency_bonus = systems::helpers::level(world, caster)
        .unwrap()
        .proficiency_bonus();

    let mut spell_save_dc = ModifierSet::new();
    spell_save_dc.add_modifier(ModifierSource::Base, BASE_SAVE_DC);
    let spellcasting_modifier = ability_scores
        .ability_modifier(spellcasting_ability)
        .total();
    spell_save_dc.add_modifier(
        ModifierSource::Ability(spellcasting_ability),
        spellcasting_modifier,
    );
    // TODO: Not sure if Proficiency is the correct modifier source here, since I don't think
    // you can have e.g. Expertise in spell save DCs.
    spell_save_dc.add_modifier(
        ModifierSource::Proficiency(ProficiencyLevel::Proficient),
        proficiency_bonus as i32,
    );

    D20CheckDC {
        key: SavingThrowKind::Ability(saving_throw_ability),
        dc: spell_save_dc,
    }
}

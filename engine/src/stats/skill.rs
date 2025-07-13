use std::{fmt, hash::Hash};

use crate::{creature::character::Character, effects::hooks::SkillCheckHook};

use super::{
    ability::Ability,
    d20_check::{D20Check, D20CheckResult, D20CheckSet},
};

use strum::EnumIter;

#[derive(EnumIter, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Skill {
    // --- Strength ---
    Athletics,
    // --- Dexterity ---
    Acrobatics,
    SleightOfHand,
    Stealth,
    // Not technically a skill, but it behaves like one
    Initiative,
    // --- Intelligence ---
    Arcana,
    History,
    Investigation,
    Nature,
    Religion,
    // --- Wisdom ---
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival,
    // --- Charisma ---
    Deception,
    Intimidation,
    Performance,
    Persuasion,
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[macro_export]
macro_rules! skill_ability_map {
    ( $( $skill:ident => $ability:ident ),* $(,)? ) => {
        pub const fn skill_ability(skill: Skill) -> Ability {
            match skill {
                $( Skill::$skill => Ability::$ability ),*
            }
        }
    };
}

skill_ability_map! {
    Athletics => Strength,

    Acrobatics => Dexterity,
    SleightOfHand => Dexterity,
    Stealth => Dexterity,

    Arcana => Intelligence,
    History => Intelligence,
    Investigation => Intelligence,
    Nature => Intelligence,
    Religion => Intelligence,

    AnimalHandling => Wisdom,
    Insight => Wisdom,
    Medicine => Wisdom,
    Perception => Wisdom,
    Survival => Wisdom,

    Deception => Charisma,
    Intimidation => Charisma,
    Performance => Charisma,
    Persuasion => Charisma,

    Initiative => Dexterity,
}

pub type SkillSet = D20CheckSet<Skill, SkillCheckHook>;

pub fn get_skill_hooks(skill: Skill, character: &Character) -> Vec<&SkillCheckHook> {
    character
        .effects()
        .iter()
        .filter_map(|e| e.on_skill_check.as_ref())
        .filter(|hook| hook.key == skill)
        .collect()
}

pub fn apply_check_hook(hook: &SkillCheckHook, character: &Character, d20: &mut D20Check) {
    (hook.check_hook)(character, d20);
}

pub fn apply_result_hook(
    hook: &SkillCheckHook,
    character: &Character,
    result: &mut D20CheckResult,
) {
    (hook.result_hook)(character, result);
}

pub fn create_skill_set() -> SkillSet {
    SkillSet::new(
        get_skill_hooks,
        apply_check_hook,
        apply_result_hook,
        skill_ability,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::ability::Ability;

    #[test]
    fn skill_ability_map() {
        assert_eq!(skill_ability(Skill::Acrobatics), Ability::Dexterity);
        assert_eq!(skill_ability(Skill::Athletics), Ability::Strength);
        assert_eq!(skill_ability(Skill::Stealth), Ability::Dexterity);
        assert_eq!(skill_ability(Skill::Arcana), Ability::Intelligence);
        assert_eq!(skill_ability(Skill::History), Ability::Intelligence);
    }
}

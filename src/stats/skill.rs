use std::hash::Hash;

use crate::{creature::character::Character, effects::hooks::SkillCheckHook};

use super::{
    ability::Ability,
    d20_check::{D20Check, D20CheckResult, D20CheckSet},
};

use strum::EnumIter;

#[derive(EnumIter, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Skill {
    Acrobatics,
    Athletics,
    Stealth,
    Arcana,
    History,
    // Add more as needed
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
    Acrobatics => Dexterity,
    Athletics  => Strength,
    Stealth    => Dexterity,
    Arcana     => Intelligence,
    History    => Intelligence,
}

pub type SkillSet = D20CheckSet<Skill, SkillCheckHook>;

pub fn get_skill_hooks(skill: Skill, character: &Character) -> Vec<&SkillCheckHook> {
    character
        .effects()
        .iter()
        .filter_map(|e| e.skill_check_hook.as_ref())
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
    use crate::stats::{ability::Ability, modifier::ModifierSource, proficiency::Proficiency};

    #[test]
    fn skill_ability_map() {
        assert_eq!(skill_ability(Skill::Acrobatics), Ability::Dexterity);
        assert_eq!(skill_ability(Skill::Athletics), Ability::Strength);
        assert_eq!(skill_ability(Skill::Stealth), Ability::Dexterity);
        assert_eq!(skill_ability(Skill::Arcana), Ability::Intelligence);
        assert_eq!(skill_ability(Skill::History), Ability::Intelligence);
    }

    #[test]
    fn skill_set() {
        let mut skill_set = create_skill_set();
        skill_set.set_proficiency(Skill::Acrobatics, Proficiency::Proficient);
        skill_set.add_modifier(
            Skill::Acrobatics,
            ModifierSource::Item("Ring of Acrobatics".to_string()),
            2,
        );
    }
}

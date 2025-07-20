use std::{fmt, hash::Hash};

use crate::{
    components::{
        ability::Ability,
        d20_check::{D20CheckDC, D20CheckSet},
        effects::hooks::D20CheckHooks,
    },
    systems,
};

use hecs::{Entity, World};
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

pub type SkillSet = D20CheckSet<Skill>;

pub type SkillCheckDC = D20CheckDC<Skill>;

pub fn get_skill_hooks(skill: Skill, world: &World, entity: Entity) -> Vec<D20CheckHooks> {
    systems::effects::effects(world, entity)
        .iter()
        .filter_map(|e| e.on_skill_check.get(&skill))
        .cloned()
        .collect()
}

pub fn create_skill_set() -> SkillSet {
    SkillSet::new(skill_ability, get_skill_hooks)
}

#[cfg(test)]
mod tests {
    use crate::components::ability::Ability;

    use super::*;

    #[test]
    fn skill_ability_map() {
        assert_eq!(skill_ability(Skill::Acrobatics), Ability::Dexterity);
        assert_eq!(skill_ability(Skill::Athletics), Ability::Strength);
        assert_eq!(skill_ability(Skill::Stealth), Ability::Dexterity);
        assert_eq!(skill_ability(Skill::Arcana), Ability::Intelligence);
        assert_eq!(skill_ability(Skill::History), Ability::Intelligence);
    }
}

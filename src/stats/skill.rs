use std::hash::Hash;

use super::ability::Ability;

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

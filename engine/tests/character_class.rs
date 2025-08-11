extern crate nat20_rs;

mod tests {

    use std::collections::{HashMap, HashSet};

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::{Ability, AbilityScoreDistribution},
            class::{ClassName, SubclassName},
            level::CharacterLevels,
            proficiency::Proficiency,
            saving_throw::SavingThrowSet,
            skill::{Skill, SkillSet},
        },
        entities::character::Character,
        registry,
        systems::{self, level_up::LevelUpDecision},
    };

    #[test]
    fn character_level_up_fighter() {
        let mut world = World::new();
        let character = world.spawn(Character::default());

        systems::level_up::apply_level_up_decision(
            &mut world,
            character,
            3,
            vec![
                LevelUpDecision::Class(ClassName::Fighter),
                LevelUpDecision::AbilityScores(AbilityScoreDistribution {
                    scores: HashMap::from([
                        (Ability::Strength, 15),
                        (Ability::Dexterity, 14),
                        (Ability::Constitution, 13),
                        (Ability::Intelligence, 8),
                        (Ability::Wisdom, 10),
                        (Ability::Charisma, 12),
                    ]),
                    plus_2_bonus: Ability::Strength,
                    plus_1_bonus: Ability::Constitution,
                }),
                LevelUpDecision::Effect(
                    registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
                ),
                LevelUpDecision::SkillProficiency(HashSet::from([
                    Skill::Athletics,
                    Skill::Perception,
                ])),
                LevelUpDecision::Class(ClassName::Fighter),
                LevelUpDecision::Class(ClassName::Fighter),
                LevelUpDecision::Subclass(SubclassName {
                    class: ClassName::Fighter,
                    name: "Champion".to_string(),
                }),
            ],
        );

        {
            let levels = systems::helpers::get_component::<CharacterLevels>(&mut world, character);
            assert_eq!(levels.total_level(), 3);
            assert_eq!(levels.class_level(&ClassName::Fighter).unwrap().level(), 3);
            assert_eq!(
                levels.class_level(&ClassName::Fighter).unwrap().subclass(),
                Some(&SubclassName {
                    class: ClassName::Fighter,
                    name: "Champion".to_string()
                })
            );
        }

        {
            let effects = systems::effects::effects(&mut world, character);
            assert_eq!(effects.len(), 2);
            for effect_id in [
                &registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID,
                &registry::effects::IMPROVED_CRITICAL_ID,
            ] {
                assert!(
                    effects.contains(&registry::effects::EFFECT_REGISTRY.get(&effect_id).unwrap())
                );
            }
        }

        {
            let skills = systems::helpers::get_component::<SkillSet>(&mut world, character);
            for skill in [Skill::Athletics, Skill::Perception] {
                assert_eq!(skills.get(skill).proficiency(), &Proficiency::Proficient);
            }
        }

        {
            let saving_throws =
                systems::helpers::get_component::<SavingThrowSet>(&mut world, character);
            for ability in [Ability::Strength, Ability::Constitution] {
                assert_eq!(
                    saving_throws.get(ability).proficiency(),
                    &Proficiency::Proficient
                );
            }
        }
    }
}

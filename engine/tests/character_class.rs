extern crate nat20_rs;

mod tests {

    use std::collections::HashSet;

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::Ability,
            id::ItemId,
            level::CharacterLevels,
            level_up::ChoiceItem,
            proficiency::ProficiencyLevel,
            saving_throw::{SavingThrowKind, SavingThrowSet},
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
                // Level 1
                // TODO: Everyone is dragonborn for now
                LevelUpDecision::single_choice(ChoiceItem::Race(
                    registry::races::DRAGONBORN_ID.clone(),
                )),
                LevelUpDecision::single_choice(ChoiceItem::Subrace(
                    registry::races::DRAGONBORN_WHITE_ID.clone(),
                )),
                LevelUpDecision::single_choice(ChoiceItem::Background(
                    registry::backgrounds::SOLDIER_ID.clone(),
                )),
                LevelUpDecision::single_choice(ChoiceItem::Class(
                    registry::classes::FIGHTER_ID.clone(),
                )),
                LevelUpDecision::AbilityScores(
                    registry::classes::CLASS_REGISTRY
                        .get(&registry::classes::FIGHTER_ID)
                        .unwrap()
                        .default_abilities
                        .clone(),
                ),
                LevelUpDecision::single_choice_with_id(
                    "choice.fighting_style",
                    ChoiceItem::Feat(
                        registry::feats::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
                    ),
                ),
                LevelUpDecision::SkillProficiency(HashSet::from([
                    Skill::Acrobatics,
                    Skill::Perception,
                ])),
                LevelUpDecision::single_choice_with_id(
                    "choice.starting_equipment.fighter",
                    ChoiceItem::Equipment {
                        items: vec![
                            (1, ItemId::from_str("item.chainmail")),
                            (1, ItemId::from_str("item.greatsword")),
                            (1, ItemId::from_str("item.flail")),
                            (8, ItemId::from_str("item.javelin")),
                        ],
                        money: "4 GP".to_string(),
                    },
                ),
                LevelUpDecision::single_choice_with_id(
                    "choice.starting_equipment.soldier",
                    ChoiceItem::Equipment {
                        items: Vec::new(),
                        money: "50 GP".to_string(),
                    },
                ),
                // Level 2
                LevelUpDecision::single_choice(ChoiceItem::Class(
                    registry::classes::FIGHTER_ID.clone(),
                )),
                // Level 3
                LevelUpDecision::single_choice(ChoiceItem::Class(
                    registry::classes::FIGHTER_ID.clone(),
                )),
                LevelUpDecision::single_choice(ChoiceItem::Subclass(
                    registry::classes::CHAMPION_ID.clone(),
                )),
            ],
        );

        {
            let levels = systems::helpers::get_component::<CharacterLevels>(&mut world, character);
            assert_eq!(levels.total_level(), 3);
            assert_eq!(
                levels
                    .class_level(&registry::classes::FIGHTER_ID.clone())
                    .unwrap()
                    .level(),
                3
            );
            assert_eq!(
                levels
                    .class_level(&registry::classes::FIGHTER_ID.clone())
                    .unwrap()
                    .subclass(),
                Some(&registry::classes::CHAMPION_ID.clone())
            );
        }

        {
            let effects = systems::effects::effects(&mut world, character);
            assert_eq!(effects.len(), 6);
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
                assert_eq!(
                    skills.get(skill).proficiency().level(),
                    &ProficiencyLevel::Proficient
                );
            }
        }

        {
            let saving_throws =
                systems::helpers::get_component::<SavingThrowSet>(&mut world, character);
            for ability in [Ability::Strength, Ability::Constitution] {
                assert_eq!(
                    saving_throws
                        .get(SavingThrowKind::Ability(ability))
                        .proficiency()
                        .level(),
                    &ProficiencyLevel::Proficient
                );
            }
        }
    }
}

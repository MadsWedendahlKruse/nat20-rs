extern crate nat20_rs;

mod tests {

    use std::collections::HashSet;

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::Ability,
            id::{
                BackgroundId, ClassId, EffectId, FeatId, ItemId, SpeciesId, SubclassId,
                SubspeciesId,
            },
            level::CharacterLevels,
            level_up::ChoiceItem,
            proficiency::ProficiencyLevel,
            saving_throw::{SavingThrowKind, SavingThrowSet},
            skill::{Skill, SkillSet},
        },
        entities::character::Character,
        registry::registry::ClassesRegistry,
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
                LevelUpDecision::single_choice(ChoiceItem::Species(SpeciesId::from_str(
                    "species.dragonborn",
                ))),
                LevelUpDecision::single_choice(ChoiceItem::Subspecies(SubspeciesId::from_str(
                    "subspecies.dragonborn.white",
                ))),
                LevelUpDecision::single_choice(ChoiceItem::Background(BackgroundId::from_str(
                    "background.soldier",
                ))),
                LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                    "class.fighter",
                ))),
                LevelUpDecision::AbilityScores(
                    ClassesRegistry::get(&ClassId::from_str("class.fighter"))
                        .unwrap()
                        .default_abilities
                        .clone(),
                ),
                LevelUpDecision::single_choice_with_id(
                    "choice.fighting_style",
                    ChoiceItem::Feat(FeatId::from_str(
                        "feat.fighting_style.great_weapon_fighting",
                    )),
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
                LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                    "class.fighter",
                ))),
                // Level 3
                LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                    "class.fighter",
                ))),
                LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassId::from_str(
                    "subclass.fighter.champion",
                ))),
            ],
        );

        {
            let levels = systems::helpers::get_component::<CharacterLevels>(&mut world, character);
            assert_eq!(levels.total_level(), 3);
            assert_eq!(
                levels
                    .class_level(&ClassId::from_str("class.fighter"))
                    .unwrap()
                    .level(),
                3
            );
            assert_eq!(
                levels
                    .class_level(&ClassId::from_str("class.fighter"))
                    .unwrap()
                    .subclass(),
                Some(&SubclassId::from_str("subclass.fighter.champion"))
            );
        }

        {
            let effects = systems::effects::effects(&mut world, character);
            let effect_ids: HashSet<&EffectId> = effects.iter().map(|e| e.id()).collect();
            for effect_id in [
                EffectId::from_str("effect.fighting_style.great_weapon_fighting"),
                EffectId::from_str("effect.fighter.champion.improved_critical"),
            ] {
                assert!(
                    effect_ids.contains(&effect_id),
                    "Effect {:?} not found in character effects: {:?}",
                    effect_id,
                    effect_ids
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

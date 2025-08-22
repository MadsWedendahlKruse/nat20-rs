pub mod equipment {
    use crate::components::{
        id::ItemId,
        items::{
            equipment::equipment::{EquipmentItem, EquipmentKind},
            item::{Item, ItemRarity},
            money::MonetaryValue,
        },
    };

    pub fn boots() -> EquipmentItem {
        EquipmentItem {
            item: Item {
                id: ItemId::from_str("item.boots"),
                name: "Boots".to_string(),
                description: "A test pair of boots.".to_string(),
                weight: 1.8,
                value: MonetaryValue::from("10 GP"),
                rarity: ItemRarity::Common,
            },
            kind: EquipmentKind::Boots,
            effects: Vec::new(),
        }
    }

    pub fn gloves() -> EquipmentItem {
        EquipmentItem {
            item: Item {
                id: ItemId::from_str("item.gloves"),
                name: "Gloves".to_string(),
                description: "A test pair of gloves.".to_string(),
                weight: 0.5,
                value: MonetaryValue::from("5 GP"),
                rarity: ItemRarity::Common,
            },
            kind: EquipmentKind::Gloves,
            effects: Vec::new(),
        }
    }
}

pub mod creatures {

    use hecs::{Entity, World};

    use crate::{
        components::{ability::Ability, modifier::ModifierSource, skill::Skill},
        systems::{self, level_up::LevelUpDecision},
    };

    pub mod heroes {
        use std::collections::{HashMap, HashSet};

        use crate::{
            components::{
                class::{ClassName, SubclassName},
                id::{EntityIdentifier, Name},
                level_up::ChoiceItem,
                skill::SkillSet,
                spells::spellbook::Spellbook,
            },
            entities::character::Character,
            registry,
        };

        use super::*;

        pub fn add_initiative(world: &mut World, entity: Entity) {
            systems::helpers::get_component_mut::<SkillSet>(world, entity).add_modifier(
                Skill::Initiative,
                ModifierSource::Custom("Admin testing".to_string()),
                20,
            );
        }

        // TODO: Should spawn an Entity in a World instead of returning a Character
        pub fn fighter(world: &mut World) -> EntityIdentifier {
            let name = Name::new("Johnny Fighter");
            let character = Character::new(name.clone());
            let entity = world.spawn(character);
            systems::level_up::apply_level_up_decision(
                world,
                entity,
                5,
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
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Fighter)),
                    LevelUpDecision::AbilityScores(
                        registry::classes::CLASS_REGISTRY
                            .get(&ClassName::Fighter)
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
                                (1, registry::items::CHAINMAIL_ID.clone()),
                                (1, registry::items::GREATSWORD_ID.clone()),
                                (1, registry::items::FLAIL_ID.clone()),
                                (8, registry::items::JAVELIN_ID.clone()),
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
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Fighter)),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Fighter)),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassName {
                        class: ClassName::Fighter,
                        name: "Champion".to_string(),
                    })),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Fighter)),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(
                        registry::feats::ABILITY_SCORE_IMPROVEMENT_ID.clone(),
                    )),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Strength,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Fighter)),
                ],
            );

            EntityIdentifier::new(entity, name)
        }

        pub fn wizard(world: &mut World) -> EntityIdentifier {
            let name = Name::new("Jimmy Wizard");
            let character = Character::new(name.clone());
            let entity = world.spawn(character);
            systems::level_up::apply_level_up_decision(
                world,
                entity,
                5,
                vec![
                    // Level 1
                    // TODO: Everyone is dragonborn for now
                    LevelUpDecision::single_choice(ChoiceItem::Race(
                        registry::races::DRAGONBORN_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Subrace(
                        registry::races::DRAGONBORN_RED_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Background(
                        registry::backgrounds::SAGE_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Wizard)),
                    LevelUpDecision::AbilityScores(
                        registry::classes::CLASS_REGISTRY
                            .get(&ClassName::Wizard)
                            .unwrap()
                            .default_abilities
                            .clone(),
                    ),
                    LevelUpDecision::SkillProficiency(HashSet::from([
                        Skill::Investigation,
                        Skill::Insight,
                    ])),
                    LevelUpDecision::single_choice_with_id(
                        "choice.starting_equipment.sage",
                        ChoiceItem::Equipment {
                            items: vec![
                                (1, registry::items::QUARTERSTAFF_ID.clone()),
                                (1, registry::items::ROBE_ID.clone()),
                            ],
                            money: "8 GP".to_string(),
                        },
                    ),
                    // Level 2
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Wizard)),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Wizard)),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassName {
                        class: ClassName::Wizard,
                        name: "Evoker".to_string(),
                    })),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Wizard)),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(
                        registry::feats::ABILITY_SCORE_IMPROVEMENT_ID.clone(),
                    )),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Intelligence,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Wizard)),
                ],
            );

            // TODO: Spellcasting ability should be set automatically based on class
            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            // TODO: This should be set automatically based on class
            spellbook.set_max_prepared_spells(5);
            spellbook.add_spell(&registry::spells::MAGIC_MISSILE_ID, Ability::Intelligence);
            spellbook.add_spell(&registry::spells::FIREBALL_ID, Ability::Intelligence);
            spellbook.add_spell(&registry::spells::COUNTERSPELL_ID, Ability::Intelligence);

            EntityIdentifier::new(entity, name)
        }

        pub fn warlock(world: &mut World) -> EntityIdentifier {
            let name = Name::new("Bobby Warlock");
            let character = Character::new(name.clone());
            let entity = world.spawn(character);
            systems::level_up::apply_level_up_decision(
                world,
                entity,
                5,
                vec![
                    // Level 1
                    // TODO: Everyone is dragonborn for now
                    LevelUpDecision::single_choice(ChoiceItem::Race(
                        registry::races::DRAGONBORN_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Subrace(
                        registry::races::DRAGONBORN_BLACK_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Background(
                        registry::backgrounds::ACOLYTE_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Warlock)),
                    LevelUpDecision::AbilityScores(
                        registry::classes::CLASS_REGISTRY
                            .get(&ClassName::Warlock)
                            .unwrap()
                            .default_abilities
                            .clone(),
                    ),
                    LevelUpDecision::SkillProficiency(HashSet::from([
                        Skill::Arcana,
                        Skill::Deception,
                    ])),
                    LevelUpDecision::single_choice_with_id(
                        "choice.starting_equipment.acolyte",
                        ChoiceItem::Equipment {
                            items: vec![(1, registry::items::ROBE_ID.clone())],
                            money: "8 GP".to_string(),
                        },
                    ),
                    // Level 2
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Warlock)),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Warlock)),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassName {
                        class: ClassName::Warlock,
                        name: "Fiend Patron".to_string(),
                    })),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Warlock)),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(
                        registry::feats::ABILITY_SCORE_IMPROVEMENT_ID.clone(),
                    )),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Charisma,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassName::Warlock)),
                ],
            );

            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.update_spell_slots(5);
            spellbook.add_spell(&registry::spells::ELDRITCH_BLAST_ID, Ability::Charisma);

            EntityIdentifier::new(entity, name)
        }
    }

    pub mod monsters {

        use crate::{
            components::{
                ability::AbilityScoreMap,
                hit_points::HitPoints,
                id::{EntityIdentifier, ItemId, Name},
                items::{
                    equipment::{
                        armor::ArmorTrainingSet, loadout::TryEquipError,
                        weapon::WeaponProficiencyMap,
                    },
                    inventory::ItemInstance,
                },
                level::ChallengeRating,
                proficiency::{Proficiency, ProficiencyLevel},
                race::{CreatureSize, CreatureType, Speed},
            },
            entities::monster::Monster,
            registry,
        };

        use super::*;

        pub fn goblin_warrior(world: &mut World) -> EntityIdentifier {
            let name = Name::new("Goblin Warrior");
            let monster = Monster::new(
                name.clone(),
                ChallengeRating::new(1),
                HitPoints::new(10),
                CreatureSize::Small,
                CreatureType::Fey,
                Speed(30),
                AbilityScoreMap::from([
                    (Ability::Strength, 10),
                    (Ability::Dexterity, 14),
                    (Ability::Constitution, 12),
                    (Ability::Intelligence, 8),
                    (Ability::Wisdom, 10),
                    (Ability::Charisma, 8),
                ]),
            );
            let entity = world.spawn(monster);
            let _ = monster_equipment(
                world,
                entity,
                &[
                    // TODO: Should be LEATHER_ARMOR_ID
                    registry::items::STUDDED_LEATHER_ARMOR_ID.clone(),
                    registry::items::SCIMITAR_ID.clone(),
                    // TODO: Add SHIELD_ID
                    registry::items::SHORTBOW_ID.clone(),
                ],
            );

            EntityIdentifier::new(entity, name)
        }

        fn monster_equipment(
            world: &mut World,
            entity: Entity,
            item_ids: &[ItemId],
        ) -> Result<(), TryEquipError> {
            for item_id in item_ids {
                let item = registry::items::ITEM_REGISTRY.get(item_id).unwrap().clone();
                // Monsters are considered proficient with all their equipment
                // so we can add proficiency for what they equip
                match &item {
                    ItemInstance::Armor(armor) => {
                        systems::helpers::get_component_mut::<ArmorTrainingSet>(world, entity)
                            .insert(armor.armor_type.clone());
                    }
                    ItemInstance::Weapon(weapon) => {
                        systems::helpers::get_component_mut::<WeaponProficiencyMap>(world, entity)
                            .set_proficiency(
                                weapon.category().clone(),
                                Proficiency::new(
                                    ProficiencyLevel::Proficient,
                                    ModifierSource::None,
                                ),
                            );
                    }
                    _ => {}
                }

                systems::loadout::equip(world, entity, item)?;
            }
            Ok(())
        }
    }
}

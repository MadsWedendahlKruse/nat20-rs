pub mod equipment {
    use std::str::FromStr;

    use uom::si::{f32::Mass, mass::pound};

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
                weight: Mass::new::<pound>(1.8),
                value: MonetaryValue::from_str("10 GP").unwrap(),
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
                weight: Mass::new::<pound>(0.5),
                value: MonetaryValue::from_str("5 GP").unwrap(),
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
                id::{
                    BackgroundId, ClassId, EntityIdentifier, FeatId, ItemId, Name, SpellId,
                    SubclassId,
                },
                level_up::ChoiceItem,
                modifier::KeyedModifiable,
                skill::SkillSet,
                spells::spellbook::Spellbook,
            },
            entities::character::Character,
            registry::{
                self,
                registry::{ClassesRegistry, ItemsRegistry},
            },
        };

        use super::*;

        pub fn add_initiative(world: &mut World, entity: Entity) {
            systems::helpers::get_component_mut::<SkillSet>(world, entity).add_modifier(
                Skill::Initiative,
                ModifierSource::Custom("Admin testing".to_string()),
                20,
            );
        }

        pub fn fighter(world: &mut World) -> EntityIdentifier {
            let name = Name::new("Johnny Fighter");
            let character = Character::new(name.clone());
            let entity = world.spawn(character);
            systems::level_up::apply_level_up_decision(
                world,
                entity,
                9,
                vec![
                    // Level 1
                    // TODO: Everyone is dragonborn for now
                    LevelUpDecision::single_choice(ChoiceItem::Race(
                        registry::races::DRAGONBORN_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Subrace(
                        registry::races::DRAGONBORN_WHITE_ID.clone(),
                    )),
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
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::from_str(
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Strength,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.fighter",
                    ))),
                    // Level 6
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::from_str(
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([
                        (Ability::Strength, 1),
                        (Ability::Dexterity, 1),
                    ])),
                    // Level 7
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice_with_id(
                        "choice.fighting_style",
                        ChoiceItem::Feat(FeatId::from_str("feat.fighting_style.defense")),
                    ),
                    // Level 8
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::from_str(
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Dexterity,
                        2,
                    )])),
                    // Level 9
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.fighter",
                    ))),
                ],
            );

            let _ = systems::loadout::equip(
                world,
                entity,
                ItemsRegistry::get(&ItemId::from_str("item.crossbow"))
                    .unwrap()
                    .clone(),
            );

            let _ = systems::inventory::add_item(
                world,
                entity,
                ItemsRegistry::get(&ItemId::from_str("item.admin_dagger"))
                    .unwrap()
                    .clone(),
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
                    LevelUpDecision::single_choice(ChoiceItem::Background(BackgroundId::from_str(
                        "background.sage",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.wizard",
                    ))),
                    LevelUpDecision::AbilityScores(
                        ClassesRegistry::get(&ClassId::from_str("class.wizard"))
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
                                (1, ItemId::from_str("item.quarterstaff")),
                                (1, ItemId::from_str("item.robe")),
                            ],
                            money: "8 GP".to_string(),
                        },
                    ),
                    // Level 2
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.wizard",
                    ))),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.wizard",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassId::from_str(
                        "subclass.wizard.evoker",
                    ))),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.wizard",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::from_str(
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Intelligence,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.wizard",
                    ))),
                ],
            );

            // TODO: Spellcasting ability should be set automatically based on class
            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            // TODO: This should be set automatically based on class
            spellbook.set_max_prepared_spells(5);
            spellbook.add_spell(
                &SpellId::from_str("spell.magic_missile"),
                Ability::Intelligence,
            );
            spellbook.add_spell(&SpellId::from_str("spell.fireball"), Ability::Intelligence);
            spellbook.add_spell(
                &SpellId::from_str("spell.counterspell"),
                Ability::Intelligence,
            );

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
                    LevelUpDecision::single_choice(ChoiceItem::Background(BackgroundId::from_str(
                        "background.acolyte",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.warlock",
                    ))),
                    LevelUpDecision::AbilityScores(
                        ClassesRegistry::get(&ClassId::from_str("class.warlock"))
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
                            items: vec![(1, ItemId::from_str("item.robe"))],
                            money: "8 GP".to_string(),
                        },
                    ),
                    // Level 2
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.warlock",
                    ))),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.warlock",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassId::from_str(
                        "subclass.warlock.fiend_patron",
                    ))),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.warlock",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::from_str(
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Charisma,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::from_str(
                        "class.warlock",
                    ))),
                ],
            );

            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.add_spell(
                &SpellId::from_str("spell.eldritch_blast"),
                Ability::Charisma,
            );

            EntityIdentifier::new(entity, name)
        }
    }

    pub mod monsters {

        use uom::si::{f32::Length, length::foot};

        use crate::{
            components::{
                ability::AbilityScoreMap,
                faction::FactionSet,
                health::hit_points::HitPoints,
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
                race::{CreatureSize, CreatureType},
                speed::Speed,
            },
            entities::monster::Monster,
            registry::{self, registry::ItemsRegistry},
        };

        use super::*;

        pub fn goblin_warrior(world: &mut World) -> EntityIdentifier {
            let name = Name::new("Goblin Warrior");
            let monster = Monster::new(
                name.clone(),
                registry::ai::RANDOM_CONTROLLER_ID.clone(),
                ChallengeRating::new(1),
                HitPoints::new(10),
                CreatureSize::Small,
                CreatureType::Fey,
                Speed::new(Length::new::<foot>(30.0)),
                AbilityScoreMap::from([
                    (Ability::Strength, 10),
                    (Ability::Dexterity, 14),
                    (Ability::Constitution, 12),
                    (Ability::Intelligence, 8),
                    (Ability::Wisdom, 10),
                    (Ability::Charisma, 8),
                ]),
                FactionSet::from([registry::factions::GOBLINS_ID.clone()]),
            );
            let entity = world.spawn(monster);
            let _ = monster_equipment(
                world,
                entity,
                &[
                    // TODO: Should be LEATHER_ARMOR_ID
                    ItemId::from_str("item.studded_leather_armor"),
                    ItemId::from_str("item.scimitar"),
                    // TODO: Add SHIELD_ID
                    ItemId::from_str("item.shortbow"),
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
                let item = ItemsRegistry::get(item_id).unwrap().clone();
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

pub mod engine {
    use rerecast::ConfigBuilder;

    use crate::engine::{game_state::GameState, geometry::WorldGeometry};

    pub fn game_state() -> GameState {
        GameState::new(WorldGeometry::from_obj_path(
            "../assets/models/geometry/test_terrain.obj",
            &ConfigBuilder::default().build(),
        ))
    }
}

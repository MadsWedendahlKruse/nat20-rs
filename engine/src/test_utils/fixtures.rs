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
                id: ItemId::new("nat20_rs", "item.boots"),
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
                id: ItemId::new("nat20_rs", "item.gloves"),
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
                class::ClassAndSubclass,
                id::{
                    BackgroundId, ClassId, EntityIdentifier, FeatId, ItemId, Name, SpeciesId,
                    SpellId, SubclassId, SubspeciesId,
                },
                level_up::ChoiceItem,
                modifier::KeyedModifiable,
                skill::SkillSet,
                spells::spellbook::{SpellSource, Spellbook},
            },
            entities::character::Character,
            registry::registry::{ClassesRegistry, ItemsRegistry},
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
                    LevelUpDecision::single_choice(ChoiceItem::Species(SpeciesId::new(
                        "nat20_rs",
                        "species.dragonborn",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subspecies(SubspeciesId::new(
                        "nat20_rs",
                        "subspecies.dragonborn.white",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Background(BackgroundId::new(
                        "nat20_rs",
                        "background.soldier",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    LevelUpDecision::AbilityScores(
                        ClassesRegistry::get(&ClassId::new("nat20_rs", "class.fighter"))
                            .unwrap()
                            .default_abilities
                            .clone(),
                    ),
                    LevelUpDecision::single_choice_with_id(
                        "choice.fighting_style",
                        ChoiceItem::Feat(FeatId::new(
                            "nat20_rs",
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
                                (1, ItemId::new("nat20_rs", "item.chainmail")),
                                (1, ItemId::new("nat20_rs", "item.greatsword")),
                                (1, ItemId::new("nat20_rs", "item.flail")),
                                (8, ItemId::new("nat20_rs", "item.javelin")),
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
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassId::new(
                        "nat20_rs",
                        "subclass.fighter.champion",
                    ))),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::new(
                        "nat20_rs",
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Strength,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    // Level 6
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::new(
                        "nat20_rs",
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([
                        (Ability::Strength, 1),
                        (Ability::Dexterity, 1),
                    ])),
                    // Level 7
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice_with_id(
                        "choice.fighting_style",
                        ChoiceItem::Feat(FeatId::new("nat20_rs", "feat.fighting_style.defense")),
                    ),
                    // Level 8
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::new(
                        "nat20_rs",
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Dexterity,
                        2,
                    )])),
                    // Level 9
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.fighter",
                    ))),
                ],
            );

            let _ = systems::loadout::equip(
                world,
                entity,
                ItemsRegistry::get(&ItemId::new("nat20_rs", "item.crossbow"))
                    .unwrap()
                    .clone(),
            );

            let _ = systems::inventory::add_item(
                world,
                entity,
                ItemsRegistry::get(&ItemId::new("nat20_rs", "item.admin_dagger"))
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
                    LevelUpDecision::single_choice(ChoiceItem::Species(SpeciesId::new(
                        "nat20_rs",
                        "species.dragonborn",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subspecies(SubspeciesId::new(
                        "nat20_rs",
                        "subspecies.dragonborn.red",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Background(BackgroundId::new(
                        "nat20_rs",
                        "background.sage",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.wizard",
                    ))),
                    LevelUpDecision::AbilityScores(
                        ClassesRegistry::get(&ClassId::new("nat20_rs", "class.wizard"))
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
                                (1, ItemId::new("nat20_rs", "item.quarterstaff")),
                                (1, ItemId::new("nat20_rs", "item.robe")),
                            ],
                            money: "8 GP".to_string(),
                        },
                    ),
                    LevelUpDecision::spells(
                        "choice.cantrips",
                        &ClassId::new("nat20_rs", "class.wizard"),
                        &None,
                        vec![
                            SpellId::new("nat20_rs", "spell.fire_bolt"),
                            SpellId::new("nat20_rs", "spell.acid_splash"),
                            SpellId::new("nat20_rs", "spell.poison_spray"),
                        ],
                    ),
                    LevelUpDecision::spells(
                        "choice.spells",
                        &ClassId::new("nat20_rs", "class.wizard"),
                        &None,
                        vec![
                            SpellId::new("nat20_rs", "spell.magic_missile"),
                            SpellId::new("nat20_rs", "spell.expeditious_retreat"),
                        ],
                    ),
                    // Level 2
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.wizard",
                    ))),
                    LevelUpDecision::spells(
                        "choice.spells",
                        &ClassId::new("nat20_rs", "class.wizard"),
                        &None,
                        vec![SpellId::new("nat20_rs", "spell.shield")],
                    ),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.wizard",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassId::new(
                        "nat20_rs",
                        "subclass.wizard.evoker",
                    ))),
                    LevelUpDecision::spells(
                        "choice.spells",
                        &ClassId::new("nat20_rs", "class.wizard"),
                        &None,
                        vec![SpellId::new("nat20_rs", "spell.scorching_ray")],
                    ),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.wizard",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::new(
                        "nat20_rs",
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Intelligence,
                        2,
                    )])),
                    LevelUpDecision::spells(
                        "choice.cantrips",
                        &ClassId::new("nat20_rs", "class.wizard"),
                        &None,
                        vec![SpellId::new("nat20_rs", "spell.ray_of_frost")],
                    ),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.wizard",
                    ))),
                    LevelUpDecision::spells(
                        "choice.spells",
                        &ClassId::new("nat20_rs", "class.wizard"),
                        &None,
                        vec![
                            SpellId::new("nat20_rs", "spell.fireball"),
                            SpellId::new("nat20_rs", "spell.counterspell"),
                        ],
                    ),
                ],
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
                    LevelUpDecision::single_choice(ChoiceItem::Species(SpeciesId::new(
                        "nat20_rs",
                        "species.dragonborn",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subspecies(SubspeciesId::new(
                        "nat20_rs",
                        "subspecies.dragonborn.black",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Background(BackgroundId::new(
                        "nat20_rs",
                        "background.acolyte",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.warlock",
                    ))),
                    LevelUpDecision::AbilityScores(
                        ClassesRegistry::get(&ClassId::new("nat20_rs", "class.warlock"))
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
                            items: vec![(1, ItemId::new("nat20_rs", "item.robe"))],
                            money: "8 GP".to_string(),
                        },
                    ),
                    LevelUpDecision::spells(
                        "choice.cantrips",
                        &ClassId::new("nat20_rs", "class.warlock"),
                        &None,
                        vec![
                            SpellId::new("nat20_rs", "spell.eldritch_blast"),
                            SpellId::new("nat20_rs", "spell.poison_spray"),
                        ],
                    ),
                    // LevelUpDecision::spells(
                    //     "choice.spells",
                    //     &ClassId::new("nat20_rs", "class.warlock"),
                    //     &None,
                    //     vec![SpellId::new("nat20_rs", "spell.magic_missile")],
                    // ),
                    // Level 2
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.warlock",
                    ))),
                    // Level 3
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.warlock",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Subclass(SubclassId::new(
                        "nat20_rs",
                        "subclass.warlock.fiend_patron",
                    ))),
                    // Level 4
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.warlock",
                    ))),
                    LevelUpDecision::single_choice(ChoiceItem::Feat(FeatId::new(
                        "nat20_rs",
                        "feat.ability_score_improvement",
                    ))),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Charisma,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::single_choice(ChoiceItem::Class(ClassId::new(
                        "nat20_rs",
                        "class.warlock",
                    ))),
                ],
            );

            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            let _ = spellbook.add_spell(
                &SpellId::new("nat20_rs", "spell.eldritch_blast"),
                &SpellSource::Class(ClassAndSubclass {
                    class: ClassId::new("nat20_rs", "class.warlock"),
                    subclass: None,
                }),
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
                id::{EntityIdentifier, FactionId, ItemId, Name},
                items::{
                    equipment::{
                        armor::ArmorTrainingSet, loadout::TryEquipError,
                        weapon::WeaponProficiencyMap,
                    },
                    inventory::ItemInstance,
                },
                level::ChallengeRating,
                proficiency::{Proficiency, ProficiencyLevel},
                species::{CreatureSize, CreatureType},
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
                FactionSet::from([FactionId::new("nat20_rs", "faction.goblins")]),
            );
            let entity = world.spawn(monster);
            let _ = monster_equipment(
                world,
                entity,
                &[
                    // TODO: Should be LEATHER_ARMOR_ID
                    ItemId::new("nat20_rs", "item.studded_leather_armor"),
                    ItemId::new("nat20_rs", "item.scimitar"),
                    // TODO: Add SHIELD_ID
                    ItemId::new("nat20_rs", "item.shortbow"),
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

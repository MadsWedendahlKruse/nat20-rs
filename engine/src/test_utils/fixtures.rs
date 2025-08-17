pub mod armor {
    use crate::components::{
        id::ItemId,
        items::{
            equipment::{
                armor::Armor,
                equipment::{EquipmentItem, EquipmentKind},
            },
            item::{Item, ItemRarity},
        },
    };

    pub fn clothing() -> Armor {
        Armor::clothing(
            Item {
                id: ItemId::from_str("item.clothing"),
                name: "Clothing".to_string(),
                description: "A simple set of clothing.".to_string(),
                weight: 1.0,
                value: 15,
                rarity: ItemRarity::Common,
            },
            Vec::new(),
        )
    }

    pub fn light_armor() -> Armor {
        Armor::light(
            Item {
                id: ItemId::from_str("item.light_armor"),
                name: "Leather Armor".to_string(),
                description: "A test light armor item.".to_string(),
                weight: 5.0,
                value: 30,
                rarity: ItemRarity::Common,
            },
            12,
            Vec::new(),
        )
    }

    pub fn medium_armor() -> Armor {
        Armor::medium(
            Item {
                id: ItemId::from_str("item.medium_armor"),
                name: "Chain Shirt".to_string(),
                description: "A test medium armor item.".to_string(),
                weight: 20.0,
                value: 50,
                rarity: ItemRarity::Common,
            },
            14,
            false,
            Vec::new(),
        )
    }

    pub fn heavy_armor() -> Armor {
        Armor::heavy(
            Item {
                id: ItemId::from_str("item.heavy_armor"),
                name: "Plate Armor".to_string(),
                description: "A test heavy armor item.".to_string(),
                weight: 65.0,
                value: 1500,
                rarity: ItemRarity::Common,
            },
            18,
            Vec::new(),
        )
    }
}

pub mod weapons {
    use std::collections::HashSet;

    use crate::components::{
        damage::DamageType,
        dice::{DiceSet, DieSize},
        id::ItemId,
        items::{
            equipment::{
                equipment::{EquipmentItem, EquipmentKind},
                weapon::{Weapon, WeaponCategory, WeaponKind, WeaponProperties},
            },
            item::{Item, ItemRarity},
        },
    };

    pub fn dagger_light() -> Weapon {
        Weapon::new(
            Item {
                id: ItemId::from_str("item.dagger_light"),
                name: "Dagger".to_string(),
                description: "A test light dagger.".to_string(),
                weight: 1.0,
                value: 2,
                rarity: ItemRarity::Common,
            },
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Light]),
            vec![(1, DieSize::D4, DamageType::Piercing)],
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn rapier_finesse() -> Weapon {
        Weapon::new(
            Item {
                id: ItemId::from_str("item.rapier_finesse"),
                name: "Rapier".to_string(),
                description: "A test rapier with finesse.".to_string(),
                weight: 1.0,
                value: 25,
                rarity: ItemRarity::Common,
            },
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn trident_versatile() -> Weapon {
        let dice_set_two_handed = DiceSet {
            num_dice: 1,
            die_size: DieSize::D8,
        };
        Weapon::new(
            Item {
                id: ItemId::from_str("item.trident_versatile"),
                name: "Trident".to_string(),
                description: "A versatile trident.".to_string(),
                weight: 4.0,
                value: 5,
                rarity: ItemRarity::Common,
            },
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Versatile(dice_set_two_handed)]),
            vec![(1, DieSize::D6, DamageType::Piercing)],
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn greatsword_two_handed() -> Weapon {
        Weapon::new(
            Item {
                id: ItemId::from_str("item.greatsword_two_handed"),
                name: "Greatsword".to_string(),
                description: "A two-handed greatsword.".to_string(),
                weight: 6.0,
                value: 50,
                rarity: ItemRarity::Common,
            },
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::TwoHanded]),
            vec![(2, DieSize::D6, DamageType::Slashing)],
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn longbow() -> Weapon {
        Weapon::new(
            Item {
                id: ItemId::from_str("item.longbow"),
                name: "Longbow".to_string(),
                description: "A longbow with a range of 10/40 feet.".to_string(),
                weight: 2.0,
                value: 50,
                rarity: ItemRarity::Common,
            },
            WeaponKind::Ranged,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Range(10, 40), WeaponProperties::TwoHanded]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn greatsword_flaming() -> Weapon {
        Weapon::new(
            Item {
                id: ItemId::from_str("item.greatsword_flaming"),
                name: "Flaming Greatsword".to_string(),
                description: "A magical greatsword that deals fire damage.".to_string(),
                weight: 6.0,
                value: 1000,
                rarity: ItemRarity::Rare,
            },
            WeaponKind::Melee,
            WeaponCategory::Martial,
            HashSet::from([
                WeaponProperties::TwoHanded,
                WeaponProperties::Enchantment(1),
            ]),
            vec![
                (2, DieSize::D6, DamageType::Slashing),
                (1, DieSize::D4, DamageType::Fire),
            ],
            Vec::new(),
            Vec::new(),
        )
    }
}

pub mod equipment {
    use crate::components::{
        id::ItemId,
        items::{
            equipment::equipment::{EquipmentItem, EquipmentKind},
            item::{Item, ItemRarity},
        },
    };

    pub fn boots() -> EquipmentItem {
        EquipmentItem {
            item: Item {
                id: ItemId::from_str("item.boots"),
                name: "Boots".to_string(),
                description: "A test pair of boots.".to_string(),
                weight: 1.8,
                value: 11,
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
                value: 5,
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
                id::EntityIdentifier,
                level_up::ChoiceItem,
                skill::SkillSet,
                spells::spellbook::Spellbook,
            },
            entities::character::Character,
            registry,
            test_utils::fixtures,
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
            let name = "Johnny Fighter";
            let character = Character::new(name);
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

            let _ = systems::loadout::equip(world, entity, fixtures::armor::heavy_armor());
            let _ = systems::loadout::equip(world, entity, fixtures::weapons::greatsword_flaming());

            EntityIdentifier::new(entity, name)
        }

        pub fn wizard(world: &mut World) -> EntityIdentifier {
            let name = "Jimmy Wizard";
            let character = Character::new(name);
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

            let _ = systems::loadout::equip(world, entity, fixtures::armor::clothing());

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
            let name = "Bobby Warlock";
            let character = Character::new(name);
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

            let _ = systems::loadout::equip(world, entity, fixtures::armor::clothing());

            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.update_spell_slots(5);
            spellbook.add_spell(&registry::spells::ELDRITCH_BLAST_ID, Ability::Charisma);

            EntityIdentifier::new(entity, name)
        }
    }

    pub mod monsters {
        use std::collections::HashSet;

        use crate::{
            components::{class::ClassName, id::EntityIdentifier, level_up::ChoiceItem},
            entities::character::Character,
            registry,
            test_utils::fixtures,
        };

        use super::*;

        pub fn goblin_warrior(world: &mut World) -> EntityIdentifier {
            let name = "Goblin Warrior";
            let character = Character::new(name);
            let entity = world.spawn(character);
            // TODO: Not sure how to handle monster level-ups yet
            systems::level_up::apply_level_up_decision(
                world,
                entity,
                1,
                vec![
                    // Level 1
                    // TODO: Everyone is dragonborn for now (even the goblins)
                    LevelUpDecision::single_choice(ChoiceItem::Race(
                        registry::races::DRAGONBORN_ID.clone(),
                    )),
                    LevelUpDecision::single_choice(ChoiceItem::Subrace(
                        registry::races::DRAGONBORN_GREEN_ID.clone(),
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
                        Skill::Survival,
                    ])),
                ],
            );

            let _ = systems::loadout::equip(world, entity, fixtures::armor::medium_armor());
            let _ = systems::loadout::equip(world, entity, fixtures::weapons::dagger_light());

            EntityIdentifier::new(entity, name)
        }
    }
}

pub mod armor {
    use crate::components::items::{
        equipment::{
            armor::Armor,
            equipment::{EquipmentItem, EquipmentType},
        },
        item::ItemRarity,
    };

    pub fn clothing() -> Armor {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Clothes".to_string(),
            "A test clothing item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        Armor::clothing(equipment)
    }

    pub fn light_armor() -> Armor {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Leather Armor".to_string(),
            "A test light armor item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        Armor::light(equipment, 12)
    }

    pub fn medium_armor() -> Armor {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Chain Shirt".to_string(),
            "A test medium armor item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        Armor::medium(equipment, 14, false)
    }

    pub fn heavy_armor() -> Armor {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Plate Armor".to_string(),
            "A test heavy armor item.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Armor,
        );
        Armor::heavy(equipment, 18)
    }
}

pub mod weapons {
    use std::collections::HashSet;

    use crate::components::{
        damage::DamageType,
        dice::{DiceSet, DieSize},
        items::{
            equipment::{
                equipment::{EquipmentItem, EquipmentType},
                weapon::{Weapon, WeaponCategory, WeaponProperties},
            },
            item::ItemRarity,
        },
    };

    pub fn dagger_light() -> Weapon {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Dagger".to_string(),
            "A test dagger.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Light]),
            vec![(1, DieSize::D4, DamageType::Piercing)],
            vec![],
        )
    }

    pub fn rapier_finesse() -> Weapon {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Rapier".to_string(),
            "A test rapier.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
        )
    }

    pub fn trident_versatile() -> Weapon {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Trident".to_string(),
            "A test trident.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let dice_set_two_handed = DiceSet {
            num_dice: 1,
            die_size: DieSize::D8,
        };
        Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Versatile(dice_set_two_handed)]),
            vec![(1, DieSize::D6, DamageType::Piercing)],
            vec![],
        )
    }

    pub fn greatsword_two_handed() -> Weapon {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Greatsword".to_string(),
            "A test greatsword.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::TwoHanded]),
            vec![(2, DieSize::D6, DamageType::Slashing)],
            vec![],
        )
    }

    pub fn longbow() -> Weapon {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Longbow".to_string(),
            "A test longbow.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::RangedWeapon,
        );
        Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Range(10, 40)]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
        )
    }

    pub fn greatsword_flaming() -> Weapon {
        let equipment: EquipmentItem = EquipmentItem::new(
            "Flaming Greatsword".to_string(),
            "A test flaming greatsword.".to_string(),
            1.8,
            11,
            ItemRarity::Rare,
            EquipmentType::MeleeWeapon,
        );
        Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([
                WeaponProperties::TwoHanded,
                WeaponProperties::Enchantment(1),
            ]),
            vec![
                (2, DieSize::D6, DamageType::Slashing),
                (1, DieSize::D4, DamageType::Fire),
            ],
            vec![],
        )
    }
}

pub mod equipment {
    use crate::components::items::{
        equipment::equipment::{EquipmentItem, EquipmentType},
        item::ItemRarity,
    };

    pub fn boots() -> EquipmentItem {
        EquipmentItem::new(
            "Boots".to_string(),
            "A test pair of boots.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Boots,
        )
    }

    pub fn gloves() -> EquipmentItem {
        EquipmentItem::new(
            "Gloves".to_string(),
            "A test pair of gloves.".to_string(),
            1.8,
            11,
            ItemRarity::Common,
            EquipmentType::Gloves,
        )
    }
}

pub mod creatures {

    use hecs::{Entity, World};

    use crate::{
        components::{
            ability::Ability, items::equipment::equipment::HandSlot, modifier::ModifierSource,
            skill::Skill,
        },
        systems::{self, level_up::LevelUpDecision},
    };

    pub mod heroes {
        use std::collections::{HashMap, HashSet};

        use crate::{
            components::{
                class::{ClassName, SubclassName},
                id::EntityIdentifier,
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
                    LevelUpDecision::Race(registry::races::DRAGONBORN_ID.clone()),
                    LevelUpDecision::Subrace(registry::races::DRAGONBORN_WHITE_ID.clone()),
                    LevelUpDecision::Background(registry::backgrounds::SOLDIER_ID.clone()),
                    LevelUpDecision::Class(ClassName::Fighter),
                    LevelUpDecision::AbilityScores(
                        registry::classes::CLASS_REGISTRY
                            .get(&ClassName::Fighter)
                            .unwrap()
                            .default_abilities
                            .clone(),
                    ),
                    LevelUpDecision::Feat(
                        registry::feats::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
                    ),
                    LevelUpDecision::SkillProficiency(HashSet::from([
                        Skill::Acrobatics,
                        Skill::Perception,
                    ])),
                    // Level 2
                    LevelUpDecision::Class(ClassName::Fighter),
                    // Level 3
                    LevelUpDecision::Class(ClassName::Fighter),
                    LevelUpDecision::Subclass(SubclassName {
                        class: ClassName::Fighter,
                        name: "Champion".to_string(),
                    }),
                    // Level 4
                    LevelUpDecision::Class(ClassName::Fighter),
                    LevelUpDecision::Feat(registry::feats::ABILITY_SCORE_IMPROVEMENT_ID.clone()),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Strength,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::Class(ClassName::Fighter),
                ],
            );

            systems::loadout::equip_armor(world, entity, fixtures::armor::heavy_armor());
            let _ = systems::loadout::equip_weapon(
                world,
                entity,
                fixtures::weapons::greatsword_flaming(),
                HandSlot::Main,
            );

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
                    LevelUpDecision::Race(registry::races::DRAGONBORN_ID.clone()),
                    LevelUpDecision::Subrace(registry::races::DRAGONBORN_RED_ID.clone()),
                    LevelUpDecision::Background(registry::backgrounds::SAGE_ID.clone()),
                    LevelUpDecision::Class(ClassName::Wizard),
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
                    LevelUpDecision::Class(ClassName::Wizard),
                    // Level 3
                    LevelUpDecision::Class(ClassName::Wizard),
                    LevelUpDecision::Subclass(SubclassName {
                        class: ClassName::Wizard,
                        name: "Evoker".to_string(),
                    }),
                    // Level 4
                    LevelUpDecision::Class(ClassName::Wizard),
                    LevelUpDecision::Feat(registry::feats::ABILITY_SCORE_IMPROVEMENT_ID.clone()),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Intelligence,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::Class(ClassName::Wizard),
                ],
            );

            systems::loadout::equip_armor(world, entity, fixtures::armor::clothing());

            // TODO: Spellcasting ability should be set automatically based on class
            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.add_spell(&registry::spells::MAGIC_MISSILE_ID, Ability::Intelligence);
            spellbook.add_spell(&registry::spells::FIREBALL_ID, Ability::Intelligence);
            spellbook.add_spell(&registry::spells::COUNTERSPELL_ID, Ability::Intelligence);
            // TODO: This should be set automatically based on class
            spellbook.set_max_prepared_spells(5);

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
                    LevelUpDecision::Race(registry::races::DRAGONBORN_ID.clone()),
                    LevelUpDecision::Subrace(registry::races::DRAGONBORN_BLACK_ID.clone()),
                    LevelUpDecision::Background(registry::backgrounds::ACOLYTE_ID.clone()),
                    LevelUpDecision::Class(ClassName::Warlock),
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
                    LevelUpDecision::Class(ClassName::Warlock),
                    // Level 3
                    LevelUpDecision::Class(ClassName::Warlock),
                    LevelUpDecision::Subclass(SubclassName {
                        class: ClassName::Warlock,
                        name: "Fiend Patron".to_string(),
                    }),
                    // Level 4
                    LevelUpDecision::Class(ClassName::Warlock),
                    LevelUpDecision::Feat(registry::feats::ABILITY_SCORE_IMPROVEMENT_ID.clone()),
                    LevelUpDecision::AbilityScoreImprovement(HashMap::from([(
                        Ability::Charisma,
                        2,
                    )])),
                    // Level 5
                    LevelUpDecision::Class(ClassName::Warlock),
                ],
            );

            systems::loadout::equip_armor(world, entity, fixtures::armor::clothing());

            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.update_spell_slots(5);
            spellbook.add_spell(&registry::spells::ELDRITCH_BLAST_ID, Ability::Charisma);

            EntityIdentifier::new(entity, name)
        }
    }

    pub mod monsters {
        use std::collections::HashSet;

        use crate::{
            components::{class::ClassName, id::EntityIdentifier},
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
                    LevelUpDecision::Race(registry::races::DRAGONBORN_ID.clone()),
                    LevelUpDecision::Subrace(registry::races::DRAGONBORN_GREEN_ID.clone()),
                    LevelUpDecision::Background(registry::backgrounds::SOLDIER_ID.clone()),
                    LevelUpDecision::Class(ClassName::Fighter),
                    LevelUpDecision::AbilityScores(
                        registry::classes::CLASS_REGISTRY
                            .get(&ClassName::Fighter)
                            .unwrap()
                            .default_abilities
                            .clone(),
                    ),
                    LevelUpDecision::Feat(
                        registry::feats::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
                    ),
                    LevelUpDecision::SkillProficiency(HashSet::from([
                        Skill::Acrobatics,
                        Skill::Survival,
                    ])),
                ],
            );

            systems::loadout::equip_armor(world, entity, fixtures::armor::medium_armor());
            let _ = systems::loadout::equip_weapon(
                world,
                entity,
                fixtures::weapons::dagger_light(),
                HandSlot::Main,
            );

            EntityIdentifier::new(entity, name)
        }
    }
}

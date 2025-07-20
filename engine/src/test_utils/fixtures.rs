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
    use std::collections::HashMap;

    use hecs::{Entity, World};

    use crate::{
        components::{
            ability::{Ability, AbilityScore, AbilityScoreSet},
            items::equipment::equipment::HandSlot,
            modifier::ModifierSource,
            skill::Skill,
        },
        systems::{
            self,
            level_up::{LevelUpSelection, LevelUpSession, PredefinedChoiceProvider},
        },
    };

    pub mod heroes {
        use std::collections::HashSet;

        use crate::{
            components::{
                class::{ClassName, SubclassName},
                id::CharacterId,
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
        pub fn fighter(world: &mut World) -> (Entity, CharacterId) {
            let character = Character::new("Johnny Fighter");
            let id = character.id.clone();
            let entity = world.spawn(character);
            apply_level_up_selection(
                world,
                entity,
                5,
                vec![
                    LevelUpSelection::Class(ClassName::Fighter),
                    LevelUpSelection::Effect(
                        registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
                    ),
                    LevelUpSelection::SkillProficiency(HashSet::from([
                        Skill::Athletics,
                        Skill::Perception,
                    ])),
                    LevelUpSelection::Class(ClassName::Fighter),
                    LevelUpSelection::Class(ClassName::Fighter),
                    LevelUpSelection::Subclass(SubclassName {
                        class: ClassName::Fighter,
                        name: "Champion".to_string(),
                    }),
                    LevelUpSelection::Class(ClassName::Fighter),
                    LevelUpSelection::Class(ClassName::Fighter),
                ],
            );

            let ability_scores = HashMap::from([
                (Ability::Strength, 17),
                (Ability::Dexterity, 14),
                (Ability::Constitution, 16),
                (Ability::Intelligence, 12),
                (Ability::Wisdom, 10),
                (Ability::Charisma, 8),
            ]);

            set_ability_scores(world, entity, ability_scores);

            systems::loadout::equip_armor(world, entity, fixtures::armor::heavy_armor());
            let _ = systems::loadout::equip_weapon(
                world,
                entity,
                fixtures::weapons::greatsword_flaming(),
                HandSlot::Main,
            );

            (entity, id)
        }

        pub fn wizard(world: &mut World) -> (Entity, CharacterId) {
            let character = Character::new("Jimmy Wizard");
            let id = character.id.clone();
            let entity = world.spawn(character);
            apply_level_up_selection(
                world,
                entity,
                5,
                vec![
                    LevelUpSelection::Class(ClassName::Wizard),
                    LevelUpSelection::SkillProficiency(HashSet::from([
                        Skill::Arcana,
                        Skill::History,
                    ])),
                    LevelUpSelection::Class(ClassName::Wizard),
                    LevelUpSelection::Class(ClassName::Wizard),
                    LevelUpSelection::Subclass(SubclassName {
                        class: ClassName::Wizard,
                        name: "Evoker".to_string(),
                    }),
                    LevelUpSelection::Class(ClassName::Wizard),
                    LevelUpSelection::Class(ClassName::Wizard),
                ],
            );

            let ability_scores = HashMap::from([
                (Ability::Strength, 8),
                (Ability::Dexterity, 14),
                (Ability::Constitution, 16),
                (Ability::Intelligence, 17),
                (Ability::Wisdom, 12),
                (Ability::Charisma, 10),
            ]);

            set_ability_scores(world, entity, ability_scores);

            systems::loadout::equip_armor(world, entity, fixtures::armor::clothing());

            // TODO: Spellcasting ability should be set automatically based on class
            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.add_spell(&registry::spells::MAGIC_MISSILE_ID, Ability::Intelligence);
            spellbook.add_spell(&registry::spells::FIREBALL_ID, Ability::Intelligence);
            // TODO: This should be set automatically based on class
            spellbook.set_max_prepared_spells(5);

            (entity, id)
        }

        pub fn warlock(world: &mut World) -> (Entity, CharacterId) {
            let character = Character::new("Bobby Warlock");
            let id = character.id.clone();
            let entity = world.spawn(character);
            apply_level_up_selection(
                world,
                entity,
                5,
                vec![
                    LevelUpSelection::Class(ClassName::Warlock),
                    LevelUpSelection::SkillProficiency(HashSet::from([
                        Skill::Arcana,
                        Skill::Deception,
                    ])),
                    LevelUpSelection::Class(ClassName::Warlock),
                    LevelUpSelection::Class(ClassName::Warlock),
                    LevelUpSelection::Subclass(SubclassName {
                        class: ClassName::Warlock,
                        name: "Fiend Patron".to_string(),
                    }),
                    LevelUpSelection::Class(ClassName::Warlock),
                    LevelUpSelection::Class(ClassName::Warlock),
                ],
            );

            let ability_scores = HashMap::from([
                (Ability::Strength, 8),
                (Ability::Dexterity, 14),
                (Ability::Constitution, 16),
                (Ability::Intelligence, 12),
                (Ability::Wisdom, 10),
                (Ability::Charisma, 17),
            ]);

            set_ability_scores(world, entity, ability_scores);

            systems::loadout::equip_armor(world, entity, fixtures::armor::clothing());

            let mut spellbook = systems::helpers::get_component_mut::<Spellbook>(world, entity);
            spellbook.update_spell_slots(5);
            spellbook.add_spell(&registry::spells::ELDRITCH_BLAST_ID, Ability::Charisma);

            (entity, id)
        }
    }

    pub mod monsters {
        use std::collections::HashSet;

        use crate::{
            components::{class::ClassName, id::CharacterId},
            entities::character::Character,
            registry,
            test_utils::fixtures,
        };

        use super::*;

        pub fn goblin_warrior(world: &mut World) -> (Entity, CharacterId) {
            let character = Character::new("Goblin Warrior");
            let id = character.id.clone();
            let entity = world.spawn(character);
            // TODO: Not sure how to handle monster level-ups yet
            apply_level_up_selection(
                world,
                entity,
                1,
                vec![
                    LevelUpSelection::Class(ClassName::Fighter),
                    LevelUpSelection::Effect(
                        registry::effects::FIGHTING_STYLE_GREAT_WEAPON_FIGHTING_ID.clone(),
                    ),
                    LevelUpSelection::SkillProficiency(HashSet::from([
                        Skill::Acrobatics,
                        Skill::Survival,
                    ])),
                ],
            );

            let ability_scores = HashMap::from([
                (Ability::Strength, 8),
                (Ability::Dexterity, 15),
                (Ability::Constitution, 10),
                (Ability::Intelligence, 10),
                (Ability::Wisdom, 8),
                (Ability::Charisma, 8),
            ]);

            set_ability_scores(world, entity, ability_scores);

            systems::loadout::equip_armor(world, entity, fixtures::armor::medium_armor());
            let _ = systems::loadout::equip_weapon(
                world,
                entity,
                fixtures::weapons::dagger_light(),
                HandSlot::Main,
            );

            (entity, id)
        }
    }

    fn apply_level_up_selection(
        world: &mut World,
        entity: Entity,
        levels: u8,
        responses: Vec<LevelUpSelection>,
    ) {
        // TODO: String is a bit generic here
        let name = systems::helpers::get_component::<String>(world, entity).to_string();
        let mut choice_provider = PredefinedChoiceProvider::new(name.clone(), responses);
        for level in 1..=levels {
            let mut level_up_session = LevelUpSession::new(entity);
            level_up_session
                .advance(world, &mut choice_provider)
                .expect(&format!("Failed to apply level {} for {}", level, name));
        }
    }

    fn set_ability_scores(
        world: &mut World,
        entity: Entity,
        ability_scores: HashMap<Ability, i32>,
    ) {
        let mut ability_score_set =
            systems::helpers::get_component_mut::<AbilityScoreSet>(world, entity);
        for (ability, score) in ability_scores {
            ability_score_set.set(ability, AbilityScore::new(ability, score));
        }
    }
}

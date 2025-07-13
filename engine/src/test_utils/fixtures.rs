pub mod armor {
    use crate::items::{
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

    use crate::{
        combat::damage::DamageType,
        dice::dice::{DiceSet, DieSize},
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
            1,
            DieSize::D4,
            DamageType::Piercing,
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
            1,
            DieSize::D8,
            DamageType::Piercing,
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
            1,
            DieSize::D6,
            DamageType::Piercing,
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
            2,
            DieSize::D6,
            DamageType::Slashing,
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
            1,
            DieSize::D8,
            DamageType::Piercing,
            vec![],
        )
    }
}

pub mod equipment {
    use crate::items::{
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

    use crate::{
        creature::{
            character::Character,
            level_up::{LevelUpSelection, PredefinedChoiceProvider},
        },
        items::equipment::equipment::HandSlot,
        stats::{
            ability::{Ability, AbilityScore},
            modifier::ModifierSource,
            skill::Skill,
        },
    };

    fn apply_level_up_selection(
        character: &mut Character,
        levels: u8,
        responses: Vec<LevelUpSelection>,
    ) {
        let mut choice_provider =
            PredefinedChoiceProvider::new(character.name().to_string(), responses);
        for level in 1..=levels {
            let mut level_up_session = character.level_up();
            level_up_session
                .advance(&mut choice_provider)
                .expect(&format!(
                    "Failed to apply level {} for {}",
                    level,
                    character.name()
                ));
        }
    }

    pub mod heroes {
        use std::collections::HashSet;

        use crate::{
            creature::{
                classes::class::{ClassName, SubclassName},
                level_up::LevelUpSelection,
            },
            registry,
            test_utils::fixtures,
        };

        use super::*;

        pub fn add_initiative(hero: &mut Character) {
            hero.skills_mut().add_modifier(
                Skill::Initiative,
                ModifierSource::Custom("Admin testing".to_string()),
                20,
            );
        }

        pub fn fighter() -> Character {
            let mut character = Character::new("Johnny Fighter");
            apply_level_up_selection(
                &mut character,
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

            for (ability, score) in ability_scores {
                character
                    .ability_scores_mut()
                    .set(ability, AbilityScore::new(ability, score));
            }

            character.equip_armor(fixtures::armor::heavy_armor());
            let _ =
                character.equip_weapon(fixtures::weapons::greatsword_two_handed(), HandSlot::Main);

            character
        }

        pub fn wizard() -> Character {
            let mut character = Character::new("Jimmy Wizard");
            apply_level_up_selection(
                &mut character,
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

            for (ability, score) in ability_scores {
                character
                    .ability_scores_mut()
                    .set(ability, AbilityScore::new(ability, score));
            }

            character.equip_armor(fixtures::armor::clothing());

            // TODO: Spellcasting ability should be set automatically based on class
            character
                .spellbook_mut()
                .add_spell(&registry::spells::MAGIC_MISSILE_ID, Ability::Intelligence);
            character
                .spellbook_mut()
                .add_spell(&registry::spells::FIREBALL_ID, Ability::Intelligence);
            // TODO: This should be set automatically based on class
            character.spellbook_mut().set_max_prepared_spells(5);

            character
        }

        pub fn warlock() -> Character {
            let mut character = Character::new("Bobby Warlock");
            apply_level_up_selection(
                &mut character,
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

            for (ability, score) in ability_scores {
                character
                    .ability_scores_mut()
                    .set(ability, AbilityScore::new(ability, score));
            }

            character.equip_armor(fixtures::armor::clothing());

            character.spellbook_mut().update_spell_slots(5);
            character
                .spellbook_mut()
                .add_spell(&registry::spells::ELDRITCH_BLAST_ID, Ability::Charisma);

            character
        }
    }

    pub mod monsters {
        use std::collections::HashSet;

        use crate::{
            creature::{classes::class::ClassName, level_up::LevelUpSelection},
            registry,
            test_utils::fixtures,
        };

        use super::*;

        pub fn goblin_warrior() -> Character {
            let mut character = Character::new("Goblin Warrior");
            // TODO: Not sure how to handle monster level-ups yet
            apply_level_up_selection(
                &mut character,
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

            for (ability, score) in ability_scores {
                character
                    .ability_scores_mut()
                    .set(ability, AbilityScore::new(ability, score));
            }

            character.equip_armor(fixtures::armor::medium_armor());
            let _ = character.equip_weapon(fixtures::weapons::dagger_light(), HandSlot::Main);

            character
        }
    }
}

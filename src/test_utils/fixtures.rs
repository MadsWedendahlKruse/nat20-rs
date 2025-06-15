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

            character.spellbook_mut().update_spell_slots(5);

            // TODO: Spellcasting ability should be set automatically based on class
            character
                .spellbook_mut()
                .add_spell(fixtures::spells::magic_missile(), Ability::Intelligence);
            character
                .spellbook_mut()
                .add_spell(fixtures::spells::fireball(), Ability::Intelligence);

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
                .add_spell(fixtures::spells::eldritch_blast(), Ability::Charisma);

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

pub mod spells {
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        actions::action::{
            ActionContext, ActionKind, AreaShape, TargetType, TargetingContext, TargetingKind,
        },
        combat::damage::{DamageRoll, DamageSource, DamageType},
        dice::dice::DieSize,
        math::point::Point,
        registry,
        spells::spell::{MagicSchool, Spell},
        stats::{ability::Ability, modifier::ModifierSource},
        utils::id::SpellId,
    };

    pub fn magic_missile() -> Spell {
        Spell::new(
            "Magic Missile".to_string(),
            1,
            MagicSchool::Evocation,
            ActionKind::UnconditionalDamage {
                damage: Arc::new(|_, _| {
                    // TODO: Damage roll hooks? e.g. Empowered Evocation
                    let mut damage_roll = DamageRoll::new(
                        1,
                        DieSize::D4,
                        DamageType::Force,
                        DamageSource::Spell,
                        "Magic Missile".to_string(),
                    );
                    damage_roll
                        .primary
                        .dice_roll
                        .modifiers
                        .add_modifier(ModifierSource::Spell(SpellId::from_str("MAGIC_MISSILE")), 1);

                    damage_roll
                }),
            },
            HashMap::from([(registry::resources::ACTION.clone(), 1)]),
            Arc::new(|_, action_context| {
                let spell_level = match action_context {
                    ActionContext::Spell { level } => *level,
                    // TODO: Better error message? Replace other places too
                    _ => panic!("Invalid action context"),
                };
                TargetingContext {
                    kind: TargetingKind::Multiple {
                        max_targets: 3 + (spell_level - 1),
                    },
                    range: 120,
                    valid_target_types: vec![TargetType::Character],
                }
            }),
        )
    }

    pub fn fireball() -> Spell {
        Spell::new(
            "Fireball".to_string(),
            3,
            MagicSchool::Evocation,
            ActionKind::SavingThrowDamage {
                saving_throw: Arc::new(|caster, _| {
                    Spell::spell_save_dc(caster, Ability::Dexterity)
                }),
                half_damage_on_save: true,
                damage: Arc::new(|_, action_context| {
                    let spell_level = match action_context {
                        ActionContext::Spell { level } => *level,
                        _ => panic!("Invalid action context"),
                    };
                    DamageRoll::new(
                        8 + (spell_level as u32 - 3),
                        DieSize::D6,
                        DamageType::Fire,
                        DamageSource::Spell,
                        "Fireball".to_string(),
                    )
                }),
            },
            HashMap::from([(registry::resources::ACTION.clone(), 1)]),
            Arc::new(|_, _| TargetingContext {
                kind: TargetingKind::Area {
                    shape: AreaShape::Sphere { radius: 20 },
                    // TODO: What do we do here?
                    origin: Point {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                range: 150,
                // TODO: Can also hit objects
                valid_target_types: vec![TargetType::Character],
            }),
        )
    }

    pub fn eldritch_blast() -> Spell {
        Spell::new(
            "Eldritch Blast".to_string(),
            0, // Cantrip
            MagicSchool::Evocation,
            ActionKind::AttackRollDamage {
                attack_roll: Arc::new(|caster, _| {
                    Spell::spell_attack_roll(caster, Ability::Charisma)
                }),
                damage: Arc::new(|_, _| {
                    DamageRoll::new(
                        1,
                        DieSize::D10,
                        DamageType::Force,
                        DamageSource::Spell,
                        "Eldritch Blast".to_string(),
                    )
                }),
                damage_on_failure: None,
            },
            HashMap::from([(registry::resources::ACTION.clone(), 1)]),
            Arc::new(|caster, _| {
                let caster_level = caster.total_level();
                TargetingContext {
                    kind: TargetingKind::Multiple {
                        max_targets: match caster_level {
                            1..=4 => 1,
                            5..=10 => 2,
                            11..=16 => 3,
                            _ => 4, // Level 17+ can hit up to 4 targets
                        },
                    },
                    range: 120,
                    valid_target_types: vec![TargetType::Character],
                }
            }),
        )
    }
}

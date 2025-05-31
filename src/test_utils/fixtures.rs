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
        combat::damage::{DamageComponent, DamageRoll, DamageType},
        dice::dice::{DiceSet, DiceSetRoll, DieSize},
        items::{
            equipment::{
                equipment::{EquipmentItem, EquipmentType},
                weapon::{Weapon, WeaponCategory, WeaponProperties},
            },
            item::ItemRarity,
        },
        stats::modifier::ModifierSet,
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
            create_damage_roll(1, DieSize::D4, "Dagger", DamageType::Piercing),
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
            create_damage_roll(1, DieSize::D8, "Rapier", DamageType::Piercing),
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
            create_damage_roll(1, DieSize::D6, "Trident", DamageType::Piercing),
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
            create_damage_roll(2, DieSize::D6, "Greatsword", DamageType::Slashing),
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
            create_damage_roll(1, DieSize::D8, "Longbow", DamageType::Piercing),
        )
    }

    fn create_damage_roll(
        num_dice: u32,
        die_size: DieSize,
        label: &str,
        damage_type: DamageType,
    ) -> DamageRoll {
        DamageRoll {
            primary: DamageComponent {
                dice_roll: DiceSetRoll {
                    dice: DiceSet { num_dice, die_size },
                    modifiers: ModifierSet::new(),
                    label: label.to_string(),
                },
                damage_type,
            },
            bonus: Vec::new(),
            label: label.to_string(),
        }
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
        creature::character::{Character, CharacterClass},
        items::equipment::equipment::HandSlot,
        stats::{
            ability::{Ability, AbilityScore},
            modifier::ModifierSource,
            skill::Skill,
        },
    };

    pub mod heroes {
        use crate::test_utils::fixtures;

        use super::*;

        pub fn add_initiative(hero: &mut Character) {
            hero.skills_mut().add_modifier(
                Skill::Initiative,
                ModifierSource::Custom("Admin testing".to_string()),
                20,
            );
        }

        pub fn fighter() -> Character {
            let mut classes = HashMap::new();
            classes.insert(CharacterClass::Fighter, 5);

            let mut character = Character::new("Hero", classes, 50);

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
            let mut classes = HashMap::new();
            classes.insert(CharacterClass::Wizard, 5);

            let mut character = Character::new("Hero Wizard", classes, 20);

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
            let mut classes = HashMap::new();
            classes.insert(CharacterClass::Warlock, 5);

            let mut character = Character::new("Hero Warlock", classes, 20);

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
        use crate::test_utils::fixtures;

        use super::*;

        pub fn goblin_warrior() -> Character {
            let mut classes = HashMap::new();
            classes.insert(CharacterClass::Fighter, 1);

            let mut character = Character::new("Goblin Warrior", classes, 10);

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
    use std::sync::Arc;

    use crate::{
        combat::damage::{DamageRoll, DamageType},
        dice::dice::DieSize,
        spells::spell::{MagicSchool, Spell, SpellKind, TargetingContext},
        stats::{ability::Ability, modifier::ModifierSource},
    };

    pub fn magic_missile() -> Spell {
        Spell::new(
            "Magic Missile".to_string(),
            1,
            MagicSchool::Evocation,
            SpellKind::Damage {
                damage: Arc::new(|_, _| {
                    // TODO: Damage roll hooks? e.g. Empowered Evocation
                    let mut damage_roll = DamageRoll::new(
                        1,
                        DieSize::D4,
                        DamageType::Force,
                        "Magic Missile".to_string(),
                    );
                    damage_roll
                        .primary
                        .dice_roll
                        .modifiers
                        .add_modifier(ModifierSource::Spell("MAGIC_MISSILE".to_string()), 1);

                    damage_roll
                }),
            },
            Arc::new(|_, spell_level| TargetingContext::Multiple(3 + (spell_level - 1))),
        )
    }

    pub fn fireball() -> Spell {
        Spell::new(
            "Fireball".to_string(),
            3,
            MagicSchool::Evocation,
            SpellKind::SavingThrowDamage {
                saving_throw: Ability::Dexterity,
                half_damage_on_save: true,
                damage: Arc::new(|_, spell_level| {
                    DamageRoll::new(
                        8 + (*spell_level as u32 - 3),
                        DieSize::D6,
                        DamageType::Fire,
                        "Fireball".to_string(),
                    )
                }),
            },
            Arc::new(|_, _| TargetingContext::AreaOfEffect {
                radius: 20,
                centered_on_caster: false,
            }),
        )
    }

    pub fn eldritch_blast() -> Spell {
        Spell::new(
            "Eldritch Blast".to_string(),
            0, // Cantrip
            MagicSchool::Evocation,
            SpellKind::AttackRoll {
                damage: Arc::new(|_, _| {
                    DamageRoll::new(
                        1,
                        DieSize::D10,
                        DamageType::Force,
                        "Eldritch Blast".to_string(),
                    )
                }),
                damage_on_failure: None,
            },
            Arc::new(|caster, _| {
                let caster_level = caster.total_level();
                // TODO: Could also do something more general purpose for cantrips
                TargetingContext::Multiple(match caster_level {
                    1..=4 => 1,
                    5..=10 => 2,
                    11..=16 => 3,
                    _ => 4, // Level 17+ can hit up to 4 targets
                })
            }),
        )
    }
}

pub mod armor {
    use crate::item::{
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
        item::{
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
    use crate::item::{
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

pub mod characters {
    use std::collections::HashMap;

    use crate::{
        creature::character::{Character, CharacterClass},
        item::equipment::equipment::HandSlot,
        stats::{
            ability::{Ability, AbilityScore},
            modifier::ModifierSource,
            skill::Skill,
        },
    };

    pub fn hero() -> Character {
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

        character.equip_armor(super::armor::heavy_armor());
        let _ = character.equip_weapon(super::weapons::greatsword_two_handed(), HandSlot::Main);

        character
    }

    pub fn hero_initiative() -> Character {
        let mut hero = hero();
        hero.skills_mut().add_modifier(
            Skill::Initiative,
            ModifierSource::Custom("Admin testing".to_string()),
            20,
        );
        hero
    }

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

        character.equip_armor(super::armor::medium_armor());
        let _ = character.equip_weapon(super::weapons::dagger_light(), HandSlot::Main);

        character
    }
}

extern crate nat20_rs;

mod tests {
    use std::collections::HashSet;

    use nat20_rs::{
        combat::damage::{DamageComponent, DamageRoll, DamageType},
        creature::character::Character,
        dice::dice::{DiceSet, DiceSetRoll, DieSize},
        item::{
            equipment::{
                equipment::{EquipmentItem, EquipmentType, HandSlot},
                weapon::{Weapon, WeaponCategory, WeaponProperties, WeaponType},
            },
            item::ItemRarity,
        },
        stats::{
            ability::{Ability, AbilityScore},
            modifier::{ModifierSet, ModifierSource},
            proficiency::Proficiency,
        },
    };

    #[test]
    fn character_weapon_finesse_modifier() {
        let equipment = EquipmentItem::new(
            "Rapier".to_string(),
            "A rapier".to_string(),
            2.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let damage_roll = create_damage_roll(1, DieSize::D8, "Rapier", DamageType::Piercing);
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            damage_roll,
        );

        let mut character = Character::default();
        character
            .ability_scores_mut()
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 14));
        character.ability_scores_mut().set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 16),
        );

        assert_eq!(weapon.determine_ability(&character), Ability::Dexterity);

        // Check that the damage roll uses Dexterity modifier
        let damage_roll = weapon.damage_roll(&mut character, HandSlot::Main);
        let damage_roll_result = damage_roll.roll();
        println!("{:?}", damage_roll_result);
        // Min: 1 (1d8) + 3 (Dex) = 4
        // Max: 8 (1d8) + 3 (Dex) = 11
        assert!(
            damage_roll_result.total >= 4 && damage_roll_result.total <= 11,
            "Damage roll total out of bounds: {}",
            damage_roll_result.total
        );
        // Check that the damage roll has a dex modifier
        assert!(damage_roll
            .primary
            .dice_roll
            .modifiers
            .get(&ModifierSource::Ability(Ability::Dexterity))
            .is_some());
    }

    #[test]
    fn character_versatile_weapon() {
        // Create a versatile weapon (Trident) and equip it in the main hand
        let equipment = EquipmentItem::new(
            "Trident".to_string(),
            "A trident".to_string(),
            5.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let dice_set_two_handed = DiceSet {
            num_dice: 1,
            die_size: DieSize::D8,
        };
        let trident = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([
                WeaponProperties::Versatile(dice_set_two_handed),
                WeaponProperties::Enchantment(1),
            ]),
            create_damage_roll(1, DieSize::D6, "Trident One-Handed", DamageType::Piercing),
        );

        let mut character = Character::default();
        let unequipped_weapons = character.equip_weapon(trident, HandSlot::Main).unwrap();
        // Check that nothing was unequipped and the character has a weapon in their hand
        assert_eq!(unequipped_weapons.len(), 0);
        assert!(character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Main));

        let trident = character
            .loadout()
            .weapon_in_hand(&WeaponType::Melee, HandSlot::Main)
            .unwrap();
        let damage_roll = trident.damage_roll(&character, HandSlot::Main);
        // Check that it's using the two handed dice set
        assert!(damage_roll.primary.dice_roll.dice.num_dice == 1);
        assert!(damage_roll.primary.dice_roll.dice.die_size == DieSize::D8);

        // Equip another weapon in the other hand
        let equipment = EquipmentItem::new(
            "Dagger".to_string(),
            "A dagger".to_string(),
            1.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let dagger = Weapon::new(
            equipment,
            WeaponCategory::Simple,
            HashSet::from([WeaponProperties::Light]),
            create_damage_roll(1, DieSize::D4, "Dagger", DamageType::Piercing),
        );
        let unequipped_weapons = character.equip_weapon(dagger, HandSlot::Off).unwrap();
        // Check that nothing was unequipped and the character has a weapon in their hand
        assert_eq!(unequipped_weapons.len(), 0);
        assert!(character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Off));

        let trident = character
            .loadout()
            .weapon_in_hand(&WeaponType::Melee, HandSlot::Main)
            .unwrap();
        let damage_roll = trident.damage_roll(&character, HandSlot::Main);
        // Check that it's using the one handed dice set
        assert!(damage_roll.primary.dice_roll.dice.num_dice == 1);
        assert!(damage_roll.primary.dice_roll.dice.die_size == DieSize::D6);

        // Just for the hell of it check that we can un-equip the dagger
        let unequipped_weapon = character
            .unequip_weapon(&WeaponType::Melee, HandSlot::Off)
            .unwrap();
        // Check that the dagger was unequipped
        assert!(unequipped_weapon.name() == "Dagger");
        assert!(!character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Off));
        // Check that the character still has the trident in their main hand
        assert!(character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Main));
        // Check that the trident is still using the two handed dice set
        let trident = character
            .loadout()
            .weapon_in_hand(&WeaponType::Melee, HandSlot::Main)
            .unwrap();
        let damage_roll = trident.damage_roll(&character, HandSlot::Main);
        assert!(damage_roll.primary.dice_roll.dice.num_dice == 1);
        assert!(damage_roll.primary.dice_roll.dice.die_size == DieSize::D8);
    }

    #[test]
    fn character_two_handed_weapon() {
        // Equip two one-handed weapons
        let dagger = Weapon::new(
            EquipmentItem::new(
                "Dagger".to_string(),
                "A dagger".to_string(),
                1.0,
                1,
                ItemRarity::Common,
                EquipmentType::MeleeWeapon,
            ),
            WeaponCategory::Simple,
            HashSet::from([WeaponProperties::Light]),
            create_damage_roll(1, DieSize::D4, "Dagger", DamageType::Piercing),
        );
        let trident = Weapon::new(
            EquipmentItem::new(
                "Trident".to_string(),
                "A trident".to_string(),
                5.0,
                1,
                ItemRarity::Common,
                EquipmentType::MeleeWeapon,
            ),
            WeaponCategory::Martial,
            HashSet::from([
                WeaponProperties::Versatile(DiceSet {
                    num_dice: 1,
                    die_size: DieSize::D8,
                }),
                WeaponProperties::Enchantment(1),
            ]),
            create_damage_roll(1, DieSize::D6, "Trident One-Handed", DamageType::Piercing),
        );

        let mut character = Character::default();

        let unequipped_weapons = character.equip_weapon(dagger, HandSlot::Off).unwrap();
        assert_eq!(unequipped_weapons.len(), 0);
        let unequipped_weapons = character.equip_weapon(trident, HandSlot::Main).unwrap();
        assert_eq!(unequipped_weapons.len(), 0);
        assert!(character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Main));
        assert!(character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Off));

        // Equip a two-handed weapon (Greatsword) in the main hand
        let greatsword = Weapon::new(
            EquipmentItem::new(
                "Greatsword".to_string(),
                "A greatsword".to_string(),
                5.0,
                1,
                ItemRarity::Common,
                EquipmentType::MeleeWeapon,
            ),
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::TwoHanded]),
            create_damage_roll(2, DieSize::D6, "Greatsword", DamageType::Slashing),
        );
        let unequipped_weapons = character.equip_weapon(greatsword, HandSlot::Main).unwrap();
        // Check that both weapons were unequipped
        assert_eq!(unequipped_weapons.len(), 2);
        assert!(character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Main));
        // Off-hand should be empty
        assert!(!character
            .loadout()
            .has_weapon_in_hand(&WeaponType::Melee, HandSlot::Off));
    }

    #[test]
    fn character_attack_roll_basic() {
        let equipment = EquipmentItem::new(
            "Longsword".to_string(),
            "A longsword".to_string(),
            5.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            create_damage_roll(1, DieSize::D8, "Longsword", DamageType::Slashing),
        );

        let mut character = Character::default();
        character
            .ability_scores_mut()
            .set(Ability::Strength, AbilityScore::new(Ability::Strength, 14));
        character.ability_scores_mut().set(
            Ability::Dexterity,
            AbilityScore::new(Ability::Dexterity, 16),
        );

        // Check that the attack roll uses Dexterity modifier
        let attack_roll = weapon.attack_roll(&mut character);
        assert!(attack_roll
            .modifiers()
            .get(&ModifierSource::Ability(Ability::Dexterity))
            .is_some());
        // Check that the attack roll does not have a proficiency modifier
        assert!(attack_roll
            .modifiers()
            .get(&ModifierSource::Proficiency(Proficiency::Proficient))
            .is_none());
        // Check that the attack roll does not have an enchantment modifier
        assert!(attack_roll
            .modifiers()
            .get(&ModifierSource::Item("Enchantment".to_string()))
            .is_none());
        println!("{:?}", attack_roll);
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

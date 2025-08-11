extern crate nat20_rs;

mod tests {
    use std::collections::HashSet;

    use hecs::World;
    use nat20_rs::{
        components::{
            ability::{Ability, AbilityScore, AbilityScoreMap},
            damage::DamageType,
            dice::{DiceSet, DieSize},
            items::{
                equipment::{
                    equipment::{EquipmentItem, EquipmentType, HandSlot},
                    loadout::Loadout,
                    weapon::{Weapon, WeaponCategory, WeaponProperties, WeaponType},
                },
                item::ItemRarity,
            },
            modifier::ModifierSource,
            proficiency::Proficiency,
        },
        entities::character::Character,
        systems::{self, helpers},
    };

    #[test]
    fn character_weapon_finesse_modifier() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        // Set Strength 14, Dexterity 16
        {
            let mut scores = helpers::get_component_mut::<AbilityScoreMap>(&mut world, entity);
            scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 14));
            scores.set(
                Ability::Dexterity,
                AbilityScore::new(Ability::Dexterity, 16),
            );
        }

        let equipment = EquipmentItem::new(
            "Rapier".to_string(),
            "A rapier".to_string(),
            2.0,
            1,
            ItemRarity::Common,
            EquipmentType::MeleeWeapon,
        );
        let weapon = Weapon::new(
            equipment,
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            vec![(1, DieSize::D8, DamageType::Piercing)],
            vec![],
        );

        let ability_scores = systems::helpers::get_component::<AbilityScoreMap>(&world, entity);
        assert_eq!(
            weapon.determine_ability(&ability_scores),
            Ability::Dexterity
        );

        let damage_roll = weapon.damage_roll(
            &ability_scores,
            false, // not wielding with both hands
        );
        let result = damage_roll.roll();

        println!("{:?}", result);
        assert!(
            (4..=11).contains(&result.total),
            "Damage roll out of bounds: {}",
            result.total
        );
        assert!(
            damage_roll
                .primary
                .dice_roll
                .modifiers
                .get(&ModifierSource::Ability(Ability::Dexterity))
                .is_some()
        );
    }

    #[test]
    fn character_versatile_weapon() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        // Equip trident
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
            vec![(1, DieSize::D6, DamageType::Piercing)],
            vec![],
        );
        systems::loadout::equip_weapon(&mut world, entity, trident, HandSlot::Main).unwrap();

        // Trident used with two hands
        let roll =
            systems::combat::damage_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main);
        assert_eq!(roll.primary.dice_roll.dice.num_dice, 1);
        assert_eq!(roll.primary.dice_roll.dice.die_size, DieSize::D8);

        // Equip dagger in off-hand
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
            vec![(1, DieSize::D4, DamageType::Piercing)],
            vec![],
        );
        systems::loadout::equip_weapon(&mut world, entity, dagger, HandSlot::Off).unwrap();

        // Trident now used one-handed
        let roll =
            systems::combat::damage_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main);
        assert_eq!(roll.primary.dice_roll.dice.num_dice, 1);
        assert_eq!(roll.primary.dice_roll.dice.die_size, DieSize::D6);

        // Unequip dagger
        let unequipped =
            systems::loadout::unequip_weapon(&mut world, entity, &WeaponType::Melee, HandSlot::Off)
                .unwrap();
        assert_eq!(unequipped.equipment().item.name, "Dagger");

        // Trident used with two hands again
        let roll =
            systems::combat::damage_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main);
        assert_eq!(roll.primary.dice_roll.dice.die_size, DieSize::D8);
    }

    #[test]
    fn character_two_handed_weapon() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

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
            vec![(1, DieSize::D4, DamageType::Piercing)],
            vec![],
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
            vec![(1, DieSize::D6, DamageType::Piercing)],
            vec![],
        );

        systems::loadout::equip_weapon(&mut world, entity, dagger, HandSlot::Off).unwrap();
        systems::loadout::equip_weapon(&mut world, entity, trident, HandSlot::Main).unwrap();

        // Equip greatsword
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
            vec![(2, DieSize::D6, DamageType::Slashing)],
            vec![],
        );

        let unequipped =
            systems::loadout::equip_weapon(&mut world, entity, greatsword, HandSlot::Main).unwrap();
        assert_eq!(unequipped.len(), 2);

        // Main hand has greatsword, off-hand should be empty
        let loadout = systems::helpers::get_component::<Loadout>(&world, entity);
        assert!(loadout.has_weapon_in_hand(&WeaponType::Melee, &HandSlot::Main));
        assert!(!loadout.has_weapon_in_hand(&WeaponType::Melee, &HandSlot::Off));
    }

    #[test]
    fn character_attack_roll_basic() {
        let mut world = World::new();
        let entity = world.spawn(Character::default());

        {
            let mut scores = helpers::get_component_mut::<AbilityScoreMap>(&mut world, entity);
            scores.set(Ability::Strength, AbilityScore::new(Ability::Strength, 14));
            scores.set(
                Ability::Dexterity,
                AbilityScore::new(Ability::Dexterity, 16),
            );
        }

        let longsword = Weapon::new(
            EquipmentItem::new(
                "Longsword".to_string(),
                "A longsword".to_string(),
                5.0,
                1,
                ItemRarity::Common,
                EquipmentType::MeleeWeapon,
            ),
            WeaponCategory::Martial,
            HashSet::from([WeaponProperties::Finesse]),
            vec![(1, DieSize::D8, DamageType::Slashing)],
            vec![],
        );

        systems::loadout::equip_weapon(&mut world, entity, longsword, HandSlot::Main).unwrap();

        let roll =
            systems::combat::attack_roll(&world, entity, &WeaponType::Melee, &HandSlot::Main);

        println!("{:?}", roll);
        assert!(
            roll.d20_check
                .modifiers()
                .contains_key(&ModifierSource::Ability(Ability::Dexterity))
        );
        assert!(
            !roll
                .d20_check
                .modifiers()
                .contains_key(&ModifierSource::Proficiency(Proficiency::Proficient))
        );
        assert!(
            !roll
                .d20_check
                .modifiers()
                .contains_key(&ModifierSource::Item("Enchantment".to_string()))
        );
    }
}
